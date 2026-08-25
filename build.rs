use std::{env, fs, path::PathBuf};

use ethos_monolith::generate::{SignalGeneration, SignalGenerationOperations};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    println!("cargo:rerun-if-changed=ethos/signal.ethos");
    println!("cargo:rerun-if-changed=src/generated/signal.rs");
    let generated_directory =
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR")).join("ethos-generated");
    SignalGeneration::new(root.join("ethos"), &generated_directory)
        .generate()
        .expect("generate the Orchestrate signal contract from Ethos in OUT_DIR");
    let generated = fs::read(generated_directory.join("signal.rs"))
        .expect("read Ethos projection generated in OUT_DIR");
    let committed =
        fs::read(root.join("src/generated/signal.rs")).expect("read committed Ethos projection");
    assert_eq!(
        generated, committed,
        "committed signal.rs is stale against Ethos source"
    );
}
