use clap::Parser;
use versioning::args::Args;
use versioning::get_version;

mod args;

// TODO: add CI
// TODO: trim initial ./ if it's set for --path
fn main() -> Result<(), anyhow::Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .with_ansi(true)
        .without_time()
        .with_level(false)
        .with_target(false)
        .init();

    let args: Args = Args::parse();
    let version = get_version(args)?;

    println!("{}", version);

    Ok(())
}
