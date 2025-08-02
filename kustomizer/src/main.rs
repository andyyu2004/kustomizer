use std::path::PathBuf;

use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    Build { dir: PathBuf },
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Build { dir } => kustomizer::build(dir, &mut std::io::stdout().lock())?,
    }

    Ok(())
}
