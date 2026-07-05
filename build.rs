use std::{env, path::PathBuf};

use schema_rust::build::{CargoSchemaMetadata, GenerationDriver, GenerationPlan};

fn main() {
    SchemaBuild::from_environment().run();
}

struct SchemaBuild {
    crate_root: PathBuf,
}

impl SchemaBuild {
    fn from_environment() -> Self {
        Self {
            crate_root: PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir set")),
        }
    }

    fn run(&self) {
        println!("cargo:rerun-if-changed=schema/lib.schema");
        CargoSchemaMetadata::new("signal-orchestrate").emit_schema_directory(&self.crate_root);
        GenerationDriver::new(GenerationPlan::wire_contract(
            &self.crate_root,
            "signal-orchestrate",
            "0.5.0",
        ))
        .generate()
        .expect("generate signal-orchestrate schema artifacts")
        .write_or_check("SIGNAL_ORCHESTRATE_UPDATE_SCHEMA_ARTIFACTS")
        .expect("checked-in signal-orchestrate schema artifacts are fresh");
    }
}
