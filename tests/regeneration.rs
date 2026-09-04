//! Proves the committed generated module is fresh: identical to what
//! ethos-zero produces from the authored ethos source.

use std::{fs, path::PathBuf, process::Command};

fn project_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

#[test]
fn committed_module_matches_ethos_zero_generation() {
    let root = project_root();
    let ethos_path = root.join("ethos/signal.ethos");
    let committed = fs::read_to_string(root.join("src/generated/signal.rs"))
        .expect("read committed generated module");

    let output_dir = tempfile::tempdir().expect("temporary generation directory");
    let result = Command::new("ethos-zero")
        .arg(format!(
            "Generate.{{ {} {} }}",
            ethos_path.display(),
            output_dir.path().display()
        ))
        .output()
        .expect("run ethos-zero");

    assert!(
        result.status.success(),
        "ethos-zero generation failed: {}",
        String::from_utf8_lossy(&result.stderr)
    );

    let generated = fs::read_to_string(output_dir.path().join("signal.rs"))
        .expect("read generated module");

    assert_eq!(
        committed, generated,
        "committed src/generated/signal.rs differs from ethos-zero output; regenerate with: ethos-zero 'Generate.{{ {} {} }}'",
        ethos_path.display(),
        root.join("src/generated").display()
    );
}
