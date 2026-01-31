use std::{io::Write, path::PathBuf};

use clap::Parser;
use tracing_subscriber::layer::SubscriberExt as _;

#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    #[clap(long, short, default_value_t = false)]
    verbose: bool,

    #[clap(long, default_value = "")]
    trace_file: String,
}

#[derive(Parser)]
enum Command {
    Build {
        // Ignored, here for compatibility with kustomize
        #[clap(long, default_value = "")]
        load_restrictor: String,

        // Ignored, here for compatibility with kustomize
        #[clap(long, default_value_t = false)]
        enable_alpha_plugins: bool,

        // Ignored, here for compatibility with kustomize
        #[clap(long, default_value_t = false)]
        enable_exec: bool,

        dir: PathBuf,
    },
    Debug {
        #[clap(subcommand)]
        subcommand: Debug,
    },
    Version {},
}

#[derive(Parser)]
enum Debug {
    DiffReference { dir: PathBuf },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let _guard = match (args.trace_file.as_str(), args.verbose) {
        ("", false) => None,
        ("", true) => {
            let subscriber = tracing_subscriber::Registry::default()
                .with(tracing_tree::HierarchicalLayer::new(2));
            tracing::subscriber::set_global_default(subscriber).unwrap();
            None
        }
        (trace_file, false) => {
            let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new()
                .file(trace_file)
                .build();
            let subscriber = tracing_subscriber::Registry::default().with(chrome_layer);
            tracing::subscriber::set_global_default(subscriber).unwrap();
            Some(guard)
        }
        (trace_file, true) => {
            let (chrome_layer, guard) = tracing_chrome::ChromeLayerBuilder::new()
                .file(trace_file)
                .build();
            let subscriber = tracing_subscriber::Registry::default()
                .with(chrome_layer)
                .with(tracing_tree::HierarchicalLayer::new(2));
            tracing::subscriber::set_global_default(subscriber).unwrap();
            Some(guard)
        }
    };

    match args.command {
        Command::Build { dir, .. } => {
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
        Command::Version {} => println!("{}", env!("CARGO_PKG_VERSION")),
    }

    Ok(())
}
