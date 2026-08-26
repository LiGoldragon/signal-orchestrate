# Upgrades

## 0.17.0 — Orchestrate Lock contract

This is a clean breaking deployment. Update `signal-frame` to 0.4.0 and
`ethos-monolith` to 0.5.3 before updating this crate, then regenerate the
committed signal projection. The old `PathLock`/`Register` contract, its Dotos
text, byte fixtures, and request/reply aliases are removed; no compatibility
decoder exists. Meta remains a separate contract and is not changed here.

The generated Datom surface requires Datom 0.5.0
(`4e13442be314ebfdf7bbd32d095c88a084bde42e`) and its matching Protos 0.8.0
source identity (`3b190f9fc2c2a074ceeb6ababfea89e3dd504996`). `LockId.Integer`
is bare canonical decimal `i64`; `ObserveSelection.[Locks]` projects exactly
as `Observe.Locks`. The old nested `LocksSelection`/`Current` form rejects.
