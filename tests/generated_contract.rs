use datom::{DatomRoot, DatomText};
use protos::{Realize, SourceText};
use signal_frame::{ExchangeIdentifier, ExchangeLane, LaneSequence, SessionEpoch, WireContract};
use signal_orchestrate::{
    FlowId, Frame, Lock, LockId, LockName, LockPath, LockPaths, LockReason, LockRejection,
    LockRequest, LockSnapshot, Locks, Observation, ObserveSelection, OrchestrateReply,
    OrchestrateRequest, OrchestrateWire, ReleaseRejection,
};

fn lock() -> Lock {
    Lock {
        lock_id: LockId(17),
        lock_name: LockName("orchestrate-interfaces".into()),
        flow_id: FlowId("01a03eda".into()),
        lock_paths: LockPaths(vec![LockPath(
            "/git/github.com/LiGoldragon/signal-orchestrate".into(),
        )]),
        lock_reason: LockReason("generated contract witness".into()),
    }
}

#[test]
fn approved_contract_has_its_distinct_wire_binding_and_complete_snapshots() {
    assert_eq!(OrchestrateWire::BINDING.contract().value(), 1);
    assert_eq!(OrchestrateWire::BINDING.revision().value(), 5);

    let lock = lock();
    let request = OrchestrateRequest::Lock(LockRequest {
        lock_name: lock.lock_name.clone(),
        flow_id: lock.flow_id.clone(),
        lock_paths: lock.lock_paths.clone(),
        lock_reason: lock.lock_reason.clone(),
    });
    let observe = OrchestrateRequest::Observe(ObserveSelection::Locks);
    let replies = [
        OrchestrateReply::Locked(lock.clone()),
        OrchestrateReply::LockRejected(LockRejection::DuplicateName(lock.clone())),
        OrchestrateReply::Released(lock.clone()),
        OrchestrateReply::ReleaseRejected(ReleaseRejection::UnknownLockId),
        OrchestrateReply::Observed(Observation::Locks(LockSnapshot {
            locks: Locks(vec![lock.clone()]),
        })),
    ];

    let exchange = ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    );
    let frame = request.into_frame(exchange).expect("request frame");
    let bytes = frame.encode().expect("encode request frame");
    assert_eq!(Frame::decode(&bytes).expect("decode request frame"), frame);
    assert!(matches!(observe, OrchestrateRequest::Observe(_)));
    assert_eq!(replies.len(), 5);
}

#[test]
fn generated_datom_request_round_trips_without_legacy_command_aliases() {
    let request = OrchestrateRequest::Lock(LockRequest {
        lock_name: LockName("orchestrate-interfaces".into()),
        flow_id: FlowId("01a03eda".into()),
        lock_paths: LockPaths(vec![LockPath(
            "/git/github.com/LiGoldragon/signal-orchestrate".into(),
        )]),
        lock_reason: LockReason("generated-contract-witness".into()),
    });
    let source = request.textualize_source().expect("generated Datom text");
    assert_eq!(
        DatomText::<OrchestrateRequest>::from(source.clone())
            .realize()
            .expect("generated Datom text realizes"),
        request
    );
    assert!(!source.0.contains("PathLock"));
    assert!(!source.0.contains("Register"));

    let selection = OrchestrateRequest::Observe(ObserveSelection::Locks);
    let source = selection.textualize_source().expect("selection Datom text");
    assert_eq!(source.0, "Observe.Locks");
    assert_eq!(
        DatomText::<OrchestrateRequest>::from(source)
            .realize()
            .expect("selection realizes"),
        selection
    );

    for obsolete in ["Register.{}", "PathLock.{}", "Observe.{Locks.{Current}}"] {
        assert!(
            DatomText::<OrchestrateRequest>::from(SourceText(obsolete.into()))
                .realize()
                .is_err(),
            "obsolete contract text must reject: {obsolete}"
        );
    }
}

#[test]
fn lock_id_is_a_canonical_bare_decimal_inside_release() {
    let release = OrchestrateRequest::Release(LockId(-42));
    let source = release
        .textualize_source()
        .expect("release Datom text projects");
    assert_eq!(source.0, "Release.{-42}");
    assert_eq!(
        DatomText::<OrchestrateRequest>::from(source)
            .realize()
            .expect("release Datom text realizes"),
        release
    );

    for noncanonical in ["Release.{+42}", "Release.{042}", "Release.{-0}"] {
        assert!(
            DatomText::<OrchestrateRequest>::from(SourceText(noncanonical.into()))
                .realize()
                .is_err(),
            "noncanonical LockId must reject: {noncanonical}"
        );
    }
}
