# signal-persona-orchestrate — architecture

*The ordinary Signal contract for Persona orchestration: role claims,
claim release/handoff/observation, and activity log requests.*

---

## 0 · TL;DR

`signal-persona-orchestrate` is a contract crate. It owns the typed
wire vocabulary for the ordinary `persona-orchestrate` surface and
contains no daemon, actor, database, CLI parser, or transport policy.

The channel is declared by one `signal_channel!` invocation in
`src/lib.rs`. Each request variant declares its `SignalVerb` in the
contract, so consumers do not infer verbs by string matching.

## MUST IMPLEMENT — signal architecture migration

This contract is migrating to contract-local verbs per
`primary/reports/designer/238-signal-architecture-redirection-contract-local-verbs.md`
and `primary/reports/designer/239-signal-architecture-migration-plan.md`.

Drop the SignalVerb prefixes on every request variant. The
canonical contract-local verbs for orchestration are `Claim`
(payload `RoleClaim` becomes the noun the verb acts on), `Release`
(for `RoleRelease`), `Handoff` (for `RoleHandoff`), `Observe`
(for `RoleObservation`, payload names the query target), `Submit`
(for `ActivitySubmission`, payload `Activity`), and `Query`
(for `ActivityQuery`, payload names the filter shape). `Mutate`
on `RoleHandoff` is grammatically wrong — `Handoff` IS the verb;
the move below daemon. Move the verb-to-Sema lowering
(`Claim` → `Assert`, `Release` → `Retract`, `Handoff` → `Mutate`,
`Observe` / `Query` → `Match`, `Submit` → `Assert`) into the
runtime executor.

References: `primary/reports/designer/238-signal-architecture-redirection-contract-local-verbs.md`,
`primary/reports/designer/239-signal-architecture-migration-plan.md`.

**Note to remover:** when the refactor lands, remove this section and
add a `## Migration history — contract-local verbs (2026-05-XX)`
paragraph noting the shape change.

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

## 4 · Verb Map

| Request | Verb |
|---|---|
| `RoleClaim` | `Assert` |
| `RoleRelease` | `Retract` |
| `RoleHandoff` | `Mutate` |
| `RoleObservation` | `Match` |
| `ActivitySubmission` | `Assert` |
| `ActivityQuery` | `Match` |

`OrchestrateRequest::operation_kind()` exposes the domain operation
without asking consumers to match on verb roots.

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
- every request variant maps to its declared `SignalVerb`;
- dynamic role identifiers round-trip as ordinary payload data;
- invalid scope primitives are rejected.

## Code Map

```text
src/lib.rs            payloads, validation newtypes, signal_channel!
tests/round_trip.rs   frame round trips and verb witnesses
```

## See Also

- `../persona-orchestrate/ARCHITECTURE.md` — runtime consumer and
  state owner.
- `../signal-core/ARCHITECTURE.md` — Signal frame kernel.
- `~/primary/skills/contract-repo.md` — contract-repo discipline.
- `~/primary/skills/architectural-truth-tests.md` — witness-test
  discipline.
