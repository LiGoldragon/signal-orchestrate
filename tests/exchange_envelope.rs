//! This contract inside signal's exchange envelope, exercised as a wire.
//!
//! Every value here is framed, read back off a byte stream and attributed to
//! its exchange. Nothing here inspects source text: the one thing taken from
//! outside the crate is the digest oracle, and it is computed by the published
//! FNV-1a algorithm rather than through the path under test.

use std::io::Cursor;

use signal::{
    Answer, ByteViewable, Conclusion, ContractDigest, Contracted, Delivery, Dispatch, Ending,
    ExchangeFault, ExchangeId, ExchangeLedger, ExchangeMinting, Exchanged, FrameCapacity,
    FrameReading, FrameWriting, Greeted, Handshake, HandshakeReceipt, HandshakeRejection, Opening,
    Restorable, Signal, Signalizable,
};
use signal_orchestrate::{ETHOS, Lock, LockRequest, Observation, Query, Response};

/// FNV-1a over exactly the bytes of `ethos/signal.ethos`, taken as a signed
/// 64-bit integer and computed in Python from the published algorithm. It
/// changes when and only when the authored contract changes, which is what a
/// contract digest is for.
const ORDINARY_DIGEST: ContractDigest = 8_912_638_139_974_146_909;

/// A peer built from another source: the same envelope, a different contract.
struct OtherContract;

impl Contracted for OtherContract {
    const CONTRACT_SOURCE: &'static str = "Signal\n[]\n[]\n[]\n[ Ping.Integer ]\n";
}

/// Put a value on a byte stream the way a socket carries it, and read it back.
trait CrossesTheWire: Sized {
    fn across(&self) -> Self;
}

impl<T> CrossesTheWire for T
where
    T: Signalizable,
    Signal<T>: Restorable<T>,
{
    fn across(&self) -> Self {
        let capacity = FrameCapacity::default();
        let mut wire = Vec::new();
        wire.write_frame(&self.signalize().expect("signalize"), capacity)
            .expect("write the frame");
        let body = Cursor::new(wire)
            .read_frame(capacity)
            .expect("read the frame");
        Signal::<T>::from(body.bytes().to_vec())
            .restore()
            .expect("restore the value")
    }
}

trait Holds {
    fn held(name: &str) -> Self;
}

impl Holds for Lock {
    fn held(name: &str) -> Self {
        Self {
            lock_id: 7,
            lock_name: name.to_owned(),
            flow_id: "f6db8d".to_owned(),
            lock_path_vector: vec!["/git/github.com/LiGoldragon/orchestrate".to_owned()],
            lock_reason: "ExchangeEnvelope".to_owned(),
        }
    }
}

trait Requests {
    fn requested(name: &str) -> Self;
}

impl Requests for LockRequest {
    fn requested(name: &str) -> Self {
        Self {
            lock_name: name.to_owned(),
            flow_id: "f6db8d".to_owned(),
            lock_path_vector: vec!["/git/github.com/LiGoldragon/orchestrate".to_owned()],
            lock_reason: "ExchangeEnvelope".to_owned(),
        }
    }
}

/// A greeted ledger of the querying side, which is where exchange identifiers
/// come from.
trait Opens {
    fn greeted() -> Self;
}

impl Opens for ExchangeLedger {
    fn greeted() -> Self {
        let mut ledger = Self::default();
        ledger.greet().expect("the first greeting");
        ledger
    }
}

#[test]
fn the_contract_is_identified_by_the_digest_of_its_own_authored_source() {
    assert_eq!(<Query as Contracted>::contract_digest(), ORDINARY_DIGEST);
    assert_eq!(
        <Query as Contracted>::greeting(),
        Handshake {
            contract_digest: ORDINARY_DIGEST
        }
    );
    assert_eq!(<Query as Contracted>::CONTRACT_SOURCE, ETHOS);
}

#[test]
fn a_peer_greeting_with_this_contract_settles_the_connection() {
    assert_eq!(
        <Query as Contracted>::receipt(&<Query as Contracted>::greeting()),
        HandshakeReceipt::Greeted(ORDINARY_DIGEST)
    );
}

#[test]
fn a_peer_built_from_another_source_is_refused_rather_than_negotiated_with() {
    assert_eq!(
        <Query as Contracted>::receipt(&OtherContract::greeting()),
        HandshakeReceipt::GreetingRefused(HandshakeRejection::ContractMismatch(ORDINARY_DIGEST))
    );
}

