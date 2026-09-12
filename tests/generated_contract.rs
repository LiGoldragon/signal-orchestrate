use signal::{ByteViewable, Restorable, Signal, Signalizable};
use signal_orchestrate::{
    ConfigurationReceipt, LockRequest, OrchestrateNexusConfiguration, Query, ReleaseRejection,
    Response,
};
#[cfg(feature = "datom")]
use signal_orchestrate::{Observation, ObserveSelection};

fn lock_query() -> Query {
    Query::Lock(LockRequest {
        lock_name: "lock".into(),
        flow_id: "flow".into(),
        lock_path_vector: vec!["/tmp/path".into()],
        lock_reason: "test".into(),
    })
}

fn configuration() -> OrchestrateNexusConfiguration {
    OrchestrateNexusConfiguration {
        ordinary_socket_path: "/tmp/orchestrate.sock".into(),
        meta_socket_path: "/tmp/orchestrate-meta.sock".into(),
    }
}

#[test]
fn query_and_response_round_trip_as_portable_frames() {
    let query = lock_query();
    let query_frame = query.signalize().expect("signalize");
    assert!(!query_frame.bytes().is_empty());
    let received = Signal::<Query>::from(query_frame.bytes().to_vec());
    assert_eq!(received.restore().expect("restore query"), query);

    let response = Response::ReleaseRejected(ReleaseRejection::UnknownLockId);
    let response_frame = response.signalize().expect("signalize");
    assert!(!response_frame.bytes().is_empty());
    assert_eq!(
        response_frame.restore().expect("restore response"),
        response
    );
}

#[test]
fn configuration_and_subscription_responses_use_portable_contract_values() {
    let query = Query::Configure(configuration());
    let received = Signal::<Query>::from(
        query
            .signalize()
            .expect("signalize Configure")
            .bytes()
            .to_vec(),
    );
    assert_eq!(received.restore().expect("restore Configure"), query);

    let response = Response::ConfigurationAccepted(ConfigurationReceipt {
        orchestrate_nexus_configuration: configuration(),
        meta_configure_done: false,
    });
    let received = Signal::<Response>::from(
        response
            .signalize()
            .expect("signalize receipt")
            .bytes()
            .to_vec(),
    );
    assert_eq!(received.restore().expect("restore receipt"), response);
}

#[cfg(feature = "datom")]
#[test]
fn query_round_trips_as_datom_text() {
    use datom_codec::{Actualizing, Budget, Datomizable, Potential};
    use protos::{Protosizable, ReaderBudget, Textualizable};

    let query = lock_query();
    let rendered = query.clone().datomize(vec![]).protosize().textualize();
    let mut pending = Potential::<Query>::from(rendered);
    let decoded = pending
        .actualize(&mut Budget {
            remaining: 1_024,
            reader: ReaderBudget { remaining: 1_024 },
            depth: 0,
            maximum_depth: 1_024,
        })
        .expect("restore datom query");
    assert_eq!(decoded, query);
}

#[cfg(feature = "datom")]
#[test]
fn observe_public_datom_shape_is_bare_selection_and_carried_observation() {
    use datom_codec::Datomizable;
    use protos::{Protosizable, Textualizable};

    let selection = Query::Observe(ObserveSelection::Locks);
    let observation = Response::Observed(Observation::Locks(vec![]));
    assert_eq!(
        selection.datomize(vec![]).protosize().textualize(),
        "Observe.Locks"
    );
    assert_eq!(
        observation.datomize(vec![]).protosize().textualize(),
        "Observed.Locks.[]"
    );
}
