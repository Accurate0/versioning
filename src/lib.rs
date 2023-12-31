pub mod args;

use anyhow::Context;
use git2::{BranchType, Diff, Oid, Repository, Sort};
use regex::Regex;
use semver::{BuildMetadata, Prerelease, Version};

pub fn get_parent_commit_diff(
    repo: &Repository,
    commit_id: Oid,
    pathspec: Option<String>,
) -> Result<Diff<'_>, anyhow::Error> {
    let commit = repo.find_commit(commit_id)?;
    let commit_tree = commit.tree()?;

    let parent = if commit.parent_count() > 0 {
        repo.find_commit(commit.parent_id(0)?)
            .ok()
            .and_then(|c| c.tree().ok())
    } else {
        None
    };

    let mut opts = git2::DiffOptions::new();
    opts.ignore_whitespace(true);
    if let Some(pathspec) = pathspec {
        opts.pathspec(pathspec);
    }
    opts.show_binary(true);

    let diff = repo.diff_tree_to_tree(parent.as_ref(), Some(&commit_tree), Some(&mut opts))?;

    Ok(diff)
}

pub fn get_version(args: crate::args::Args) -> Result<Version, anyhow::Error> {
    let repo = Repository::open(args.repo)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(Sort::TOPOLOGICAL | Sort::REVERSE)?;
    revwalk.push_head()?;

    let mut all_matching_commits = Vec::new();
    for rev in revwalk {
        match rev {
            Ok(rev) => {
                let diff = get_parent_commit_diff(&repo, rev, args.path.clone())?;

                let files_changed = diff
                    .deltas()
                    .map(|delta| {
                        delta
                            .new_file()
                            .path()
                            .map(|p| p.to_str().unwrap_or("").to_string())
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>();

                if !files_changed.is_empty() {
                    all_matching_commits.push(rev);
                }
            }
            Err(e) => tracing::warn!("rev error: {}", e),
        }
    }

    let main_branch = repo.find_branch(&args.main_branch_name, BranchType::Local)?;
    let current_head = repo.head()?;
    let current_branch_name = current_head.shorthand().context("invalid branch name")?;
    let is_main_branch = main_branch.is_head();

    let major_regex = Regex::new(&args.major_regex).context("major-regex did not compile")?;
    let minor_regex = Regex::new(&args.minor_regex).context("minor-regex did not compile")?;

    let mut major = 0;
    let mut minor = 1;
    let mut patch = 0;

    // TODO: currently only works for squash commits to main
    // TODO: need to handle Merge commit regex too
    let all_matching_commits = all_matching_commits.iter().skip(1);
    for commit_id in all_matching_commits {
        let commit = repo.find_commit(*commit_id)?;
        let message = commit.message().unwrap_or_default();
        if major_regex.is_match(message) {
            major += 1;
        } else if minor_regex.is_match(message) {
            minor += 1;
        } else {
            patch += 1;
        }
    }

    let commit_count = repo.graph_ahead_behind(
        current_head.target().context("HEAD must have target")?,
        main_branch
            .into_reference()
            .target()
            .context("main branch must have target")?,
    )?;

    let branch_name_regex = Regex::new(r"(/|_)")?;
    let sanitized_branch_name = branch_name_regex.replace_all(current_branch_name, "-");

    Ok(Version {
        major,
        minor,
        patch,
        pre: if is_main_branch {
            Prerelease::EMPTY
        } else {
            Prerelease::new(&sanitized_branch_name)?
        },
        build: if is_main_branch {
            BuildMetadata::EMPTY
        } else {
            BuildMetadata::new(&commit_count.0.to_string())?
        },
    })
}
