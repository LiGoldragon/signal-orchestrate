# signal-orchestrate — architecture

*The ordinary Signal contract for Persona orchestration: role claims,
claim release/handoff/observation, and activity log requests.*

## 0 · TL;DR

`signal-orchestrate` is a contract crate. It owns the typed
wire vocabulary for the ordinary `orchestrate` surface and
contains no daemon, actor, database, CLI parser, or transport policy.

The channel is declared by one `signal_channel!` invocation in
`src/lib.rs`. Public operations use contract-local verb roots. The
daemon lowers those operations to internal commands and publishes
contract-owned effect observations.

## Contract Operation Heads

The public request surface is:

- `Claim(RoleClaim)`
- `Release(RoleRelease)`
- `Handoff(RoleHandoff)`
- `Observe(RoleObservation)`
- `Submit(ActivitySubmission)`
- `Query(ActivityQuery)`
- `Watch(ObservationSubscription)`
- `Unwatch(ObservationToken)`

There is no public `Assert`, `Mutate`, `Retract`, `Match`, `Subscribe`, or
`Validate` operation root in this contract. `orchestrate` owns its typed
component commands, Nexus decisions, and SEMA reads or writes inside the
daemon. The observer stream exposes inbound operation and outbound
contract-owned effect events for introspection.

## Tap/Untap Observable Surface

Persona components have a mandatory `Tap`/`Untap` observable
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
`/246-v4`; `EffectEmitted` carries the affected operation and a
contract-owned `EffectOutcome`.

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

## 4 · Daemon Lowering Boundary

Each contract-local operation lowers inside `orchestrate` into a daemon-owned
command and any SEMA reads or writes needed to answer it. The wire form carries
only the contract-local verb. `EffectEmitted` carries the contract-owned
`operation` and `outcome` pair; it never carries `SemaObservation`, and this
crate has no `signal-sema` dependency.

`OrchestrateRequest::operation_kind()` exposes the contract operation without
asking consumers to know any daemon storage plan.

## 5 · Non-Ownership

This crate does not own:

- daemon actors or request handlers;
- the `orchestrate.redb` database;
- lock-file projections;
- CLI argv parsing or NOTA rendering policy;
- socket paths, reconnect policy, or transport lifecycle;
- meta orchestration orders.

Meta orders belong in a `meta-signal-orchestrate` contract. This ordinary
contract is the peer/CLI surface.

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
- `~/primary/skills/contract-repo.md` — contract-repo discipline.
- `~/primary/skills/component-triad.md`.
- `~/primary/skills/architectural-truth-tests.md` — witness-test
  discipline.
