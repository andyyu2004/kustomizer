use std::path::PathBuf;

use clap::Parser;
use tracing_subscriber::layer::SubscriberExt as _;

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
    let subscriber =
        tracing_subscriber::Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
    tracing::subscriber::set_global_default(subscriber).unwrap();

    let args = Args::parse();
    match args.command {
        Command::Build { dir } => kustomizer::build(dir, &mut std::io::stdout().lock()).await?,
    }

    Ok(())
}
