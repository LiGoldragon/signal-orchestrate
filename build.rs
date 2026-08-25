use std::{env, fs, path::PathBuf};

use ethos_monolith::generate::{ComponentGeneration, ComponentGenerationOperations};

fn main() {
    let root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest directory"));
    for source in [
        "ethos/signal.ethos",
        "ethos/nexus.ethos",
        "ethos/sema.ethos",
    ] {
        println!("cargo:rerun-if-changed={source}");
    }
    for generated in [
        "src/generated/signal.rs",
        "src/generated/nexus.rs",
        "src/generated/sema.rs",
    ] {
        println!("cargo:rerun-if-changed={generated}");
    }
    let generated_directory =
        PathBuf::from(env::var_os("OUT_DIR").expect("Cargo OUT_DIR")).join("ethos-generated");
    ComponentGeneration::new(root.join("ethos"), &generated_directory)
        .generate()
        .expect("generate the Orchestrate contract modules from Ethos in OUT_DIR");
    for module in ["signal.rs", "nexus.rs", "sema.rs"] {
        let generated = fs::read(generated_directory.join(module))
            .expect("read Ethos projection generated in OUT_DIR");
        let committed = fs::read(root.join("src/generated").join(module))
            .expect("read committed Ethos projection");
        assert_eq!(
            generated, committed,
            "committed {module} is stale against Ethos source"
        );
    }
}
