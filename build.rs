use std::{env, path::PathBuf};

use ethos_monolith::generate::{
    ComponentGeneration, ComponentGenerationOperations, GeneratedComponentOperations,
};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    for source in [
        "ethos/signal.ethos",
        "ethos/nexus.ethos",
        "ethos/sema.ethos",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }
    let generated = ComponentGeneration::new(root.join("ethos"), root.join("src/generated"))
        .generate()
        .expect("generate the committed Orchestrate contract modules from Ethos");
    generated
        .assert_all_match_existing()
        .expect("committed Orchestrate contract modules are fresh");
}
