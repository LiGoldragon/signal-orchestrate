use datom::{
    EvidencedRealizing, EvidencedTextualizing, PathLockConstructing, PathLockPathConstructing,
    PathLockRegisteredConstructing, PathLockRegistrationRejectedConstructing, ProjectionViewing,
    RealizationViewing,
};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, LogVariant, NonEmpty, Reply as SignalReply,
    RootCode, SessionEpoch, SignalOperationHeads, SubReply, VariantCode, WireRoute,
};
use signal_orchestrate::{
    NativePathLockRegistered, NativePathLockRegistrationRejected,
    NativePathLockRegistrationRejection, OperationKind, OrchestrateFrame, OrchestrateFrameBody,
    OrchestrateReply, OrchestrateRequest, PathLock, PathLockRegistered,
    PathLockRegistrationRejected,
};

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn native_path_lock(name: &str, paths: &[&str], description: &str) -> datom::PathLock {
    datom::PathLock::try_new(
        name.into(),
        paths.iter().map(|path| (*path).into()).collect(),
        description.into(),
    )
    .expect("native path lock")
}

fn requested_lock() -> datom::PathLock {
    native_path_lock(
        "signal-orchestrate",
        &["/workspace//src/.", "/workspace/tests"],
        "protect the contract surface",
    )
}

fn holder_lock() -> datom::PathLock {
    native_path_lock(
        "active-holder",
        &["/workspace/holder"],
        "protect an existing surface",
    )
}

