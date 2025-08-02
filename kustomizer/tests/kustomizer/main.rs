use kustomizer::PathExt;
use std::{
    collections::{BTreeSet, HashSet},
    path::Path,
};

use anyhow::Context;

datatest_stable::harness! {
    { test = test, root = "tests/kustomizer/testdata", pattern = r".*/kustomization.yaml" },
}

fn test(path: &Path) -> datatest_stable::Result<()> {
    let mut out = std::io::Cursor::new(Vec::new());
    let base_path = path.parent().unwrap();
    let success_snapshot_path = base_path.join("output").with_extension("yaml");
    let error_snapshot_path = base_path.join("error").with_extension("stderr");

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
                        path.pretty()
                    ))?;
                }
            }
            res?;

            diff_reference_impl(path, &actual)?;
        }
        Err(err) => {
            show_reference_impl_error(path)?;

            let res = snapshot(&error_snapshot_path, &format!("{err:?}"));
            if success_snapshot_path.exists() {
                if should_update_snapshots() {
                    std::fs::remove_file(&success_snapshot_path)
                        .context("removing success snapshot")?;
                } else {
                    Err(format!(
                        "both success and error snapshots exist for {}",
                        path.pretty()
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

// Diff against reference kustomize implementation
fn show_reference_impl_error(path: &Path) -> datatest_stable::Result<()> {
    let output = std::process::Command::new("kustomize")
        .arg("build")
        .arg(".")
        .current_dir(path.parent().unwrap())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("kustomize build")?;

    if output.status.success() {
        Err(format!(
            "kustomize build succeeded for {} but expected failure",
            path.pretty()
        ))?;
    }

    let stderr = String::from_utf8_lossy(&output.stderr);
    eprintln!("{stderr}");
    Ok(())
}

// Diff against reference kustomize implementation
fn diff_reference_impl(path: &Path, actual: &str) -> datatest_stable::Result<()> {
    let parent = path.parent().unwrap();
    let output = std::process::Command::new("kustomize")
        .arg("build")
        .arg(".")
        .current_dir(parent)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .context("kustomize build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            eprintln!("kustomize build failed with error: {stderr}");
        }
        Err(format!(
            "kustomize build failed for {} with status: {}",
            path.pretty(),
            output.status
        ))?;
    }

    let expected = String::from_utf8(output.stdout).context("parsing kustomize output")?;

    // Order of documents and fields within objects do not matter for correctness.
    let expected_documents = expected
        .split("---\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(serde_yaml::from_str)
        .collect::<serde_yaml::Result<HashSet<serde_yaml::Value>>>()?;

    let actual_documents = actual
        .split("---\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(serde_yaml::from_str)
        .collect::<serde_yaml::Result<HashSet<serde_yaml::Value>>>()?;

    if expected_documents == actual_documents {
        return Ok(());
    }

    let mut expected = expected_documents
        .into_iter()
        .map(|doc| serde_yaml::to_string(&doc).unwrap())
        .collect::<Vec<_>>();
    expected.sort();

    let mut actual = actual_documents
        .into_iter()
        .map(|doc| serde_yaml::to_string(&doc).unwrap())
        .collect::<Vec<_>>();
    actual.sort();

    let actual = actual.join("---\n");
    let expected = expected.join("---\n");
    let tmp_dir = Path::new("/tmp/kustomizer-test").join(parent.file_name().unwrap());
    std::fs::create_dir_all(&tmp_dir).context("creating temporary directory")?;
    std::fs::write(tmp_dir.join("expected.yaml"), &expected)?;
    std::fs::write(tmp_dir.join("actual.yaml"), &actual)?;
    let chunks = dissimilar::diff(&expected, &actual);
    eprintln!("{}", format_chunks(chunks));

    Err(format!("reference mismatch for test {}", path.pretty()))?
}

fn snapshot(path: &Path, actual: &str) -> datatest_stable::Result<()> {
    if !path.exists() || should_update_snapshots() {
        std::fs::write(path, actual).context("writing snapshot")?;
        return Ok(());
    }

    let expected = std::fs::read_to_string(path).context("reading snapshot")?;
    if expected == actual {
        return Ok(());
    }

    let chunks = dissimilar::diff(&expected, actual);

    let formatted = format_chunks(chunks);
    println!("{formatted}");
    Err(format!("snapshot mismatch for test {}", path.pretty()))?
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
