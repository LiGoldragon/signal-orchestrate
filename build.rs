use std::{collections::BTreeMap, env, fs, path::PathBuf};

use core_ethos::bootstrap::{
    BootstrapCatalog, BootstrapGrammarIdentities, BootstrapPriorIdentities,
    BootstrapPriorVocabulary, BootstrapVersionPolicy, CanonicalIdentityOrder, EthosKind,
    EthosVersion, IdentitySchema, IdentitySchemaCatalog, InterfaceRole, NomosSchema, SchemaRole,
    TextualMetadataRecord, TextualMetadataSnapshot, TextualProjectionAddress,
};
use name_table::{LocalEncodedId, Name};
use rust_logos::{
    FixtureRustVocabulary, FixtureRustVocabularyIds, RustLogos, RustTypePath, RustTypePathResolver,
};
use schema_rust::{bootstrap::BootstrapInterfaceGeneration, build::CargoEthosSourceMetadata};
use sema_translator::bootstrap::{
    AuthorizedBootstrapTransition, BootstrapAuthorityIdentity, BootstrapAuthorityRevision,
    BootstrapTransactionAssembler,
};
use signal_sema_translator::{VocabularyEncodedId, VocabularyRoot};
use structural_codec::EncodedNameResolver;

#[path = "src/bootstrap_manifest.rs"]
mod bootstrap_manifest;

use bootstrap_manifest::{AuthoritySeat, DeclarationSeat};

const MODULE_PATH: &[&str] = &["signal_orchestrate", "coordination"];

fn main() {
    let crate_root = PathBuf::from(env::var_os("CARGO_MANIFEST_DIR").expect("manifest dir set"));
    let source_path = crate_root.join("schema/coordination.ethos");
    let rust_path = crate_root.join("src/schema/coordination/generated.rs");

    println!("cargo:rerun-if-changed=schema/coordination.ethos");
    println!("cargo:rerun-if-changed=src/bootstrap_manifest.rs");
    println!("cargo:rerun-if-changed=src/schema/coordination/generated.rs");

    let source = fs::read_to_string(&source_path).expect("read coordination Interface source");
    let catalog = bootstrap_catalog();
    let assembly = BootstrapTransactionAssembler::new(
        BootstrapAuthorityIdentity::new(bootstrap_manifest::AUTHORITY_IDENTITY),
        BootstrapAuthorityRevision::new(bootstrap_manifest::AUTHORITY_REVISION),
        BootstrapGrammarIdentities {
            document: universal(bootstrap_manifest::GRAMMAR_DOCUMENT_LOCAL),
            syntax: universal(bootstrap_manifest::GRAMMAR_SYNTAX_LOCAL),
        },
        catalog.clone(),
    )
    .assemble(&source, authorized_transition(&catalog))
    .expect("assemble authority-approved coordination Interface transaction");
    let rust = rust_logos();
    let type_paths = CoordinationRustTypePaths::new();

    BootstrapInterfaceGeneration::new(&assembly, &rust, &type_paths, &source_path, &rust_path)
        .generate()
        .expect("project coordination Interface from the verified transaction")
        .write_or_check("SIGNAL_ORCHESTRATE_UPDATE_INTERFACE_ARTIFACTS")
        .expect("checked coordination Interface projections are fresh");
    CargoEthosSourceMetadata::new("signal-orchestrate")
        .publish_owned_source_directory(crate_root.join("schema"));
}

fn universal(local: u16) -> VocabularyEncodedId {
    VocabularyEncodedId::new(VocabularyRoot::Universal, vec![LocalEncodedId::new(local)])
        .expect("manifest Universal identity is nonempty")
}

fn rust_identity(local: u16) -> VocabularyEncodedId {
    VocabularyEncodedId::new(VocabularyRoot::Rust, vec![LocalEncodedId::new(local)])
        .expect("manifest Rust identity is nonempty")
}

fn metadata_record(
    spelling: &str,
    identity: VocabularyEncodedId,
    owner: Option<VocabularyEncodedId>,
) -> TextualMetadataRecord {
    TextualMetadataRecord {
        address: TextualProjectionAddress {
            module_path: MODULE_PATH.iter().map(|part| (*part).to_owned()).collect(),
            lexical_owner: owner,
            visible_name: spelling.to_owned(),
        },
        encoded_name: identity,
    }
}

