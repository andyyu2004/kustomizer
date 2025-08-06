use std::{io::Write, path::PathBuf};

use clap::Parser;
use tracing_subscriber::layer::SubscriberExt as _;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    #[clap(long, short, default_value_t = false)]
    verbose: bool,
}

#[derive(Parser)]
enum Command {
    Build {
        dir: PathBuf,
    },
    Debug {
        #[clap(subcommand)]
        subcommand: Debug,
    },
}

#[derive(Parser)]
enum Debug {
    DiffReference { dir: PathBuf },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    if args.verbose {
        let subscriber =
            tracing_subscriber::Registry::default().with(tracing_tree::HierarchicalLayer::new(2));
        tracing::subscriber::set_global_default(subscriber).unwrap();
    }

    match args.command {
        Command::Build { dir } => {
            let resmap = kustomizer::build(dir).await?;
            let mut stdout = std::io::stdout().lock();
            writeln!(stdout, "{resmap}")?;
            stdout.flush()?;
        }
        Command::Debug { subcommand } => match subcommand {
            Debug::DiffReference { dir } => {
                let resmap = kustomizer::build(&dir).await?;
                kustomizer::dbg::diff_reference_impl(&dir, &format!("{resmap}"))?;
            }
        },
    }

    Ok(())
}
