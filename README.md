# signal-orchestrate

The generated ordinary Signal wire contract for Orchestrate PathLocks. Its
source of truth is `ethos/signal.ethos`; `build.rs` uses `ethos-monolith` to
regenerate the committed `src/generated/signal.rs` projection.

The signal channel has ContractId 1 and WireRevision 4. It carries the closed
request/reply surface:

- `OrchestrateRequest::Register(PathLock)` and `::Release(PathLockRelease)`.
- `OrchestrateReply::{PathLockRegistered, PathLockRegistrationRejected,
  PathLockReleased, PathLockReleaseRejected}`.

The ordinary textual contact points are concrete payloads, rather than their
wire enum envelopes:

```text
PathLock.{orchestrate-interfaces [/git/github.com/LiGoldragon/signal-orchestrate] (generated contract witness)}
PathLockRelease.{orchestrate-interfaces}
```

`Frame` is generated alongside the source-owned `OrchestrateWire` binding and
can encode/decode the request/reply channel through `signal-frame`.

The crate owns neither the daemon, socket lifecycle, persistence, nor CLI
argument parsing.