fn fixed_specifications() -> Vec<(AuthoritySeat, Vec<SchemaRole>)> {
    use bootstrap_manifest as manifest;

    vec![
        (
            manifest::INTERFACE_SEAT,
            vec![SchemaRole::FileKind(EthosKind::Interface)],
        ),
        (
            manifest::NEXUS_SEAT,
            vec![SchemaRole::FileKind(EthosKind::Nexus)],
        ),
        (
            manifest::SEMA_SEAT,
            vec![SchemaRole::FileKind(EthosKind::Sema)],
        ),
        (
            manifest::INPUT_SEAT,
            vec![SchemaRole::InterfaceRole(InterfaceRole::Input)],
        ),
        (
            manifest::OUTPUT_SEAT,
            vec![SchemaRole::InterfaceRole(InterfaceRole::Output)],
        ),
        (
            manifest::REFUSAL_SEAT,
            vec![SchemaRole::InterfaceRole(InterfaceRole::Refusal)],
        ),
        (
            manifest::STRING_SEAT,
            vec![SchemaRole::Nominal { persistent: true }],
        ),
        (
            manifest::INTEGER_SEAT,
            vec![SchemaRole::Nominal { persistent: true }],
        ),
        (
            manifest::BOOLEAN_SEAT,
            vec![SchemaRole::Nominal { persistent: true }],
        ),
        (
            manifest::UNIT_SEAT,
            vec![SchemaRole::Nominal { persistent: true }],
        ),
        (manifest::VECTOR_SEAT, vec![SchemaRole::Shape { arity: 1 }]),
        (manifest::OPTION_SEAT, vec![SchemaRole::Shape { arity: 1 }]),
        (manifest::MAP_SEAT, vec![SchemaRole::Shape { arity: 2 }]),
        (manifest::RESULT_SEAT, vec![SchemaRole::Shape { arity: 2 }]),
        (
            manifest::STREAM_SEAT,
            vec![
                SchemaRole::Shape { arity: 1 },
                SchemaRole::Nomos(NomosSchema::StreamInitiation { arity: 2 }),
            ],
        ),
        (
            manifest::STREAM_IDENTITY_SEAT,
            vec![SchemaRole::Shape { arity: 1 }],
        ),
    ]
}

fn bootstrap_catalog() -> BootstrapCatalog {
    use bootstrap_manifest as manifest;

    let specifications = fixed_specifications();
    let metadata = TextualMetadataSnapshot::new(
        specifications
            .iter()
            .map(|(seat, _)| metadata_record(seat.spelling, universal(seat.local), None))
            .collect(),
    )
    .expect("manifest fixed textual metadata is exact");
    let schemas = IdentitySchemaCatalog::new(
        specifications
            .iter()
            .map(|(seat, roles)| {
                IdentitySchema::new(universal(seat.local), roles.clone())
                    .expect("manifest fixed schema roles are admitted")
            })
            .collect(),
    )
    .expect("manifest fixed identities are unique");
    let priors = BootstrapPriorVocabulary::new(
        BootstrapPriorIdentities {
            interface_kind: universal(manifest::INTERFACE_SEAT.local),
            nexus_kind: universal(manifest::NEXUS_SEAT.local),
            sema_kind: universal(manifest::SEMA_SEAT.local),
            input_role: universal(manifest::INPUT_SEAT.local),
            output_role: universal(manifest::OUTPUT_SEAT.local),
            refusal_role: universal(manifest::REFUSAL_SEAT.local),
            string_type: universal(manifest::STRING_SEAT.local),
            integer_type: universal(manifest::INTEGER_SEAT.local),
            boolean_type: universal(manifest::BOOLEAN_SEAT.local),
            unit_type: universal(manifest::UNIT_SEAT.local),
            vector_shape: universal(manifest::VECTOR_SEAT.local),
            option_shape: universal(manifest::OPTION_SEAT.local),
            map_shape: universal(manifest::MAP_SEAT.local),
            result_shape: universal(manifest::RESULT_SEAT.local),
            stream_nomos: universal(manifest::STREAM_SEAT.local),
            stream_shape: universal(manifest::STREAM_SEAT.local),
            stream_identity_shape: universal(manifest::STREAM_IDENTITY_SEAT.local),
        },
        &schemas,
        &metadata,
    )
    .expect("manifest seats satisfy bootstrap prior relationships");
    let canonical_order = CanonicalIdentityOrder::new(
        specifications
            .iter()
            .map(|(seat, _)| (universal(seat.local), seat.canonical.to_be_bytes().to_vec())),
    )
    .expect("manifest fixed canonical bytes are unique");

    BootstrapCatalog::new(
        MODULE_PATH.iter().map(|part| (*part).to_owned()).collect(),
        metadata,
        schemas,
        priors,
        BootstrapVersionPolicy::exact(EthosVersion::new(1, 0, 0)),
        canonical_order,
    )
    .expect("coordination bootstrap catalog is complete")
}

