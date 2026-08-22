# signal-orchestrate architecture

`signal-orchestrate` declares one ordinary Signal request/reply contract and no
runtime, storage, text parser, observation stream, or meta surface.

```text
Datom client text
  → datom::PathLock (checked native constructor)
  → signal_orchestrate::PathLock
  → Register(PathLock) ──Signal frame──> orchestrate daemon
                                      ├─ PathLockRegistered
                                      └─ PathLockRegistrationRejected
                                           ├─ DuplicateActiveName { holder: PathLock }
                                           └─ PathOverlap { path: PathLockPath, holder: PathLock }
```

`PathLock` is the binary carrier of exactly the native Datom record's three
concepts. `TryFrom<datom::PathLock>` reads the native validated value through
its view trait; the reverse conversion calls Datom's checked constructor.
`PathLockPath` represents the native normalized conflict-path concept. The
success and rejection reply carriers likewise convert explicitly and losslessly
to and from Datom's `PathLockRegistered`, `PathLockRegistrationRejected`,
`DuplicateActiveName`, and `PathOverlap` values.

Registration is one request and one reply: a daemon either registers the full
lock or returns one typed rejection. The daemon owns active-name and overlap
decisions; this contract carries no sessions, lanes, roles, authority,
recovery, release, observations, filesystem mutation, or Dotos representation.

The contract marker uses wire revision 3. Its crate version is `0.15.0`.

`tests/round_trip.rs` fixes literal canonical native Datom text witnesses for
the request, success reply, duplicate-name refusal, and overlap refusal. It
also asserts binary request and reply frames round-trip and proves the native
constructor rejects invalid path-lock data.
