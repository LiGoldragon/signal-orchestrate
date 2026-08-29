use ethos_zero::{FileLocation, FileReader, Manifest, RustEmitter};
use std::{fs, path::PathBuf, process::Command};

struct EmptyManifest;

impl Manifest for EmptyManifest {
    fn resolve(&self, _: &str) -> Option<FileLocation> {
        None
    }
}

fn main() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let source = fs::read_to_string(root.join("ethos/signal.ethos")).expect("authored interface");
    let file = FileReader::new(&EmptyManifest)
        .read(&source)
        .expect("interface embodiment");
    let generated = RustEmitter::wire_contract()
        .emit(&file)
        .expect("WireContract emission");
    let temporary = root.join("src/generated/signal.regenerate.rs");
    fs::write(&temporary, generated).expect("temporary generated module");
    assert!(
        Command::new("rustfmt")
            .args(["--edition", "2024"])
            .arg(&temporary)
            .status()
            .expect("rustfmt invocation")
            .success(),
        "rustfmt generated module"
    );
    let rendered = fs::read(&temporary).expect("formatted generated module");
    fs::remove_file(&temporary).expect("temporary generated module cleanup");
    fs::write(root.join("src/generated/signal.rs"), rendered).expect("generated module write");
}
