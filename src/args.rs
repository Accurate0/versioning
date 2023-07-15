use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[arg(short, long, default_value = ".")]
    pub repo: String,
    #[arg(short, long)]
    pub path: Option<String>,
    #[arg(long, default_value = "(breaking|\\+semver:major)")]
    pub major_regex: String,
    #[arg(long, default_value = "(feature)")]
    pub minor_regex: String,
    #[arg(long, default_value = "main")]
    pub main_branch_name: String,
}
