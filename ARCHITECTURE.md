# signal-orchestrate architecture

`ethos/signal.ethos` owns the ordinary wire contract. `ethos/nexus.ethos` and
`ethos/sema.ethos` are the required empty component modules. A Cargo build
invokes `ethos-monolith::ComponentGeneration` over that exact directory and
rejects stale committed output. Consequently `src/generated/signal.rs` is
provenance-marked generated output, not a second handwritten interface.

The `Channel.{Orchestrate 1 4}` declaration generates the source-owned
`OrchestrateWire` binding, request operation, closed reply enum, Dotos codecs,
and `signal-frame` channel declaration. Consumers use the public re-exports
from `signal_orchestrate`; this crate has no daemon policy.

The one concrete ordinary input is a `PathLock` containing a name, absolute
path vector, and description. Its refusal is closed:
`DuplicateActiveName(PathLock)` or `PathOverlap(PathLockOverlap)`. Release is
named by the PathLock name and may only refuse `UnknownActiveName`.
