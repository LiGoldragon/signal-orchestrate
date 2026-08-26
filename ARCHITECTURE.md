# signal-orchestrate architecture

`ethos/signal.ethos` owns the ordinary wire contract. A Cargo build invokes
`ethos-monolith::SignalGeneration` for that source and rejects stale committed
`src/generated/signal.rs` output. Consequently that file is provenance-marked
generated output, not a second handwritten interface.

The `Channel.{Orchestrate 1 5}` declaration generates the source-owned
`OrchestrateWire` binding, request operation, closed reply enum, typed Datom
projection, and `signal-frame` channel declaration. Consumers use the public
re-exports from `signal_orchestrate`; this crate has no daemon policy.

The input is closed as `Lock(LockRequest)`, `Release(LockId)`, or
`Observe(ObserveSelection)`. Lock values are nominal and complete: their
snapshot includes `LockId`, `LockName`, `FlowId`, `LockPaths`, and
`LockReason`. Observe is a unit selection, canonically `Observe.Locks`, and
its reply contains `Observation::Locks(LockSnapshot { locks })`. Refusals are
`LockRejected(DuplicateName | PathOverlap)` and
`ReleaseRejected(UnknownLockId)`.

Datom owns the textual boundary. The generated `Operation` root implements
Datom realization/textualization, and Datom 0.5 supplies the canonical
bare-decimal `i64` codec used by `LockId.Integer`.
