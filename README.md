# signal-orchestrate

The ordinary Signal wire contract for Orchestrate Locks. Its source of truth
is `ethos/signal.ethos`; `src/generated/signal.rs` is the ethos-zero
projection of that file, held byte-identical by `build.rs`.

- `Query::{Configure(OrchestrateNexusConfiguration), Lock(LockRequest),
  Release(LockId), Observe(ObserveSelection)}`
- `Response::{ConfigurationAccepted(ConfigurationReceipt),
  ConfigurationRefused(ConfigurationRejection), Locked(Lock), Released(Lock),
  Observed(Observation), LockRejected(LockRejection),
  ReleaseRejected(ReleaseRejection)}`

A Lock snapshot always carries `LockId`, `LockName`, `FlowId`, its vector of
`LockPath`, and `LockReason`. `Observe.Locks` is the canonical selection text
and an empty observation is exactly `Observed.Locks.[]`.

`impl signal::Contracted for Query` gives the contract its wire identity: the
digest of `ETHOS`. Peers greet with it once per connection and refuse a
mismatch rather than negotiate. Framing and the `Dispatch`/`Delivery`
envelope are `signal`'s; this crate owns neither, nor the Nexus, its sockets,
its persistence, or any CLI.

Every generated root is a typed Datom root behind the `datom` feature.