fn declaration_record(seat: &DeclarationSeat) -> TextualMetadataRecord {
    metadata_record(
        seat.spelling,
        universal(seat.local),
        seat.owner_local.map(universal),
    )
}

fn authorized_transition(catalog: &BootstrapCatalog) -> AuthorizedBootstrapTransition {
    let mut after = catalog.metadata().records().to_vec();
    after.extend(
        bootstrap_manifest::DECLARATION_SEATS
            .iter()
            .map(declaration_record),
    );
    AuthorizedBootstrapTransition::new(
        TextualMetadataSnapshot::new(after)
            .expect("manifest declaration projection addresses are exact"),
        bootstrap_manifest::DECLARATION_SEATS
            .iter()
            .map(|seat| (universal(seat.local), seat.canonical.to_be_bytes().to_vec()))
            .collect(),
        BTreeMap::new(),
    )
}

#[derive(Default)]
struct RustNames(BTreeMap<VocabularyEncodedId, Name>);

impl EncodedNameResolver<VocabularyRoot> for RustNames {
    fn resolve(&self, encoded_id: &VocabularyEncodedId) -> Option<&Name> {
        self.0.get(encoded_id)
    }
}

fn rust_logos() -> RustLogos {
    let locals = bootstrap_manifest::RUST_VOCABULARY_LOCALS;
    let ids = FixtureRustVocabularyIds::new(
        rust_identity(locals[0]),
        rust_identity(locals[1]),
        rust_identity(locals[2]),
        rust_identity(locals[3]),
        rust_identity(locals[4]),
        rust_identity(locals[5]),
        rust_identity(locals[6]),
        rust_identity(locals[7]),
        rust_identity(locals[8]),
        rust_identity(locals[9]),
    );
    let mut names = RustNames::default();
    for (local, spelling) in locals.into_iter().zip([
        "NewtypeItemRecord",
        "EnumerationItemRecord",
        "VariantRecord",
        "TupleFieldRecord",
        "TypeReferenceRecord",
        "struct",
        "enum",
        "pub",
        ",",
        ";",
    ]) {
        names.0.insert(rust_identity(local), Name::new(spelling));
    }
    RustLogos::new(
        FixtureRustVocabulary::seal(ids, &names).expect("manifest Rust vocabulary is sealed"),
    )
}

struct CoordinationRustTypePaths(BTreeMap<VocabularyEncodedId, RustTypePath>);

impl CoordinationRustTypePaths {
    fn new() -> Self {
        Self(BTreeMap::from([
            (
                universal(bootstrap_manifest::STRING_SEAT.local),
                RustTypePath::try_new(vec![
                    "std".to_owned(),
                    "string".to_owned(),
                    "String".to_owned(),
                ])
                .expect("String is an explicit Rust type path"),
            ),
            (
                universal(bootstrap_manifest::INTEGER_SEAT.local),
                RustTypePath::try_new(vec!["u64".to_owned()])
                    .expect("u64 is an explicit Rust type path"),
            ),
            (
                universal(bootstrap_manifest::VECTOR_SEAT.local),
                RustTypePath::try_new(vec!["Vec".to_owned()])
                    .expect("Vec is an explicit Rust type path"),
            ),
            (
                universal(bootstrap_manifest::OPTION_SEAT.local),
                RustTypePath::try_new(vec!["Option".to_owned()])
                    .expect("Option is an explicit Rust type path"),
            ),
        ]))
    }
}

impl RustTypePathResolver for CoordinationRustTypePaths {
    fn resolve_type_path(&self, encoded_id: &VocabularyEncodedId) -> Option<&RustTypePath> {
        self.0.get(encoded_id)
    }
}
