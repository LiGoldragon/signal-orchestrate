use ethos_zero::{FileLocation, FileReader, Manifest, RustEmitter};
use std::{fs, process::Command};

struct EmptyManifest;

impl Manifest for EmptyManifest {
    fn resolve(&self, _: &str) -> Option<FileLocation> {
        None
    }
}

#[test]
fn generated_signal_is_a_byte_identical_rustfmt_wire_contract_projection() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(root.join("ethos/signal.ethos")).expect("authored interface");
    let file = FileReader::new(&EmptyManifest)
        .read(&source)
        .expect("interface embodiment");
    let generated = RustEmitter::wire_contract()
        .emit(&file)
        .expect("WireContract emission");
    let directory = std::env::temp_dir().join(format!(
        "signal-orchestrate-regenerate-{}",
        std::process::id()
    ));
    fs::create_dir_all(&directory).expect("temporary regeneration directory");
    let rendered = directory.join("signal.rs");
    fs::write(&rendered, generated).expect("temporary generated module");
    assert!(
        Command::new("rustfmt")
            .args(["--edition", "2024"])
            .arg(&rendered)
            .status()
            .expect("rustfmt invocation")
            .success(),
        "rustfmt generated module"
    );
    assert_eq!(
        fs::read(root.join("src/generated/signal.rs")).expect("committed generated module"),
        fs::read(&rendered).expect("self-regenerated module"),
        "src/generated/signal.rs must be regenerated from ethos/signal.ethos"
    );
    fs::remove_dir_all(directory).expect("temporary regeneration cleanup");
}
