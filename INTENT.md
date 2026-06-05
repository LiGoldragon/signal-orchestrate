# INTENT — signal-orchestrate

*The ordinary wire vocabulary contract for Persona orchestration. Defines the
typed request/reply/event channel that the `orchestrate` CLI and peers use to
claim, release, hand off, and observe roles, and to submit and query the activity
log. Companion to `ARCHITECTURE.md` and `Cargo.toml`. Maintenance: `primary/skills/repo-intent.md`.*

## Repo-scope only

This file carries only the intent that is FOR this `signal-orchestrate` contract.
Workspace-shape intent stays in the primary workspace `primary/INTENT.md`.
Component daemon intent stays in `orchestrate/INTENT.md`. Owner-only orchestration
policy stays in `owner-signal-orchestrate`.

## Why this repo exists

`signal-orchestrate` is the **ordinary peer-callable wire contract** for the
`orchestrate` daemon. It carries the role-lifecycle relation (claim, release,
handoff, observation) and the activity-log relation (submit, query). It contains
no daemon, actor, database, CLI parser, or transport policy — the daemon lowers
these public operations to component-local commands and Sema effects internally.
Owner-only orchestration policy (lane registry, scheduling, supervision policy)
stays in `owner-signal-orchestrate`; runtime actors, the store, and the lowering
logic live in `orchestrate`.

## The channel shape

The Orchestrate channel carries:

- **Role lifecycle:** `Claim(RoleClaim)`, `Release(RoleRelease)`,
  `Handoff(RoleHandoff)`, `Observe(Observation)`.
- **Activity log:** `Submit(ActivitySubmission)`, `Query(ActivityQuery)`.
- **Observation subscription:** `Watch(ObservationSubscription)` opening an
  `ObservationStream`, `Unwatch(ObservationToken)` closing it.
- **Replies:** `ClaimAcceptance` / `ClaimRejection` (carrying typed
  `ScopeConflict` records), `ReleaseAcknowledgment`, `HandoffAcceptance` /
  `HandoffRejection`, `RoleSnapshot`, `LanesObserved`, `ActivityAcknowledgment`,
  `ActivityList`, `PartialApplied`, and the observation open/close replies.
- **Events:** `Observed(ObservationEvent)` on the `ObservationStream`.

The wire vocabulary is contract-local: the daemon lowers these public operations
into component-local commands; Sema classification happens at observation time,
not on the wire. This contract already carries no public `Assert` / `Retract` /
`Mutate` / `Match` tags — it migrated to contract-local operation roots
(2026-05-19).

## Channels are closed, boundaries are named

- Wire enums are closed. No `Unknown` escape hatch; partial application is a
  typed `PartialApplied` reply, not an open escape.
- Request payloads do not mint activity timestamps, claim sequence, or daemon
  identity — the daemon store supplies them when committing.
- `orchestrate` mints those values at the daemon; request records carry the
  claimed scopes, the role, and the submitted activity body only.
- No stringly-typed dispatch. Scope references, role names, and rejection
  reasons are typed closed enums.

## Wire vocabulary discipline

Per `primary/skills/contract-repo.md` §"Public contracts use contract-local
operation verbs":

- Operation roots are domain verbs in verb form: `Claim`, `Release`, `Handoff`,
  `Observe`, `Submit`, `Query`, `Watch`, `Unwatch` — not Sema class words.
- Reply success variants name the outcome: `Claim` → `ClaimAcceptance`;
  rejections carry typed records (`ClaimRejection` with `ScopeConflict`,
  `HandoffRejection`).
- Payload record names are domain nouns the operation carries (`RoleClaim`,
  `RoleRelease`, `ActivitySubmission`, `ActivityQuery`), not `Request` or
  generic containers.
- The mandatory `Tap`/`Untap` observable surface (macro-injected) is the
  standardized observability hook for this persona component; the domain
  `Watch`/`Unwatch` pair carries the role-observation subscription alongside it.

## Constraints

- This crate carries only typed wire vocabulary, NOTA codecs, and round-trip
  witnesses.
- No runtime code: no actors, no tokio, no socket binding, no redb, no claim or
  scheduling logic.
- Contract types derive NOTA in this crate. Consumers do not carry shadow types.
- Every operation, reply, and event variant round-trips through both rkyv frames
  and NOTA text.
- Activity timestamps and claim sequence are not accepted from callers; the
  daemon store supplies them at commit.
- Owner-only orchestration authority (lane registry, scheduling policy,
  supervision policy) stays in `owner-signal-orchestrate`, not here.

## Three-layer model

Layer 1 (this crate): contract operations on the wire (`Claim`, `Release`,
`Handoff`, `Observe`, `Submit`, `Query`).
Layer 2 (daemon): component-local `OrchestrateCommand` enum that the daemon
executes against its store.
Layer 3 (observation): payloadless Sema class labels for cross-component
introspection; the observer stream exposes inbound operation and outbound effect
events.

The contract names the public action at the boundary; the daemon decides what
internal work and Sema class label each action maps to. Sema classification
never appears on the wire.

## Code map

```text
src/lib.rs                           — RoleClaim/RoleRelease/ActivitySubmission records, NOTA codecs, signal_channel! invocation
schema/signal-orchestrate.concept.schema — concept-schema source for the contract
tests/round_trip.rs                  — rkyv frame and NOTA round-trip witnesses per operation
```

## Non-ownership

This crate does not own:

- `orchestrate` daemon runtime, Kameo actors, or component lifecycle;
- the orchestrate redb store, claim tables, or activity log storage;
- socket binding, transport, or version handshake policy;
- claim conflict resolution, scheduling, lane registry, or supervision logic;
- owner-only orchestration policy (that is `owner-signal-orchestrate`);
- CLI formatting, audit wrapping, or Nexus record composition.

## See also

- `ARCHITECTURE.md` — detailed channel shape, the contract-local-verb migration,
  the mandatory observable surface, and closed-enum discipline.
- `../orchestrate/INTENT.md` — daemon-side intent (schema-driven planes, actors,
  state, policy lowering).
- `../owner-signal-orchestrate/INTENT.md` — owner-only orchestration policy contract.
- `primary/skills/contract-repo.md` — contract repo discipline and naming rules.
- `primary/skills/component-triad.md` — repo triad structure and wire layers.
