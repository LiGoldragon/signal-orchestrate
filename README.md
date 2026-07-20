# signal-orchestrate

The ordinary Signal contract for **`orchestrate`**:
role claim/release/handoff/observation plus activity submission
and query.

Read `src/lib.rs` for the public interface — two enums
(`OrchestrateRequest`, `OrchestrateReply`) declared via the
`signal_channel!` macro. The variants ARE the messages this
channel carries:

- **Role lifecycle:** `RoleClaim`, `RoleRelease`,
  `RoleHandoff`, `RoleObservation`.
- **Activity log:** `ActivitySubmission`, `ActivityQuery`.

## Quick reference

```rust
use signal_orchestrate::{
    OrchestrateRequest, RoleClaim, RoleName, ScopeReason, ScopeReference, WirePath,
};

// Designer claims a path and a task scope
let request = OrchestrateRequest::RoleClaim(RoleClaim {
    role: RoleName::from_wire_token("design-system-refresh")?,
    scopes: vec![
        ScopeReference::Path(
            WirePath::from_absolute_path("/git/.../signal/ARCHITECTURE.md")?
        ),
    ],
    reason: ScopeReason::from_text("rescope per /91 §3.1")?,
});
// Hand the request to orchestrate's daemon over OrchestrateFrame.
```

The state actor replies with `OrchestrateReply::ClaimAcceptance`
on success or `OrchestrateReply::ClaimRejection` (carrying
typed `ScopeConflict` records) on overlap.

## v0.9 compatibility family

Version 0.9.2 is a maintained compatibility release for current
Criome-compatible Orchestrate maintenance. It pins the legacy Nota, Signal
Frame, Signal Criome, and schema-rust family to immutable revisions. Its
checked-in schema artifacts are intentionally unchanged from v0.9.1 and are
fresh only with the reply-reliability schema-rust 0.7.1 generator. In
particular, this preserves the typed `EngineRefusal` wire reply surface; a
0.7.0 generator would remove that surface from generated artifacts.

## See also

- `ARCHITECTURE.md` — channel role + boundaries
- `~/primary/skills/contract-repo.md` — contract-repo
  discipline
- `signal-frame` — kernel that supplies `Frame`,
  `Request`, `Reply`, `signal_channel!`
- `orchestrate` — the consumer that implements
  this contract