fn path_lock() -> PathLock {
    requested_lock().try_into().expect("convert to Signal")
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
fn native_path_lock_registered_converts_through_signal_losslessly() {
    let native = NativePathLockRegistered::new(requested_lock());
    let signal = PathLockRegistered::try_from(native.clone()).expect("convert to Signal");
    assert_eq!(
        NativePathLockRegistered::try_from(signal).expect("convert to Datom"),
        native
    );
}

#[test]
fn literal_datom_path_lock_round_trips_through_the_signal_carrier() {
    let native = requested_lock();
    let projected = native.textualize_evidenced().expect("project Datom");
    assert_eq!(
        projected.text().source.0,
        "PathLock.{signal-orchestrate [/workspace/src /workspace/tests] (protect the contract surface)}"
    );
    let realized = projected.text().realize_evidenced().expect("realize Datom");
    assert_eq!(realized.value(), &native);

    let signal = PathLock::try_from(realized.value().clone()).expect("convert to Signal");
    assert_eq!(
        datom::PathLock::try_from(signal).expect("convert to Datom"),
        native
    );
}

#[test]
fn literal_datom_path_lock_registered_round_trips_through_signal() {
    let native = NativePathLockRegistered::new(requested_lock());
    let projected = native.textualize_evidenced().expect("project Datom");
    assert_eq!(
        projected.text().source.0,
        "PathLockRegistered.{PathLock.{signal-orchestrate [/workspace/src /workspace/tests] (protect the contract surface)}}"
    );
    let realized = projected.text().realize_evidenced().expect("realize Datom");
    assert_eq!(realized.value(), &native);

    let signal = PathLockRegistered::try_from(realized.value().clone()).expect("convert to Signal");
    assert_eq!(
        NativePathLockRegistered::try_from(signal).expect("convert to Datom"),
        native
    );
}

#[test]
fn literal_datom_duplicate_active_name_round_trips_through_signal() {
    let native = NativePathLockRegistrationRejected::new(
        requested_lock(),
        NativePathLockRegistrationRejection::DuplicateActiveName {
            holder: holder_lock(),
        },
    );
    let projected = native.textualize_evidenced().expect("project Datom");
    assert_eq!(
        projected.text().source.0,
        "PathLockRegistrationRejected.{PathLock.{signal-orchestrate [/workspace/src /workspace/tests] (protect the contract surface)} DuplicateActiveName.{PathLock.{active-holder [/workspace/holder] (protect an existing surface)}}}"
    );
    let realized = projected.text().realize_evidenced().expect("realize Datom");
    assert_eq!(realized.value(), &native);

    let signal = PathLockRegistrationRejected::try_from(realized.value().clone())
        .expect("convert to Signal");
    assert_eq!(
        NativePathLockRegistrationRejected::try_from(signal).expect("convert to Datom"),
        native
    );
}

#[test]
fn literal_datom_path_overlap_round_trips_through_signal() {
    let native = NativePathLockRegistrationRejected::new(
        requested_lock(),
        NativePathLockRegistrationRejection::PathOverlap {
            path: datom::PathLockPath::try_new("/workspace//src/.".into())
                .expect("normalized conflict path"),
            holder: holder_lock(),
        },
    );
    let projected = native.textualize_evidenced().expect("project Datom");
    assert_eq!(
        projected.text().source.0,
        "PathLockRegistrationRejected.{PathLock.{signal-orchestrate [/workspace/src /workspace/tests] (protect the contract surface)} PathOverlap.{/workspace/src PathLock.{active-holder [/workspace/holder] (protect an existing surface)}}}"
    );
    let realized = projected.text().realize_evidenced().expect("realize Datom");
    assert_eq!(realized.value(), &native);

    let signal = PathLockRegistrationRejected::try_from(realized.value().clone())
        .expect("convert to Signal");
    assert_eq!(
        NativePathLockRegistrationRejected::try_from(signal).expect("convert to Datom"),
        native
    );
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
fn literal_path_lock_registration_frame_fixture() {
    let frame = OrchestrateRequest::Register(path_lock())
        .into_frame(exchange())
        .expect("frame");
    assert_eq!(
        frame.encode_length_prefixed().expect("encode"),
        vec![
            0, 0, 0, 194, 1, 0, 0, 0, 3, 0, 0, 0, 115, 105, 103, 110, 97, 108, 45, 111, 114, 99,
            104, 101, 115, 116, 114, 97, 116, 101, 47, 119, 111, 114, 107, 115, 112, 97, 99, 101,
            47, 115, 114, 99, 47, 119, 111, 114, 107, 115, 112, 97, 99, 101, 47, 116, 101, 115,
            116, 115, 142, 0, 0, 0, 226, 255, 255, 255, 144, 0, 0, 0, 232, 255, 255, 255, 112, 114,
            111, 116, 101, 99, 116, 32, 116, 104, 101, 32, 99, 111, 110, 116, 114, 97, 99, 116, 32,
            115, 117, 114, 102, 97, 99, 101, 2, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0, 0, 0, 0, 146, 0, 0, 0, 113, 255, 255, 255, 153, 255, 255, 255, 2, 0, 0, 0, 156, 0,
            0, 0, 161, 255, 255, 255, 181, 255, 255, 255, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
            0,
        ]
    );
}

#[test]
fn path_lock_registration_reply_frames_round_trip() {
    let registered = OrchestrateReply::PathLockRegistered(
        NativePathLockRegistered::new(requested_lock())
            .try_into()
            .expect("convert registered reply"),
    );
    assert_eq!(round_trip_reply(registered.clone()), registered);

    let duplicate = OrchestrateReply::PathLockRegistrationRejected(
        NativePathLockRegistrationRejected::new(
            requested_lock(),
            NativePathLockRegistrationRejection::DuplicateActiveName {
                holder: holder_lock(),
            },
        )
        .try_into()
        .expect("convert duplicate reply"),
    );
    assert_eq!(round_trip_reply(duplicate.clone()), duplicate);

    let overlap = OrchestrateReply::PathLockRegistrationRejected(
        NativePathLockRegistrationRejected::new(
            requested_lock(),
            NativePathLockRegistrationRejection::PathOverlap {
                path: datom::PathLockPath::try_new("/workspace//src/.".into())
                    .expect("normalized conflict path"),
                holder: holder_lock(),
            },
        )
        .try_into()
        .expect("convert overlap reply"),
    );
    assert_eq!(round_trip_reply(overlap.clone()), overlap);
}

#[test]
fn native_path_lock_constructor_rejects_invalid_data() {
    for (name, paths, description) in [
        ("empty-paths", vec![], "reject empty paths"),
        (
            "parent-path",
            vec!["/workspace/../escape"],
            "reject parent path",
        ),
        ("blank-description", vec!["/workspace/src"], "  "),
        ("   ", vec!["/workspace/src"], "reject blank name"),
    ] {
        assert!(
            datom::PathLock::try_new(
                name.into(),
                paths.into_iter().map(Into::into).collect(),
                description.into(),
            )
            .is_err()
        );
    }
}
