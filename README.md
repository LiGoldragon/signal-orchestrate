# signal-orchestrate

The generated ordinary Signal wire contract for Orchestrate Locks. Its source
of truth is `ethos/signal.ethos`; the `regenerate` example invokes Ethos-zero
WireContract emission and rustfmt to produce the committed
`src/generated/signal.rs` projection.

The signal channel has ContractId 1 and WireRevision 6. It carries closed
Request, Reply, and Refusal roots:

- `Request::{Lock(LockRequest), Release(LockId),
  Observe(ObserveSelection)}`.
- `Reply::{Locked(Lock), Released(Lock), Observed(Observation)}`.
- `Refusal::{LockRejected(LockRejection), ReleaseRejected(ReleaseRejection)}`.

Every generated root is a typed Datomic root. There are no Dotos codecs,
legacy `PathLock` names, or ordinary command aliases in this crate. A Lock
snapshot always carries `LockId`, `LockName`, `FlowId`, `LockPaths`, and
`LockReason`; Observe selects `Locks` as canonical `Observe.Locks` text and
an empty observation is exactly `Observed.Locks.[]`.

`Frame` is generated alongside protocol/channel constants. Hand-owned
`SignalFrameCodec` length-prefixes, rkyv-validates, and checks those constants.
Datomic 0.7 realizes and textualizes `LockId` through its canonical
bare-decimal `i64` codec; this crate adds no contract-local integer
representation.

The crate owns neither the daemon, socket lifecycle, persistence, nor CLI
argument parsing.