#[test]
fn the_greeting_and_a_lock_query_cross_the_wire_as_dispatches() {
    let greeting: Dispatch<Query> = Dispatch::Greet(<Query as Contracted>::greeting());
    assert_eq!(greeting.across(), greeting);

    let mut ledger = ExchangeLedger::greeted();
    let exchange = ledger.open().expect("open an exchange");
    let opening = Dispatch::Open(Opening {
        exchange,
        query: Query::Lock(LockRequest::requested("alpha")),
    });
    let received = opening.across();
    assert_eq!(received, opening);
    let Dispatch::Open(received) = received else {
        panic!("a query crosses the wire inside the exchange it opens");
    };
    assert_eq!(received.exchange(), exchange);
    assert!(!received.is_connection_wide());
}

#[test]
fn abandoning_a_subscription_crosses_the_wire_naming_only_its_exchange() {
    let mut ledger = ExchangeLedger::greeted();
    let watched = ledger.open().expect("open the subscription");
    let other = ledger.open().expect("open a second exchange");
    let abandon: Dispatch<Query> = Dispatch::Abandon(watched);
    assert_eq!(abandon.across(), Dispatch::Abandon(watched));
    assert_ne!(watched, other);
}

#[test]
fn the_state_on_open_is_an_answer_even_when_no_lock_is_held() {
    let exchange: ExchangeId = 1;
    let opening = Delivery::Answer(Answer {
        exchange,
        response: Response::Observed(Observation::Locks(Vec::new())),
    });
    assert_eq!(
        opening.across(),
        opening,
        "an empty state on open is a frame, not an absence: a peer that must \
         tell it from a state not yet sent has only position to tell them apart"
    );
}

#[test]
fn an_observation_and_a_one_answer_reply_are_told_apart_by_exchange_alone() {
    let capacity = FrameCapacity::default();
    let mut ledger = ExchangeLedger::greeted();
    let watching = ledger.open().expect("open the subscription");
    let acquiring = ledger.open().expect("open the Lock exchange");
    let held = Lock::held("alpha");

    // Exactly the order the answering side writes them in: the state on open,
    // then the Lock's own answer, then the Lock exchange's end, then the change
    // the acquisition caused, reaching the subscription that was never closed.
    let written: Vec<Delivery<Response>> = vec![
        Delivery::Greeted(HandshakeReceipt::Greeted(ORDINARY_DIGEST)),
        Delivery::Answer(Answer {
            exchange: watching,
            response: Response::Observed(Observation::Locks(Vec::new())),
        }),
        Delivery::Answer(Answer {
            exchange: acquiring,
            response: Response::Locked(held.clone()),
        }),
        Delivery::End(Ending::completed(acquiring)),
        Delivery::Answer(Answer {
            exchange: watching,
            response: Response::Observed(Observation::Locks(vec![held.clone()])),
        }),
    ];

    let mut wire = Vec::new();
    for delivery in &written {
        wire.write_frame(&delivery.signalize().expect("signalize"), capacity)
            .expect("write one frame");
    }

    let mut stream = Cursor::new(wire);
    let mut read = Vec::new();
    for _ in 0..written.len() {
        let body = stream.read_frame(capacity).expect("read one frame");
        read.push(
            Signal::<Delivery<Response>>::from(body.bytes().to_vec())
                .restore()
                .expect("restore one delivery"),
        );
    }
    assert_eq!(read, written);

    let observations = read
        .iter()
        .filter_map(|delivery| match delivery {
            Delivery::Answer(answer) if answer.exchange() == watching => Some(&answer.response),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert_eq!(
        observations,
        vec![
            &Response::Observed(Observation::Locks(Vec::new())),
            &Response::Observed(Observation::Locks(vec![held])),
        ],
        "the subscription's frames are attributable by exchange alone, with a \
         second exchange's answer and ending interleaved between them"
    );
}

#[test]
fn a_subscriber_the_answering_side_could_not_keep_current_ends_lagged() {
    let watching: ExchangeId = 3;
    let lagged: Delivery<Response> =
        Delivery::End(Ending::faulted(watching, ExchangeFault::Lagged));
    let received = lagged.across();
    assert_eq!(received, lagged);
    let Delivery::End(ending) = received else {
        panic!("a lagged subscription ends its exchange");
    };
    assert_eq!(ending.exchange(), watching);
    assert_eq!(
        ending.conclusion,
        Conclusion::Faulted(ExchangeFault::Lagged)
    );
}
