use anyhow::{Context, bail};
use std::{
    collections::HashSet,
    path::Path,
    process::{Command, Stdio},
};

use crate::PathExt;

// Diff against reference kustomize implementation
pub fn diff_reference_impl(path: &Path, actual: &str) -> anyhow::Result<()> {
    assert!(path.exists(), "path does not exist: {}", path.pretty());
    assert!(path.is_dir(), "path is not a directory: {}", path.pretty());

    let output = Command::new("kustomize")
        .arg("build")
        .arg("--load-restrictor=LoadRestrictionsNone")
        .arg("--enable-alpha-plugins")
        .arg("--enable-exec")
        .arg(".")
        .current_dir(path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("kustomize build")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        if !stderr.is_empty() {
            eprintln!("kustomize build failed with error: {stderr}");
        }

        bail!(
            "kustomize build failed for {} with status: {}",
            path.pretty(),
            output.status
        )
    }

    let expected = String::from_utf8(output.stdout).context("parsing kustomize output")?;

    // Order of documents and fields within objects do not matter for correctness.
    // Splitting by --- is easily broken by strings containing ---
    let expected_documents = expected
        .split("---\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(|s| serde_yaml::from_str(s).context(format!("parsing YAML document\n{s}")))
        .collect::<anyhow::Result<HashSet<serde_yaml::Value>>>()
        .context("parsing kustomize output")?;

    let actual_documents = actual
        .split("---\n")
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(serde_yaml::from_str)
        .collect::<serde_yaml::Result<HashSet<serde_yaml::Value>>>()
        .context("parsing actual output")?;

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
    let tmp_dir = Path::new("/tmp/kustomizer-test").join(path.file_name().unwrap());
    std::fs::create_dir_all(&tmp_dir).context("creating temporary directory")?;
    let actual_path = tmp_dir.join("actual.yaml");
    let expected_path = tmp_dir.join("expected.yaml");
    std::fs::write(&expected_path, &expected)?;
    std::fs::write(&actual_path, &actual)?;

    let output = Command::new("dyff")
        .arg("between")
        .arg("--color=on")
        .arg("--ignore-order-changes")
        .arg("--ignore-whitespace-changes")
        .arg("--set-exit-code")
        .arg(expected_path)
        .arg(actual_path)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .current_dir(&tmp_dir)
        .output()
        .context("running diff")?;

    match output.status.code() {
        Some(0) => Ok(()),
        Some(1) => {
            let diff = String::from_utf8_lossy(&output.stdout);
            eprintln!("dyff found differences for {}:\n{diff}", path.pretty());
            bail!("reference mismatch for test {}", path.pretty())
        }
        _ => {
            let stderr = String::from_utf8_lossy(&output.stderr);
            eprintln!("dyff failed with error: {stderr}");
            bail!(
                "dyff failed for {} with status: {}",
                path.pretty(),
                output.status
            )
        }
    }
}

pub fn format_chunks(chunks: Vec<dissimilar::Chunk<'_>>) -> String {
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
