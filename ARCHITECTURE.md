# signal-orchestrate architecture

`ethos/signal.ethos` owns the ordinary wire contract. Ethos-zero 0.5.0 emits
the committed `src/generated/signal.rs`; the executable regeneration tool and
byte-identical rustfmt test reject stale output. Consequently that file is
provenance-marked generated output, not a second handwritten interface.

The `Channel.{Orchestrate 1 6}` declaration generates the source-owned
protocol and channel constants, closed request/reply/refusal roots, typed
Datomic anatomy, and rkyv frame record. `src/codec.rs` owns the separate
length-prefix/rkyv validation boundary. Consumers use the public re-exports
from `signal_orchestrate`; this crate has no Nexus policy.

The Request root is closed as `Lock(LockRequest)`, `Release(LockId)`, or
`Observe(ObserveSelection)`. Lock values are nominal and complete: their
snapshot includes `LockId`, `LockName`, `FlowId`, `LockPaths`, and
`LockReason`. Observe is a unit selection, canonically `Observe.Locks`, and
its Reply is `Observed(Observation::Locks(Locks))`, including the approved
empty form `Observed.Locks.[]`. Refusal is closed as
`LockRejected(DuplicateName | PathOverlap)` or
`ReleaseRejected(UnknownLockId)`.

Datomic owns the textual boundary. Every generated root implements its
source-linked Datomic anatomy, and Datomic 0.7 supplies the canonical
bare-decimal `i64` codec used by `LockId.Integer`.
