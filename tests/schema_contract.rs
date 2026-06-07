use schema_next::{SchemaEngine, SchemaIdentity, SchemaSourceArtifact, StreamRelation};
use std::path::PathBuf;

fn schema_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("lib.schema")
}

#[test]
fn signal_orchestrate_schema_lowers_ordinary_routes_and_streams() {
    let source = std::fs::read_to_string(schema_file()).expect("read signal-orchestrate schema");
    let artifact = SchemaSourceArtifact::from_schema_text(&source).expect("schema source decodes");
    let schema = artifact
        .source()
        .lower(
            &SchemaEngine::default(),
            SchemaIdentity::new("signal-orchestrate:lib", "0.2.0"),
        )
        .expect("schema lowers");

    assert_eq!(schema.input().variants.len(), 8);
    assert_eq!(schema.output().variants.len(), 12);
    assert_eq!(schema.streams().len(), 1);

    let claim = &schema.input().variants[0];
    assert_eq!(claim.name.as_str(), "Claim");
    assert_eq!(
        claim
            .payload
            .as_ref()
            .and_then(schema_next::TypeReference::plain_name)
            .map(schema_next::Name::as_str),
        Some("RoleClaim")
    );

    let watch = &schema.input().variants[6];
    assert_eq!(watch.name.as_str(), "Watch");
    let relation = watch.stream_relation.as_ref().expect("Watch opens stream");
    assert!(matches!(
        relation,
        StreamRelation::Opens(name) if name.as_str() == "ObservationStream"
    ));

    let stream = &schema.streams()[0];
    assert_eq!(stream.name.as_str(), "ObservationStream");
}
