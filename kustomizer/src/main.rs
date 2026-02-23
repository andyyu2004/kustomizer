use std::{io::Write, path::PathBuf};

use clap::Parser;
use tracing_subscriber::layer::SubscriberExt as _;

/// A fast kustomize implementation in Rust.
#[derive(Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
    /// Enable verbose tracing output to stderr.
    #[clap(long, short, default_value_t = false)]
    verbose: bool,

    /// Write a Chrome trace to the given file path.
    #[clap(long, default_value = "")]
    trace_file: String,
}

#[derive(Parser)]
enum Command {
    /// Build a kustomization target from a directory.
    Build {
        /// Ignored, accepted for compatibility with kustomize.
        #[clap(long, default_value = "")]
        load_restrictor: String,

        /// Ignored, accepted for compatibility with kustomize.
        #[clap(long, default_value_t = false)]
        enable_alpha_plugins: bool,

        /// Ignored, accepted for compatibility with kustomize.
        #[clap(long, default_value_t = false)]
        enable_exec: bool,

        /// Path to the directory containing kustomization.yaml.
        dir: PathBuf,
    },
    /// Debugging utilities.
    Debug {
        #[clap(subcommand)]
        subcommand: Debug,
    },
    /// Print the version.
    Version {},
}

#[derive(Parser)]
enum Debug {
    /// Build a kustomization and diff the output against the reference kustomize implementation.
    ///
    /// Runs `kustomize build` on the same directory and compares the output using `dyff`.
    /// Requires `kustomize` and `dyff` to be on PATH.
    DiffReference {
        /// Path to the directory containing kustomization.yaml.
        dir: PathBuf,
    },
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
