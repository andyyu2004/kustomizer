use std::path::PathBuf;

use clap::Parser;
use kustomizer::manifest::Kustomization;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Parser)]
enum Command {
    Build { dir: PathBuf },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Build { dir } => {
            let file = std::fs::File::open(dir.join("kustomization.yaml"))?;
            let kustomization = serde_yaml::from_reader::<_, Kustomization>(file)?;
            serde_yaml::to_writer(std::io::stdout(), &kustomization)?;
        }
    }

    Ok(())
}
