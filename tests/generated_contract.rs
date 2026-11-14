use signal::{ByteViewable, Restorable, Signal, Signalizable};
use signal_orchestrate::{
    ConfigurationPath, ConfigurationReceipt, FlowId, Input, LockName, LockPath, LockReason,
    LockRejection, LockRequest, MetaConfigureDone, MetaSocketPath, OrchestrateNexusConfiguration,
    OrdinarySocketPath, Output, Refusal, ReleaseRejection,
};
#[cfg(feature = "datom")]
use signal_orchestrate::{Observation, ObserveSelection};

fn lock_query() -> Input {
    Input::Lock(LockRequest {
        lock_name: LockName("lock".into()),
        flow_id: FlowId("flow".into()),
        lock_path_vector: vec![LockPath("/tmp/path".into())],
        lock_reason: LockReason("test".into()),
    })
}

fn configuration() -> OrchestrateNexusConfiguration {
    OrchestrateNexusConfiguration {
        ordinary_socket_path: OrdinarySocketPath(ConfigurationPath("/tmp/orchestrate.sock".into())),
        meta_socket_path: MetaSocketPath(ConfigurationPath("/tmp/orchestrate-meta.sock".into())),
    }
}

#[test]
fn query_and_response_round_trip_as_portable_frames() {
    let query = lock_query();
    let query_frame = query.signalize().expect("signalize");
    assert!(!query_frame.bytes().is_empty());
    let received = Signal::<Input>::from(query_frame.bytes().to_vec());
    assert_eq!(received.restore().expect("restore query"), query);

    let response = Output::ReleaseRejected(ReleaseRejection::UnknownLockId);
    let response_frame = response.signalize().expect("signalize");
    assert!(!response_frame.bytes().is_empty());
    assert_eq!(
        response_frame.restore().expect("restore response"),
        response
    );
}

#[test]
fn configuration_and_subscription_responses_use_portable_contract_values() {
    let query = Input::Configure(configuration());
    let received = Signal::<Input>::from(
        query
            .signalize()
            .expect("signalize Configure")
            .bytes()
            .to_vec(),
    );
    assert_eq!(received.restore().expect("restore Configure"), query);

    let response = Output::Configured(ConfigurationReceipt {
        orchestrate_nexus_configuration: configuration(),
        meta_configure_done: MetaConfigureDone(false),
    });
    let received = Signal::<Output>::from(
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
    let mut pending = Potential::<Input>::from(rendered);
    let decoded = pending
        .actualize(&mut Budget {
            remaining: 1_024,
            reader: ReaderBudget::try_from(1_024usize).unwrap(),
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

    let selection = Input::Observe(ObserveSelection::Locks);
    let observation = Output::Observed(Observation::Locks(vec![]));
    assert_eq!(
        selection.datomize(vec![]).protosize().textualize(),
        "Observe.Locks"
    );
    assert_eq!(
        observation.datomize(vec![]).protosize().textualize(),
        "Observed.Locks.[]"
    );
}

#[test]
fn a_store_refusal_is_an_output_of_the_contract() {
    let refusal = Output::Refused(Refusal::Store("the commit failed".into()));
    let received = Signal::<Output>::from(
        refusal
            .signalize()
            .expect("signalize refusal")
            .bytes()
            .to_vec(),
    );
    assert_eq!(received.restore().expect("restore refusal"), refusal);
}

#[cfg(feature = "datom")]
#[test]
fn configured_and_refused_are_written_by_their_names() {
    use datom_codec::Datomizable;
    use protos::{Protosizable, Textualizable};

    let configured = Output::Configured(ConfigurationReceipt {
        orchestrate_nexus_configuration: configuration(),
        meta_configure_done: MetaConfigureDone(false),
    });
    assert!(
        configured
            .datomize(vec![])
            .protosize()
            .textualize()
            .starts_with("Configured.{ ")
    );
    let refused = Output::Refused(Refusal::Store("the commit failed".into()));
    assert_eq!(
        refused.datomize(vec![]).protosize().textualize(),
        "Refused.Store.«the commit failed»"
    );
}

/// Every fault a Lock request's paths can carry, each with the path at fault.
fn path_rejections() -> Vec<LockRejection> {
    vec![
        LockRejection::NoPath,
        LockRejection::RelativePath(LockPath("relative/path".into())),
        LockRejection::ParentComponent(LockPath("/a/../b".into())),
        LockRejection::NulCharacter(LockPath("/held\0phantom".into())),
        LockRejection::DuplicatePath(LockPath("/a/b".into())),
    ]
}

#[test]
fn a_path_fault_is_a_lock_rejection_of_the_contract() {
    for rejection in path_rejections() {
        let rejected = Output::LockRejected(rejection);
        let received = Signal::<Output>::from(
            rejected
                .signalize()
                .expect("signalize rejection")
                .bytes()
                .to_vec(),
        );
        assert_eq!(received.restore().expect("restore rejection"), rejected);
    }
}

#[cfg(feature = "datom")]
#[test]
fn a_path_fault_is_written_by_its_name() {
    use datom_codec::Datomizable;
    use protos::{Protosizable, Textualizable};

    let written: Vec<String> = path_rejections()
        .into_iter()
        .filter(|rejection| !matches!(rejection, LockRejection::NulCharacter(_)))
        .map(|rejection| {
            Output::LockRejected(rejection)
                .datomize(vec![])
                .protosize()
                .textualize()
        })
        .collect();
    assert_eq!(
        written,
        [
            "LockRejected.NoPath",
            "LockRejected.RelativePath.relative/path",
            "LockRejected.ParentComponent./a/../b",
            "LockRejected.DuplicatePath./a/b",
        ]
    );
}
