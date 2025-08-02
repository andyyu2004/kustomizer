use std::path::PathBuf;

use anyhow::Context;
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

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    match args.command {
        Command::Build { dir } => {
            let kustomization = kustomizer::load_kustomization(dir)?;
            dbg!(&kustomization.value);
            serde_yaml::to_writer(std::io::stdout(), &kustomization.value)?;
            let build_context = kustomizer::build::Builder::default()
                .build(&kustomization)
                .await
                .context("gathering build context")?;

            dbg!(&build_context);
        }
    }

    Ok(())
}
