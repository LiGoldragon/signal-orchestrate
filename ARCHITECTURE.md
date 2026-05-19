# signal-persona-orchestrate — architecture

*The ordinary Signal contract for Persona orchestration: role claims,
claim release/handoff/observation, and activity log requests.*

---

## 0 · TL;DR

`signal-persona-orchestrate` is a contract crate. It owns the typed
wire vocabulary for the ordinary `persona-orchestrate` surface and
contains no daemon, actor, database, CLI parser, or transport policy.

The channel is declared by one `signal_channel!` invocation in
`src/lib.rs`. Public operations use contract-local verb roots. The
daemon lowers those operations to Sema effects internally.

## Migration history — contract-local verbs (2026-05-19)

This contract migrated from `signal-core` public `SignalVerb` wrappers
to `signal-frame` contract-local operation roots.

The public request surface is now:

- `Claim(RoleClaim)`
- `Release(RoleRelease)`
- `Handoff(RoleHandoff)`
- `Observe(RoleObservation)`
- `Submit(ActivitySubmission)`
- `Query(ActivityQuery)`
- `Watch(ObservationSubscription)`
- `Unwatch(ObservationToken)`

There is no public `Assert` / `Retract` / `Mutate` / `Match` tag in
this contract. `persona-orchestrate` owns the lower Sema translation.
The observer stream exposes inbound operation and outbound Sema-effect
events for introspection.

## 1 · Channel

| Side | Component |
|---|---|
| Request producer | `persona-orchestrate` CLI, transitional workspace helpers, or peers speaking the ordinary orchestration surface. |
| Request consumer | `persona-orchestrate` daemon. |
| Wire type | `OrchestrateFrame` / `OrchestrateRequest` / `OrchestrateReply`. |

The channel is request/reply. Activity timestamps are not accepted
from callers; the daemon store supplies them when committing
`ActivitySubmission`.

## 2 · Requests And Replies

```text
OrchestrateRequest                 OrchestrateReply
├─ RoleClaim                       ├─ ClaimAcceptance
├─ RoleRelease                     ├─ ClaimRejection
├─ RoleHandoff                     ├─ ReleaseAcknowledgment
├─ RoleObservation                 ├─ HandoffAcceptance
├─ ActivitySubmission              ├─ HandoffRejection
└─ ActivityQuery                   ├─ RoleSnapshot
                                   ├─ ActivityAcknowledgment
                                   └─ ActivityList
```

Closed enums have no `Unknown` variant. Conflicts and rejections
carry typed records (`ScopeConflict`, `HandoffRejectionReason`) so
callers pattern-match instead of parsing strings.

## 3 · Typed Values

This contract owns:

- `RoleIdentifier` / `RoleName`: validated dynamic role token. The
  `RoleName` name remains as a compatibility alias while callers move
  off the old fixed-role enum shape.
- `HarnessKind`: Codex or Claude, carried as data in role status
  instead of being hidden in the role string.
- `ScopeReference`
- `WirePath`
- `TaskToken`
- `ScopeReason`
- `TimestampNanos`

`RoleIdentifier`, `WirePath`, `TaskToken`, and `ScopeReason` are
validated newtypes. Construct them through `from_wire_token`,
`from_absolute_path`, `from_wire_token`, and `from_text` respectively.
Invalid values are rejected at the contract boundary and also during
NOTA decode.

## 4 · Operation Roots

| Operation | Lower Sema effect |
|---|---|
| `Claim` | `Assert` |
| `Release` | `Retract` |
| `Handoff` | `Mutate` |
| `Observe` | `Match` |
| `Submit` | `Assert` |
| `Query` | `Match` |
| `Watch` | `Subscribe` |
| `Unwatch` | `Retract` |

`OrchestrateRequest::operation_kind()` exposes the contract operation
without asking consumers to know the lower Sema effect.

## 5 · Non-Ownership

This crate does not own:

- daemon actors or request handlers;
- the `persona-orchestrate.redb` database;
- lock-file projections;
- CLI argv parsing or NOTA rendering policy;
- socket paths, reconnect policy, or transport lifecycle;
- owner-only orchestration orders.

Owner-only orders belong in an `owner-signal-persona-orchestrate`
contract. This ordinary contract is the peer/CLI surface.

## 6 · Witness Tests

`tests/round_trip.rs` proves:

- every request variant round-trips through an `OrchestrateFrame`;
- every reply variant round-trips through an `OrchestrateFrame`;
- operation roots encode as contract-local NOTA heads;
- dynamic role identifiers round-trip as ordinary payload data;
- observer events round-trip through the streaming frame shape;
- invalid scope primitives are rejected.

## Code Map

```text
src/lib.rs            payloads, validation newtypes, signal_channel!
tests/round_trip.rs   frame round trips and contract-local operation witnesses
```

## See Also

- `../persona-orchestrate/ARCHITECTURE.md` — runtime consumer and
  state owner.
- `../signal-frame/ARCHITECTURE.md` — Signal frame kernel.
- `../signal-sema/ARCHITECTURE.md` — lower Sema operation vocabulary.
- `~/primary/skills/contract-repo.md` — contract-repo discipline.
- `~/primary/skills/architectural-truth-tests.md` — witness-test
  discipline.
