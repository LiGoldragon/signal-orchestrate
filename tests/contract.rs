use datomic::Datomic;
use protos::PortionText;
use signal_orchestrate::{
    CHANNEL_CONTRACT_ID, CHANNEL_WIRE_REVISION, ChannelContractId, ChannelWireRevision, FlowId,
    Frame, FrameBody, FrameCodecError, INTERFACE_VERSION, Lock, LockId, LockName, LockOverlap,
    LockPath, LockPaths, LockReason, LockRejection, LockRequest, Locks, Observation,
    ObserveSelection, ProtocolVersion, Refusal, ReleaseRejection, Reply, Request, SignalFrameCodec,
};

fn name(value: &str) -> LockName {
    LockName::try_from(value).expect("representable name")
}

fn flow(value: &str) -> FlowId {
    FlowId::try_from(value).expect("representable flow")
}

fn path(value: &str) -> LockPath {
    LockPath::try_from(value).expect("representable path")
}

fn reason(value: &str) -> LockReason {
    LockReason::try_from(value).expect("representable reason")
}

fn lock() -> Lock {
    Lock {
        lock_id: LockId(17),
        lock_name: name("orchestrate-interfaces"),
        flow_id: flow("01a04a30"),
        lock_paths: LockPaths(vec![path("/git/github.com/LiGoldragon/signal-orchestrate")]),
        lock_reason: reason("generated-contract-witness"),
    }
}

fn assert_datom_root<Value>(value: Value, expected: &str)
where
    Value: Datomic + Clone + std::fmt::Debug + PartialEq,
{
    let portion = value.portion();
    assert_eq!(portion.canonical_text().as_ref(), expected);
    assert_eq!(
        Value::embody(&portion).expect("Datomic root realizes"),
        value
    );
}

#[test]
fn all_ordinary_datom_roots_round_trip_with_lock_release_refusal_and_observation() {
    let lock = lock();
    let request = Request::Lock(LockRequest {
        lock_name: lock.lock_name.clone(),
        flow_id: lock.flow_id.clone(),
        lock_paths: lock.lock_paths.clone(),
        lock_reason: lock.lock_reason.clone(),
    });
    assert_datom_root(
        request,
        "Lock.{orchestrate-interfaces 01a04a30 [/git/github.com/LiGoldragon/signal-orchestrate] generated-contract-witness}",
    );
    assert_datom_root(Request::Release(LockId(-42)), "Release.-42");
    assert_datom_root(Request::Observe(ObserveSelection::Locks), "Observe.Locks");
    assert_datom_root(
        Reply::Locked(lock.clone()),
        "Locked.{17 orchestrate-interfaces 01a04a30 [/git/github.com/LiGoldragon/signal-orchestrate] generated-contract-witness}",
    );
    assert_datom_root(
        Reply::Released(lock.clone()),
        "Released.{17 orchestrate-interfaces 01a04a30 [/git/github.com/LiGoldragon/signal-orchestrate] generated-contract-witness}",
    );
    assert_datom_root(
        Reply::Observed(Observation::Locks(Locks(vec![]))),
        "Observed.Locks.[]",
    );
    assert_datom_root(
        Refusal::LockRejected(LockRejection::DuplicateName(lock.clone())),
        "LockRejected.DuplicateName.{17 orchestrate-interfaces 01a04a30 [/git/github.com/LiGoldragon/signal-orchestrate] generated-contract-witness}",
    );
    assert_datom_root(
        Refusal::LockRejected(LockRejection::PathOverlap(LockOverlap {
            lock_path: path("/git/github.com/LiGoldragon/overlap"),
            lock,
        })),
        "LockRejected.PathOverlap.{/git/github.com/LiGoldragon/overlap {17 orchestrate-interfaces 01a04a30 [/git/github.com/LiGoldragon/signal-orchestrate] generated-contract-witness}}",
    );
    assert_datom_root(
        Refusal::ReleaseRejected(ReleaseRejection::UnknownLockId),
        "ReleaseRejected.UnknownLockId",
    );
}

#[test]
fn rkyv_frame_is_length_prefixed_validated_and_bound_to_ordinary_constants() {
    assert_eq!(CHANNEL_CONTRACT_ID, ChannelContractId(1));
    assert_eq!(CHANNEL_WIRE_REVISION, ChannelWireRevision(6));
    assert_eq!(INTERFACE_VERSION, ProtocolVersion::new(0, 3, 0));
    let frame = Frame {
        channel_contract_id: CHANNEL_CONTRACT_ID,
        channel_wire_revision: CHANNEL_WIRE_REVISION,
        protocol_version: INTERFACE_VERSION,
        body: FrameBody::Request(Request::Observe(ObserveSelection::Locks)),
    };
    let bytes = frame.encode_length_prefixed().expect("rkyv frame encodes");
    assert_eq!(
        Frame::decode_length_prefixed(&bytes).expect("rkyv frame validates"),
        frame
    );
    let wrong_channel = Frame {
        channel_contract_id: ChannelContractId(99),
        ..frame
    };
    assert!(matches!(
        wrong_channel.encode_length_prefixed(),
        Err(FrameCodecError::WrongChannelContract { .. })
    ));
}
