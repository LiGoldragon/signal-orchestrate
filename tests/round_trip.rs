use datom::{EvidencedRealizing, EvidencedTextualizing, ProjectionViewing, RealizationViewing};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, LogVariant, NonEmpty, Reply as SignalReply,
    RootCode, SessionEpoch, SignalOperationHeads, SubReply, VariantCode, WireRoute,
};
use signal_orchestrate::{
    OperationKind, OrchestrateFrame, OrchestrateFrameBody, OrchestrateReply, OrchestrateRequest,
    PathLock, PathLockRegistered, PathLockRegistrationRejected, PathLockRegistrationRejection,
};

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn native_path_lock() -> datom::PathLock {
    datom::PathLock {
        name: "signal-orchestrate".into(),
        paths: vec!["/workspace//src/.".into(), "/workspace/tests".into()],
        description: "protect the contract surface".into(),
    }
}

fn canonical_path_lock() -> datom::PathLock {
    datom::PathLock {
        name: "signal-orchestrate".into(),
        paths: vec!["/workspace/src".into(), "/workspace/tests".into()],
        description: "protect the contract surface".into(),
    }
}

fn path_lock() -> PathLock {
    PathLock::try_from(native_path_lock()).expect("native path lock")
}

fn round_trip_request(request: OrchestrateRequest) -> OrchestrateRequest {
    let route = WireRoute::try_from_log_variant(request.log_variant()).expect("request route");
    let frame = request.into_frame(exchange()).expect("frame");
    assert_eq!(frame.short_header().route(), route);

    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = OrchestrateFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        OrchestrateFrameBody::Request { request, .. } => request.payloads().head().clone(),
        other => panic!("expected request frame, got {other:?}"),
    }
}

fn round_trip_reply(reply: OrchestrateReply) -> OrchestrateReply {
    let frame = OrchestrateFrame::new(
        WireRoute::new(RootCode::new(0), VariantCode::new(0)),
        OrchestrateFrameBody::Reply {
            exchange: exchange(),
            reply: SignalReply::committed(NonEmpty::single(SubReply::Ok(reply))),
        },
    );
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = OrchestrateFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        OrchestrateFrameBody::Reply { reply, .. } => match reply {
            SignalReply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
                other => panic!("expected accepted reply payload, got {other:?}"),
            },
            other => panic!("expected accepted reply, got {other:?}"),
        },
        other => panic!("expected reply frame, got {other:?}"),
    }
}

#[test]
fn literal_datom_path_lock_round_trips_through_the_signal_carrier() {
    let native = native_path_lock();
    let projected = native.textualize_evidenced().expect("project Datom");
    assert_eq!(
        projected.text().source.0,
        "PathLock.{signal-orchestrate [/workspace/src /workspace/tests] (protect the contract surface)}"
    );
    let realized = projected.text().realize_evidenced().expect("realize Datom");
    assert_eq!(realized.value(), &canonical_path_lock());

    let signal = PathLock::try_from(realized.value().clone()).expect("convert to Signal");
    assert_eq!(datom::PathLock::from(signal), canonical_path_lock());
}

#[test]
fn literal_path_lock_registration_frame_round_trips() {
    let request = OrchestrateRequest::Register(path_lock());
    assert_eq!(round_trip_request(request.clone()), request);
    assert_eq!(request.kind(), OperationKind::Register);
    assert_eq!(
        <OrchestrateRequest as SignalOperationHeads>::HEADS,
        ["Register"]
    );
}

#[test]
fn path_lock_registration_replies_round_trip() {
    let registered = OrchestrateReply::PathLockRegistered(PathLockRegistered { lock: path_lock() });
    assert_eq!(round_trip_reply(registered.clone()), registered);

    let duplicate = OrchestrateReply::PathLockRegistrationRejected(PathLockRegistrationRejected {
        lock: path_lock(),
        rejection: PathLockRegistrationRejection::DuplicateActiveName {
            holder: "signal-orchestrate".into(),
        },
    });
    assert_eq!(round_trip_reply(duplicate.clone()), duplicate);

    let overlap = OrchestrateReply::PathLockRegistrationRejected(PathLockRegistrationRejected {
        lock: path_lock(),
        rejection: PathLockRegistrationRejection::PathOverlap {
            path: "/workspace/src".into(),
            holder: "existing-holder".into(),
        },
    });
    assert_eq!(round_trip_reply(overlap.clone()), overlap);
}

#[test]
fn path_lock_carrier_admits_only_native_valid_path_lock_data() {
    for invalid in [
        datom::PathLock {
            name: "empty-paths".into(),
            paths: vec![],
            description: "reject empty paths".into(),
        },
        datom::PathLock {
            name: "parent-path".into(),
            paths: vec!["/workspace/../escape".into()],
            description: "reject parent path".into(),
        },
        datom::PathLock {
            name: "blank-description".into(),
            paths: vec!["/workspace/src".into()],
            description: "  ".into(),
        },
    ] {
        assert!(PathLock::try_from(invalid).is_err());
    }
}
