use std::path::Path;

use anyhow::Context;

datatest_stable::harness! {
    { test = test, root = "tests/kustomizer/testdata", pattern = r".*/kustomization.yaml" },
}

fn test(path: &Path) -> datatest_stable::Result<()> {
    let mut out = std::io::Cursor::new(Vec::new());

    match kustomizer::build(path, &mut out) {
        Ok(()) => {
            let actual = String::from_utf8(out.into_inner())?;
            let snapshot_path = path
                .parent()
                .unwrap()
                .join("expected")
                .with_extension("yaml");
            snapshot(&snapshot_path, &actual)?;
        }
        Err(err) => {
            eprintln!(
                "Error building kustomization at {}: {}",
                path.display(),
                err
            );

            let expected_path = path.join("expected").with_extension("stderr");
            snapshot(&expected_path, &format!("{:?}", err))?;
        }
    }
    Ok(())
}

fn snapshot(path: &Path, actual: &str) -> datatest_stable::Result<()> {
    if !path.exists() || std::env::var("UPDATE_SNAPSHOTS").is_ok() {
        std::fs::write(path, actual).context("writing snapshot")?;
        return Ok(());
    }

    let expected = std::fs::read_to_string(path).context("reading snapshot")?;
    let chunks = dissimilar::diff(&expected, actual);
    if chunks.is_empty() {
        return Ok(());
    }

    let formatted = format_chunks(chunks);
    eprintln!("Snapshot mismatch for {}:\n{}", path.display(), formatted);

    Err(format!("Snapshot mismatch for {}", path.display()).into())
}

fn format_chunks(chunks: Vec<dissimilar::Chunk>) -> String {
    let mut buf = String::new();
    for chunk in chunks {
        let formatted = match chunk {
            dissimilar::Chunk::Equal(text) => text.into(),
            dissimilar::Chunk::Delete(text) => format!("\x1b[4m\x1b[31m{}\x1b[0m", text),
            dissimilar::Chunk::Insert(text) => format!("\x1b[4m\x1b[32m{}\x1b[0m", text),
        };
        buf.push_str(&formatted);
    }
    buf
}
