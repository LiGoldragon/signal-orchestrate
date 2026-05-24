use schema::{Leg, LoadedSchema, RouteBody};
use std::path::PathBuf;

fn schema_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("signal-orchestrate.concept.schema")
}

#[test]
fn signal_orchestrate_concept_schema_lowers_ordinary_routes() {
    let loaded = LoadedSchema::read_path(schema_file()).expect("signal-orchestrate schema reads");
    let assembled = loaded.assembled();

    assert_eq!(assembled.routes().len(), 8);

    let claim = assembled
        .route_for_short_header(Leg::Ordinary, u64::from_le_bytes([0, 0, 0, 0, 0, 0, 0, 0]))
        .expect("claim route");
    assert_eq!(claim.root().as_str(), "Claim");
    assert_eq!(claim.endpoint().name().as_str(), "RoleClaim");
    assert!(matches!(claim.body(), RouteBody::Type(name) if name.as_str() == "RoleClaim"));

    assert!(
        assembled
            .route_for_short_header(Leg::Owner, u64::from_le_bytes([0, 0, 0, 0, 0, 0, 0, 0]))
            .is_none()
    );
}
