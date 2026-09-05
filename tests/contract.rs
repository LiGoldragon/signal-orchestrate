use datom_codec::{Actualizable, IncorporationBudget, Potential, Textualizable};
use protos::Text;
use signal_orchestrate::*;

fn text(value: &str) -> Text {
    value.try_into().expect("fixture text")
}

fn lock() -> Lock {
    Lock(
        17.into(),
        text("orchestrate-interfaces"),
        text("01a04a30"),
        vec![text("/git/github.com/LiGoldragon/signal-orchestrate")],
        text("generated-contract-witness"),
    )
}

fn assert_datom_round_trip<T>(value: T, expected_text: &str)
where
    T: datom_codec::Datomic
        + Textualizable<datom_codec::Datom>
        + Clone
        + std::fmt::Debug
        + PartialEq,
{
    let text = <T as Textualizable<datom_codec::Datom>>::textualize(&value);
    assert_eq!(text, expected_text);
    let potential = Potential::<T>::from(text);
    let round_tripped: T = potential
        .actualize(IncorporationBudget::try_from(1_024).expect("fixed positive budget"))
        .expect("round-trip actualize");
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

    assert_datom_round_trip(Request::Release((-42).into()), "Release.-42");

    assert_datom_round_trip(Request::Observe(ObserveSelection::Locks), "Observe.Locks");

    assert_datom_round_trip(
        Response::Locked(lock.clone()),
        "Locked.{ 17 orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness }",
    );

    assert_datom_round_trip(
        Response::Released(lock.clone()),
        "Released.{ 17 orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness }",
    );

    assert_datom_round_trip(
        Response::Observed(Observation::Locks(vec![])),
        "Observed.Locks.[]",
    );

    assert_datom_round_trip(
        Response::LockRejected(LockRejection::DuplicateName(lock.clone())),
        "LockRejected.DuplicateName.{ 17 orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness }",
    );

    assert_datom_round_trip(
        Response::LockRejected(LockRejection::PathOverlap(LockOverlap(
            text("/git/github.com/LiGoldragon/overlap"),
            lock,
        ))),
        "LockRejected.PathOverlap.{ /git/github.com/LiGoldragon/overlap { 17 orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] generated-contract-witness } }",
    );

    assert_datom_round_trip(
        Response::ReleaseRejected(ReleaseRejection::UnknownLockId),
        "ReleaseRejected.UnknownLockId",
    );
}

#[test]
fn spaced_reason_uses_curly_quotes() {
    let request = Request::Lock(LockRequest(
        text("orchestrate-interfaces"),
        text("01a04a30"),
        vec![text("/git/github.com/LiGoldragon/signal-orchestrate")],
        text("create isolated workspace for one authorized witness"),
    ));
    assert_datom_round_trip(
        request,
        "Lock.{ orchestrate-interfaces 01a04a30 [ /git/github.com/LiGoldragon/signal-orchestrate ] \u{201C}create isolated workspace for one authorized witness\u{201D} }",
    );
}

#[test]
fn rkyv_request_wire_round_trips_and_validates_back_to_the_public_contract() {
    let request = Request::Observe(ObserveSelection::Locks);
    let bytes = rkyv::to_bytes::<rkyv::rancor::Error>(&request.clone().into_wire())
        .expect("request wire encodes");
    let wire =
        rkyv::from_bytes::<RequestWire, rkyv::rancor::Error>(&bytes).expect("request wire decodes");
    assert_eq!(
        Request::try_from_wire(wire).expect("wire validates"),
        request
    );
}
