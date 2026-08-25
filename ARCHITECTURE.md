# signal-orchestrate architecture

`ethos/signal.ethos` owns the ordinary wire contract. A Cargo build invokes
`ethos-monolith::SignalGeneration` for that source and rejects stale committed
`src/generated/signal.rs` output. Consequently that file is provenance-marked
generated output, not a second handwritten interface.

The `Channel.{Orchestrate 1 4}` declaration generates the source-owned
`OrchestrateWire` binding, request operation, closed reply enum, Dotos codecs,
and `signal-frame` channel declaration. Consumers use the public re-exports
from `signal_orchestrate`; this crate has no daemon policy.

The one concrete ordinary input is a `PathLock` containing a name, absolute
path vector, and description. Its refusal is closed:
`DuplicateActiveName(PathLock)` or `PathOverlap(PathLockOverlap)`. Release is
named by the PathLock name and may only refuse `UnknownActiveName`.
