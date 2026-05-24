# signal-orchestrate — architecture

*The ordinary Signal contract for Persona orchestration: role claims,
claim release/handoff/observation, and activity log requests.*

---

## 0 · TL;DR

`signal-orchestrate` is a contract crate. It owns the typed
wire vocabulary for the ordinary `orchestrate` surface and
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
this contract. `orchestrate` owns its typed Component
Commands (Layer 2) and projects them to payloadless Sema class labels
(Layer 3) for observation. The observer stream exposes inbound
operation and outbound effect events for introspection.

## MUST IMPLEMENT — Tap/Untap mandatory observable surface

Per the three-layer model affirmed 2026-05-20 (psyche 2026-05-20T02:00Z;
spec `primary/reports/designer/246-v4-bundled-fix-deep-design-with-examples.md`):
persona components have a *mandatory* `Tap`/`Untap` observable
surface — the macro injects `Tap(ObserverFilter)` /
`Untap(<Channel>ObserverSubscriptionToken)` verbs uniformly across
every persona daemon. The existing domain-specific `Watch`/`Unwatch`
pair (for role observation) is a separate subscription;
`persona-introspect` reaches the standardized observability through
`Tap`/`Untap`, while `Watch`/`Unwatch` carries the domain-shaped role
observation.

If the domain `Watch`/`Unwatch` and the mandatory observability
collide on naming, the macro-injected `Tap`/`Untap` wins and the
domain verb stays as `Watch`/`Unwatch` (no collision today). Add the
`observable { filter default; operation_event OperationReceived;
effect_event EffectEmitted; }` block when the macro grammar lands per
`/246-v4`; `EffectEmitted` carries the lowered `SemaObservation`.

## 1 · Channel

| Side | Component |
|---|---|
| Request producer | `orchestrate` CLI, transitional workspace helpers, or peers speaking the ordinary orchestration surface. |
| Request consumer | `orchestrate` daemon. |
| Wire type | `OrchestrateFrame` / `OrchestrateRequest` / `OrchestrateReply`. |

The channel is request/reply for ordinary operations and stream-capable
for observation. Activity timestamps are not accepted from callers; the
daemon store supplies them when committing `ActivitySubmission`.

## 2 · Requests And Replies

```text
OrchestrateRequest                 OrchestrateReply
├─ Claim(RoleClaim)                ├─ ClaimAcceptance
├─ Release(RoleRelease)            ├─ ClaimRejection
├─ Handoff(RoleHandoff)            ├─ ReleaseAcknowledgment
├─ Observe(RoleObservation)        ├─ HandoffAcceptance
├─ Submit(ActivitySubmission)      ├─ HandoffRejection
├─ Query(ActivityQuery)            ├─ RoleSnapshot
├─ Watch(ObservationSubscription)  ├─ ActivityAcknowledgment
└─ Unwatch(ObservationToken)       ├─ ActivityList
                                   ├─ PartialApplied
                                   ├─ ObservationOpened
                                   └─ ObservationClosed
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
- `PartialApplied`, `ApplicationSuccess`, `ApplicationFailure`,
  `DownstreamComponent`, and `ApplicationFailureReason`: typed
  record-divergence reply vocabulary for fanned-out Mutate chains.

`RoleIdentifier`, `WirePath`, `TaskToken`, and `ScopeReason` are
validated newtypes. Construct them through `from_wire_token`,
`from_absolute_path`, `from_wire_token`, and `from_text` respectively.
Invalid values are rejected at the contract boundary and also during
NOTA decode.

## 4 · Sema-class projections (Layer 3)

Each contract-local operation's daemon-side Component Command
projects to a payloadless Sema class label for observation. The wire
form carries the contract-local verb only; the table below is the
*expected daemon-side classification*:

| Operation | Projected Sema class |
|---|---|
| `Claim` | `Assert` |
| `Release` | `Retract` |
| `Handoff` | `Mutate` |
| `Observe` | `Match` |
| `Submit` | `Assert` |
| `Query` | `Match` |
| `Watch` | `Subscribe` |
| `Unwatch` | `Retract` |
| `Tap` (mandatory) | `Subscribe` |
| `Untap` (mandatory) | `Retract` |

`OrchestrateRequest::operation_kind()` exposes the contract operation
without asking consumers to know the Sema class.

## 5 · Non-Ownership

This crate does not own:

- daemon actors or request handlers;
- the `orchestrate.redb` database;
- lock-file projections;
- CLI argv parsing or NOTA rendering policy;
- socket paths, reconnect policy, or transport lifecycle;
- owner-only orchestration orders.

Owner-only orders belong in an `owner-signal-orchestrate`
contract. This ordinary contract is the peer/CLI surface.

## 6 · Witness Tests

`tests/round_trip.rs` proves:

- every request variant round-trips through an `OrchestrateFrame`;
- every reply variant round-trips through an `OrchestrateFrame`;
- `PartialApplied` carries successful and failed downstream legs as
  typed data, not strings;
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

- `../orchestrate/ARCHITECTURE.md` — runtime consumer and
  state owner.
- `../signal-frame/ARCHITECTURE.md` — Signal frame kernel.
- `../signal-sema/ARCHITECTURE.md` — payloadless Sema classification
  vocabulary used at the observation layer.
- `~/primary/skills/contract-repo.md` — contract-repo discipline.
- `~/primary/skills/component-triad.md` §"Verbs come in three layers".
- `~/primary/skills/architectural-truth-tests.md` — witness-test
  discipline.
