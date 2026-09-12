# signal-orchestrate architecture

`ethos/signal.ethos` is the ordinary wire contract and the only interface in
this crate. Ethos-zero emits `src/generated/signal.rs` from it, and `build.rs`
asserts the committed projection is byte-identical to what the pinned
generator produces, so the projection is generated output and never a second
hand-written interface.

The crate declares vocabulary and nothing else: no runtime, no actors, no
sockets, no policy, no argument parsing. It owns no framing and no envelope
either — both come from `signal`.

## The roots

`Query` is closed as `Configure(OrchestrateNexusConfiguration)`,
`Lock(LockRequest)`, `Release(LockId)` and `Observe(ObserveSelection)`.

`Response` is closed as `ConfigurationAccepted`, `ConfigurationRefused`,
`Locked(Lock)`, `Released(Lock)`, `Observed(Observation)`,
`LockRejected(LockRejection)` and `ReleaseRejected(ReleaseRejection)`. A
refusal is a response variant of this contract, because a frame written on a
socket must be readable by the peer on it.

A `Lock` value is nominal and complete: `LockId`, `LockName`, `FlowId`, the
vector of `LockPath`, and `LockReason`. `LockId` is assigned by the Nexus and
is the sole release target. `ObserveSelection` is a unit selection, textually
`Observe.Locks`; its answer is `Observed(Observation::Locks(_))`, and the
empty state is exactly `Observed.Locks.[]`.

## Identity on the wire

`ETHOS` is the authored source, and `impl signal::Contracted for Query` makes
that source the contract's identity: `Query::contract_digest()` is the FNV-1a
digest of those bytes, and `Query::greeting()` is the `Handshake` a peer opens
a connection with. Two peers built from different sources refuse each other
with `HandshakeReceipt::GreetingRefused(ContractMismatch(_))`. There is no
version range and no negotiation.

The identity is borne by `Query` rather than by `Response` because the
querying side is the side that greets.

## What rides above and below

Below: `signal`'s four-byte big-endian length prefix and one validated rkyv
archive per frame.

Above: `signal`'s exchange layer. The querying side sends
`Dispatch<Query>` — `Greet`, `Open(Opening { exchange, query })`, or
`Abandon(exchange)`. The answering side sends `Delivery<Response>` —
`Greeted(receipt)`, `Answer(Answer { exchange, response })`, or
`End(Ending { exchange, conclusion })`. Nothing on the wire says whether an
exchange takes one answer or many; this contract's source says, and both
sides read it.

`Observe` opens an exchange that goes on answering. Its first answer is sent
even when no Lock is held, because a peer that must tell an empty state from
a state not yet sent has only position to tell them apart. A subscriber the
answering side could not keep current has its exchange ended with
`ExchangeFault::Lagged`; the recovery is to open `Observe` again, which by the
subscription's own semantics delivers the state on open.

## The Datom boundary

Every generated root is a typed Datom root behind the `datom` feature, which
the CLI clients enable and the Nexus does not — the Nexus thinks in typed
values and never textualizes. Datom-codec supplies the canonical bare-decimal
`i64` codec used by `LockId.Integer`; this crate adds no contract-local
integer representation.
