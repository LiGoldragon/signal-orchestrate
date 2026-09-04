use datomic::{DatomicActualizable, Datomic, Textualizable};
use signal_orchestrate::*;

fn lock() -> Lock {
    Lock(
        17,
        "orchestrate-interfaces".to_owned(),
        "01a04a30".to_owned(),
        vec!["/git/github.com/LiGoldragon/signal-orchestrate".to_owned()],
        "generated-contract-witness".to_owned(),
    )
}

fn assert_datom_round_trip<T>(value: T, expected_text: &str)
where
    T: Datomic + Textualizable + Clone + std::fmt::Debug + PartialEq,
{
    let text = value.textualize();
    assert_eq!(text, expected_text);
    let potential = protos::Potential::<T>::from(text);
    let round_tripped: T = potential.actualize().expect("round-trip actualize");
    assert_eq!(round_tripped, value);
}

#[test]
fn all_datom_roots_round_trip() {
    let lock = lock();

    assert_datom_round_trip(
        Request::Lock(LockRequest(
            lock.1.clone(),
            lock.2.clone(),
            lock.3.clone(),
            lock.4.clone(),
        )),
        "Lock.{ orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness }",
    );

    assert_datom_round_trip(Request::Release(-42), "Release.-42");

    assert_datom_round_trip(
        Request::Observe(ObserveSelection::Locks),
        "Observe.Locks",
    );

    assert_datom_round_trip(
        Reply::Locked(lock.clone()),
        "Locked.{ 17 orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness }",
    );

    assert_datom_round_trip(
        Reply::Released(lock.clone()),
        "Released.{ 17 orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness }",
    );

    assert_datom_round_trip(
        Reply::Observed(Observation::Locks(vec![])),
        "Observed.Locks.[]",
    );

    assert_datom_round_trip(
        Reply::LockRejected(LockRejection::DuplicateName(lock.clone())),
        "LockRejected.DuplicateName.{ 17 orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness }",
    );

    assert_datom_round_trip(
        Reply::LockRejected(LockRejection::PathOverlap(LockOverlap(
            "/git/github.com/LiGoldragon/overlap".to_owned(),
            lock,
        ))),
        "LockRejected.PathOverlap.{ /git/github.com/LiGoldragon/overlap { 17 orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness } }",
    );

    assert_datom_round_trip(
        Reply::ReleaseRejected(ReleaseRejection::UnknownLockId),
        "ReleaseRejected.UnknownLockId",
    );
}

#[test]
fn spaced_reason_uses_curly_quotes() {
    let request = Request::Lock(LockRequest(
        "orchestrate-interfaces".to_owned(),
        "01a04a30".to_owned(),
        vec!["/git/github.com/LiGoldragon/signal-orchestrate".to_owned()],
        "create isolated workspace for one authorized witness".to_owned(),
    ));
    assert_datom_round_trip(
        request,
        "Lock.{ orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] \u{201C}create isolated workspace for one authorized witness\u{201D} }",
    );
}

#[test]
fn rkyv_frame_round_trips_with_version_validation() {
    let frame = Frame(
        SIGNAL_VERSION,
        Body::Request(Request::Observe(ObserveSelection::Locks)),
    );
    let bytes = frame.encode_length_prefixed().expect("rkyv frame encodes");
    assert_eq!(
        Frame::decode_length_prefixed(&bytes).expect("rkyv frame decodes"),
        frame,
    );

    let wrong_version = Frame(
        Version(99, 0, 0),
        Body::Request(Request::Observe(ObserveSelection::Locks)),
    );
    assert!(matches!(
        wrong_version.encode_length_prefixed(),
        Err(FrameCodecError::VersionMismatch { .. })
    ));
}
