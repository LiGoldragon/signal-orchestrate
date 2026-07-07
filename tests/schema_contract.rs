use schema::{
    EnumDeclaration, Root, SchemaEngine, SchemaIdentity, SchemaSourceArtifact, StreamRelation,
};
use std::path::PathBuf;

fn schema_file() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("schema")
        .join("lib.schema")
}

fn root_enum(root: &Root) -> &EnumDeclaration {
    root.as_enum().expect("root is the enum-body form")
}

#[test]
fn signal_orchestrate_schema_lowers_ordinary_routes_and_streams() {
    let source = std::fs::read_to_string(schema_file()).expect("read signal-orchestrate schema");
    let artifact = SchemaSourceArtifact::from_schema_text(&source).expect("schema source decodes");
    let schema = artifact
        .source()
        .lower(
            &SchemaEngine::default(),
            SchemaIdentity::new("signal-orchestrate:lib", "0.5.0"),
        )
        .expect("schema lowers");

    let input = root_enum(schema.input());
    let output = root_enum(schema.output());

    assert_eq!(input.variants.len(), 12);
    assert_eq!(output.variants.len(), 22);
    assert_eq!(schema.streams().len(), 2);

    let claim = &input.variants[0];
    assert_eq!(claim.name.as_str(), "Claim");
    assert_eq!(
        claim
            .payload
            .as_ref()
            .and_then(schema::TypeReference::plain_name)
            .map(schema::Name::as_str),
        Some("RoleClaim")
    );

    let workflow_observation = &input.variants[8];
    assert_eq!(workflow_observation.name.as_str(), "ObserveWorkflowRun");
    let relation = workflow_observation
        .stream_relation
        .as_ref()
        .expect("ObserveWorkflowRun opens stream");
    assert!(matches!(
        relation,
        StreamRelation::Opens(name) if name.as_str() == "WorkflowRunStream"
    ));

    let watch = &input.variants[10];
    assert_eq!(watch.name.as_str(), "Watch");
    let relation = watch.stream_relation.as_ref().expect("Watch opens stream");
    assert!(matches!(
        relation,
        StreamRelation::Opens(name) if name.as_str() == "ObservationStream"
    ));

    let stream = &schema.streams()[0];
    assert_eq!(stream.name.as_str(), "WorkflowRunStream");

    let stream = &schema.streams()[1];
    assert_eq!(stream.name.as_str(), "ObservationStream");
}
