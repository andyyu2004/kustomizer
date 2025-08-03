// TODO anonymous the partly-argocd resources and put them in as a test.

use kustomizer::{
    PathExt,
    dbg::{diff_reference_impl, format_chunks},
};
use std::path::Path;

use anyhow::Context;

datatest_stable::harness! {
    { test = test, root = "tests/kustomizer/testdata", pattern = r".*/kustomization.yaml" },
}

#[tokio::main]
async fn test(path: &Path) -> datatest_stable::Result<()> {
    let base_path = path.parent().unwrap();
    let success_snapshot_path = base_path.join("output").with_extension("yaml");
    let error_snapshot_path = base_path.join("error").with_extension("stderr");

    match kustomizer::build(path).await {
        Ok(resmap) => {
            let actual = resmap.to_string();
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

            diff_reference_impl(base_path, &actual)?;
        }
        Err(err) => {
            show_reference_impl_error(base_path)?;

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
        .arg("--load-restrictor=LoadRestrictionsNone")
        .arg("--enable-alpha-plugins")
        .arg("--enable-exec")
        .arg(".")
        .current_dir(path)
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
