# signal-orchestrate

The generated ordinary Signal wire contract for Orchestrate Locks. Its source
of truth is `ethos/signal.ethos`; `build.rs` uses `ethos-monolith` to
regenerate the committed `src/generated/signal.rs` projection.

The signal channel has ContractId 1 and WireRevision 5. It carries the closed
request/reply surface:

- `OrchestrateRequest::{Lock(LockRequest), Release(LockId),
  Observe(ObserveSelection)}`.
- `OrchestrateReply::{Locked(Lock), LockRejected(LockRejection),
  Released(Lock), ReleaseRejected(ReleaseRejection), Observed(Observation)}`.

The generated request root is a typed Datom root. There are no Dotos codecs,
legacy `PathLock` names, or ordinary command aliases in this crate. A Lock
snapshot always carries `LockId`, `LockName`, `FlowId`, `LockPaths`, and
`LockReason`; Observe selects `Locks` as canonical `Observe.Locks` text and
replies with a complete `LockSnapshot`.

`Frame` is generated alongside the source-owned `OrchestrateWire` binding and
can encode/decode the request/reply channel through `signal-frame`. Datom 0.5
realizes and textualizes `LockId` through its canonical bare-decimal `i64`
codec; this crate adds no contract-local integer representation.

The crate owns neither the daemon, socket lifecycle, persistence, nor CLI
argument parsing.
