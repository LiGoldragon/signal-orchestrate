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

## See also

- `ARCHITECTURE.md` — channel role + boundaries
- `~/primary/skills/contract-repo.md` — contract-repo
  discipline
- `signal-core` — kernel that supplies `Frame`,
  `Request`, `Reply`, `signal_channel!`
- `orchestrate` — the consumer that implements
  this contract
