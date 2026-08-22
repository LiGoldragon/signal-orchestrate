# signal-orchestrate

The ordinary binary Signal contract for atomic Datom path-lock registration.

The sole request is `Register(PathLock)`. It returns either
`PathLockRegistered` or `PathLockRegistrationRejected`. Rejections are typed:
`DuplicateActiveName` carries the full active holder lock and `PathOverlap`
carries the normalized overlapping path and full holder lock.

Clients construct native Datom path-lock requests and replies at their text
boundary, then use the explicit `TryFrom` conversions. Those conversions use
Datom's checked constructors and views, retaining validated names and
descriptions, nonempty normalized paths, and normalized conflicting paths.
Signal carries only the resulting length-prefixed rkyv frame; this crate has no
Dotos codec.
