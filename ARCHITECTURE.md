# signal-orchestrate architecture

`signal-orchestrate` declares one ordinary Signal request/reply contract and no
runtime, storage, text parser, observation stream, or meta surface.

```text
Datom client text
  → datom::PathLock { name, paths, description }
  → signal_orchestrate::PathLock
  → Register(PathLock) ──Signal frame──> orchestrate daemon
                                      ├─ PathLockRegistered
                                      └─ PathLockRegistrationRejected
                                           ├─ DuplicateActiveName { holder }
                                           └─ PathOverlap { path, holder }
```

`PathLock` is the binary carrier of exactly the native Datom record's three
concepts. `TryFrom<datom::PathLock>` crosses the text boundary by running the
native textualize/realize cycle; it therefore preserves Datom's canonical path
normalization and rejects an empty, duplicate, relative, or parent-traversing
path list and a blank or multiline description. `From<PathLock>` returns the
same native carrier.

Registration is one request and one reply: a daemon either registers the full
lock or returns one typed rejection. The daemon owns active-name and overlap
decisions; this contract carries no sessions, lanes, roles, authority,
recovery, release, observations, filesystem mutation, or Dotos representation.

The contract marker uses wire revision 2. Its crate version is `0.14.0`.

`tests/round_trip.rs` fixes a literal Datom `PathLock.{…}` text witness,
crosses it into and out of the binary carrier, and round-trips the Register
request plus both typed rejection cases through length-prefixed Signal frames.
