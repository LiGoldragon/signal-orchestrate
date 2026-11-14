# signal-orchestrate

The ordinary Signal wire contract for Orchestrate Locks. Its source of truth
is `ethos/signal.ethos`; `src/generated/signal.rs` is the ethos-zero
projection of that file, held byte-identical by `build.rs`.

- `Input::{Configure(OrchestrateNexusConfiguration), Lock(LockRequest),
  Release(LockId), Observe(ObserveSelection)}`
- `Output::{ConfigurationAccepted(ConfigurationReceipt), Locked(HeldLock),
  Released(HeldLock), Observed(Observation),
  ConfigurationRefused(ConfigurationRejection), LockRejected(LockRejection),
  ReleaseRejected(ReleaseRejection)}`: the n-th output answers the n-th input,
  and the three past the last input are refusals.
- `Answering`, one capability per input, and `impl Answerable for Input`, the
  dispatch a Nexus answers through: `input.answer(&nexus)`.

A `HeldLock`, the Lock the Nexus holds, always carries `LockId`, `LockName`, `FlowId`, its vector of
`LockPath`, and `LockReason`. `Observe.Locks` is the canonical selection text
and an empty observation is exactly `Observed.Locks.[]`.

`impl signal::Contracted for Input` gives the contract its wire identity: the
digest of `ETHOS`. Peers greet with it once per connection and refuse a
mismatch rather than negotiate. Framing and the `Dispatch`/`Delivery`
envelope are `signal`'s; this crate owns neither, nor the Nexus, its sockets,
its persistence, or any CLI.

Every generated root is a typed Datom root behind the `datom` feature.
