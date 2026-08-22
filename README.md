# signal-orchestrate

The ordinary binary Signal contract for atomic Datom path-lock registration.

The sole request is `Register(PathLock)`. It returns either
`PathLockRegistered` or `PathLockRegistrationRejected`. Rejections are typed:
`DuplicateActiveName` identifies the active holder and `PathOverlap` identifies
the overlapping canonical path and its holder.

Clients construct `datom::PathLock` at their text boundary and convert it with
`PathLock::try_from`. That conversion uses the native Datom carrier, so its
nonempty normalized paths and description rules are retained. Signal carries
only the resulting length-prefixed rkyv frame; this crate has no Dotos codec.
