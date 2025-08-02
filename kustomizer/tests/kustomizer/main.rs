use std::path::Path;

use anyhow::Context;

datatest_stable::harness! {
    { test = test, root = "tests/kustomizer/testdata", pattern = r".*/kustomization.yaml" },
}

fn test(path: &Path) -> datatest_stable::Result<()> {
    let mut out = std::io::Cursor::new(Vec::new());
    let base_path = path.parent().unwrap();
    let success_snapshot_path = base_path.join("expected").with_extension("yaml");
    let error_snapshot_path = base_path.join("expected").with_extension("stderr");

    match kustomizer::build(path, &mut out) {
        Ok(()) => {
            let actual = String::from_utf8(out.into_inner())?;
            let res = snapshot(&success_snapshot_path, &actual);
            if error_snapshot_path.exists() {
                if should_update_snapshots() {
                    std::fs::remove_file(&error_snapshot_path)
                        .context("removing error snapshot")?;
                } else {
                    Err(format!(
                        "both success and error snapshots exist for {}",
                        path.display()
                    ))?;
                }
            }
            res?;
        }
        Err(err) => {
            eprintln!(
                "Error building kustomization at {}: {}",
                path.display(),
                err
            );

            let res = snapshot(&error_snapshot_path, &format!("{:?}", err));
            if success_snapshot_path.exists() {
                if should_update_snapshots() {
                    std::fs::remove_file(&success_snapshot_path)
                        .context("removing success snapshot")?;
                } else {
                    Err(format!(
                        "both success and error snapshots exist for {}",
                        path.display()
                    ))?;
                }
            }
            res?;
        }
    }
    Ok(())
}

fn should_update_snapshots() -> bool {
    std::env::var("UPDATE_SNAPSHOTS").is_ok()
}

fn snapshot(path: &Path, actual: &str) -> datatest_stable::Result<()> {
    if !path.exists() || should_update_snapshots() {
        std::fs::write(path, actual).context("writing snapshot")?;
        return Ok(());
    }

    let expected = std::fs::read_to_string(path).context("reading snapshot")?;
    let chunks = dissimilar::diff(&expected, actual);
    if chunks.is_empty() {
        return Ok(());
    }

    let formatted = format_chunks(chunks);
    println!("{formatted}");
    Err(format!("Snapshot mismatch for {}", path.display()))?
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
