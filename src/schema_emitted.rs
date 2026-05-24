//! Hand-equivalent module showing what `signal_channel!([schema])`
//! would emit from `orchestrate.schema`.
//!
//! This is the Phase 3 deliverable of the orchestrate port
//! (second-designer/173). It does not replace the live
//! hand-written wire types in `lib.rs`; it documents — in
//! executable Rust — the shape a future macro pass produces.
//!
//! Why a separate module:
//!
//! - The current schema macro MVP
//!   (`signal-frame/macros/src/schema_reader.rs`) supports
//!   **one endpoint per operation root** and **non-unit payloads**
//!   only. Two routes in `orchestrate.schema` violate that today:
//!   `Observe [Roles Lanes]` (two endpoints, both unit) and
//!   `Retire [Role Lane]` (two endpoints, mixed payload).
//! - Rather than dumb the schema down to match the current
//!   macro, we leave the schema rich (source-of-truth shape) and
//!   show the hand-equivalent so the next macro pass has a
//!   concrete target.
//!
//! The intent is to prove the schema is implementable end-to-end
//! and that the macro family — once it grows multi-endpoint +
//! unit-payload support — produces code equivalent to the live
//! hand-written contract in `lib.rs`.
//!
//! See:
//! - `orchestrate.schema` — the source-of-truth schema file
//! - `reports/second-designer/173-orchestrate-port-to-schema-engine-and-no-downtime-upgrade-2026-05-24.md`

use crate::{
    ActivityQuery, ActivitySubmission, LaneAuthority, LaneIdentifier, ObservationSubscription,
    ObservationToken, Role, RoleClaim, RoleHandoff, RoleIdentifier, RoleRelease,
};
use signal_sema::SemaOperation;

// ─── Ordinary operation root — header layer 1 ─────────────
//
// Emitted from the ordinary header:
//
//     [
//       (Claim [Role])
//       (Release [Role])
//       (Handoff [Role])
//       (Observe [Roles Lanes])
//       (Submit [Activity])
//       (Query [Activity])
//       (Watch [Operations])
//       (Unwatch [Token])
//     ]

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Claim(ClaimOperation),
    Release(ReleaseOperation),
    Handoff(HandoffOperation),
    Observe(ObserveOperation),
    Submit(SubmitOperation),
    Query(QueryOperation),
    Watch(WatchOperation),
    Unwatch(UnwatchOperation),
}

impl Operation {
    pub fn root_kind(&self) -> OperationRoot {
        match self {
            Self::Claim(_) => OperationRoot::Claim,
            Self::Release(_) => OperationRoot::Release,
            Self::Handoff(_) => OperationRoot::Handoff,
            Self::Observe(_) => OperationRoot::Observe,
            Self::Submit(_) => OperationRoot::Submit,
            Self::Query(_) => OperationRoot::Query,
            Self::Watch(_) => OperationRoot::Watch,
            Self::Unwatch(_) => OperationRoot::Unwatch,
        }
    }

    /// Lower the operation to its two-byte short header
    /// (root discriminator + endpoint discriminator) per
    /// `reports/designer/326-v13` §1 "uniform header form".
    pub fn short_header(&self) -> ShortHeader {
        let (root_byte, endpoint_byte) = match self {
            Self::Claim(inner) => (0u8, inner.endpoint_discriminator()),
            Self::Release(inner) => (1, inner.endpoint_discriminator()),
            Self::Handoff(inner) => (2, inner.endpoint_discriminator()),
            Self::Observe(inner) => (3, inner.endpoint_discriminator()),
            Self::Submit(inner) => (4, inner.endpoint_discriminator()),
            Self::Query(inner) => (5, inner.endpoint_discriminator()),
            Self::Watch(inner) => (6, inner.endpoint_discriminator()),
            Self::Unwatch(inner) => (7, inner.endpoint_discriminator()),
        };
        ShortHeader::new(root_byte, endpoint_byte)
    }
}

/// Root-byte discriminator for the ordinary header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperationRoot {
    Claim,
    Release,
    Handoff,
    Observe,
    Submit,
    Query,
    Watch,
    Unwatch,
}

// ─── Per-root endpoint enums ───────────────────────────────
//
// Emitted from the namespace body declarations that share the
// header root name. The schema namespace says, for example:
//
//   Claim [(Role RoleClaim)]
//
// The root has one endpoint `Role` whose payload is `RoleClaim`.
// A future multi-endpoint schema (e.g. `Observe [Roles Lanes]`)
// produces an enum with multiple variants — `ObserveOperation::Roles`
// and `ObserveOperation::Lanes`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimOperation {
    Role(RoleClaim),
}

impl ClaimOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Role(_) => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReleaseOperation {
    Role(RoleRelease),
}

impl ReleaseOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Role(_) => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HandoffOperation {
    Role(RoleHandoff),
}

impl HandoffOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Role(_) => 0,
        }
    }
}

/// Multi-endpoint operation root — both endpoints are unit
/// (no payload). The current schema-macro MVP does not yet
/// emit this shape; a future macro pass that adds unit-payload
/// support produces exactly this enum.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ObserveOperation {
    Roles,
    Lanes,
}

impl ObserveOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Roles => 0,
            Self::Lanes => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SubmitOperation {
    Activity(ActivitySubmission),
}

impl SubmitOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Activity(_) => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryOperation {
    Activity(ActivityQuery),
}

impl QueryOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Activity(_) => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WatchOperation {
    Operations(ObservationSubscription),
}

impl WatchOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Operations(_) => 0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UnwatchOperation {
    Token(ObservationToken),
}

impl UnwatchOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Token(_) => 0,
        }
    }
}

// ─── Owner operation root — header layer 2 ─────────────────
//
// Emitted from the owner header:
//
//     [
//       (Create [Role])
//       (Retire [Role Lane])
//       (Refresh [RepositoryIndex])
//       (Register [Lane])
//       (SetAuthority [Lane])
//     ]
//
// Owner operations are inexpressible on the ordinary socket
// (per `skills/component-triad.md` §"Two authority tiers").

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OwnerOperation {
    Create(CreateOperation),
    Retire(RetireOperation),
    Refresh(RefreshOperation),
    Register(RegisterOperation),
    SetAuthority(SetAuthorityOperation),
}

impl OwnerOperation {
    pub fn short_header(&self) -> ShortHeader {
        let (root_byte, endpoint_byte) = match self {
            Self::Create(inner) => (0u8, inner.endpoint_discriminator()),
            Self::Retire(inner) => (1, inner.endpoint_discriminator()),
            Self::Refresh(inner) => (2, inner.endpoint_discriminator()),
            Self::Register(inner) => (3, inner.endpoint_discriminator()),
            Self::SetAuthority(inner) => (4, inner.endpoint_discriminator()),
        };
        ShortHeader::new(root_byte, endpoint_byte)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CreateOperation {
    Role(CreateRoleOrder),
}

impl CreateOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Role(_) => 0,
        }
    }
}

/// Multi-endpoint operation root with mixed payloads. The two
/// variants carry different payload shapes: a typed
/// `RetireRoleOrder` and a bare `LaneIdentifier`. Today the live
/// owner contract exposes this as the `Retirement` nested enum;
/// the schema lifts both into a single multi-endpoint header
/// (per `reports/designer/326-v13` §"uniform header form").
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RetireOperation {
    Role(RetireRoleOrder),
    Lane(LaneIdentifier),
}

impl RetireOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Role(_) => 0,
            Self::Lane(_) => 1,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RefreshOperation {
    RepositoryIndex(RefreshRepositoryIndexOrder),
}

impl RefreshOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::RepositoryIndex(_) => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegisterOperation {
    Lane(LaneRegistrationRequest),
}

impl RegisterOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Lane(_) => 0,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SetAuthorityOperation {
    Lane(LaneAuthorityChange),
}

impl SetAuthorityOperation {
    pub const fn endpoint_discriminator(&self) -> u8 {
        match self {
            Self::Lane(_) => 0,
        }
    }
}

// ─── Owner payload mirrors ─────────────────────────────────
//
// The real `CreateRoleOrder`, `RetireRoleOrder`, and
// `RefreshRepositoryIndexOrder` types live in
// `owner-signal-orchestrate`. We reproduce them as structural
// placeholders here so the dispatch trait + endpoint enums in
// this documentation module compile without a circular
// dependency. The macro-emitted world re-emits these in the
// owner contract crate exactly the same way.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateRoleOrder {
    pub role: RoleIdentifier,
    pub harness: crate::HarnessKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RetireRoleOrder {
    pub role: RoleIdentifier,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RefreshRepositoryIndexOrder;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneRegistrationRequest {
    pub role: Role,
    pub authority: LaneAuthority,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LaneAuthorityChange {
    pub lane: LaneIdentifier,
    pub authority: LaneAuthority,
}

// ─── ShortHeader projection — wire-level byte pair ─────────
//
// The schema emits a constant-time projection from any operation
// to a two-byte `(root, endpoint)` discriminator. The signal
// kernel uses this for routing without needing to decode the
// payload — the byte pair plus the leg (ordinary / owner) is
// enough to find the dispatch arm.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ShortHeader {
    pub root: u8,
    pub endpoint: u8,
}

impl ShortHeader {
    pub const fn new(root: u8, endpoint: u8) -> Self {
        Self { root, endpoint }
    }

    pub const fn as_u16(self) -> u16 {
        ((self.root as u16) << 8) | self.endpoint as u16
    }
}

// ─── Dispatch traits — the daemon's seam ───────────────────
//
// Emitted from the schema as one trait per leg that every daemon
// implements. The trait names every (root, endpoint) pair as a
// typed method; the daemon library writes the body of each
// method. The macro also emits the dispatch fn that reads the
// short header, decodes the payload, and dispatches to the right
// trait method.
//
// Both legs (ordinary + owner) have their own trait so the
// permission boundary is enforced at the type level: an owner-
// only handler cannot be invoked from the ordinary socket
// listener because the function signatures don't match.

pub trait OrdinaryDispatch {
    type Error;
    type Reply;

    fn claim_role(&mut self, payload: RoleClaim) -> Result<Self::Reply, Self::Error>;
    fn release_role(&mut self, payload: RoleRelease) -> Result<Self::Reply, Self::Error>;
    fn handoff_role(&mut self, payload: RoleHandoff) -> Result<Self::Reply, Self::Error>;
    fn observe_roles(&mut self) -> Result<Self::Reply, Self::Error>;
    fn observe_lanes(&mut self) -> Result<Self::Reply, Self::Error>;
    fn submit_activity(&mut self, payload: ActivitySubmission) -> Result<Self::Reply, Self::Error>;
    fn query_activity(&mut self, payload: ActivityQuery) -> Result<Self::Reply, Self::Error>;
    fn watch_operations(
        &mut self,
        payload: ObservationSubscription,
    ) -> Result<Self::Reply, Self::Error>;
    fn unwatch_token(&mut self, payload: ObservationToken) -> Result<Self::Reply, Self::Error>;

    fn dispatch(&mut self, operation: Operation) -> Result<Self::Reply, Self::Error> {
        match operation {
            Operation::Claim(ClaimOperation::Role(payload)) => self.claim_role(payload),
            Operation::Release(ReleaseOperation::Role(payload)) => self.release_role(payload),
            Operation::Handoff(HandoffOperation::Role(payload)) => self.handoff_role(payload),
            Operation::Observe(ObserveOperation::Roles) => self.observe_roles(),
            Operation::Observe(ObserveOperation::Lanes) => self.observe_lanes(),
            Operation::Submit(SubmitOperation::Activity(payload)) => self.submit_activity(payload),
            Operation::Query(QueryOperation::Activity(payload)) => self.query_activity(payload),
            Operation::Watch(WatchOperation::Operations(payload)) => self.watch_operations(payload),
            Operation::Unwatch(UnwatchOperation::Token(payload)) => self.unwatch_token(payload),
        }
    }
}

pub trait OwnerDispatch {
    type Error;
    type Reply;

    fn create_role(&mut self, payload: CreateRoleOrder) -> Result<Self::Reply, Self::Error>;
    fn retire_role(&mut self, payload: RetireRoleOrder) -> Result<Self::Reply, Self::Error>;
    fn retire_lane(&mut self, payload: LaneIdentifier) -> Result<Self::Reply, Self::Error>;
    fn refresh_repository_index(
        &mut self,
        payload: RefreshRepositoryIndexOrder,
    ) -> Result<Self::Reply, Self::Error>;
    fn register_lane(
        &mut self,
        payload: LaneRegistrationRequest,
    ) -> Result<Self::Reply, Self::Error>;
    fn set_lane_authority(
        &mut self,
        payload: LaneAuthorityChange,
    ) -> Result<Self::Reply, Self::Error>;

    fn dispatch(&mut self, operation: OwnerOperation) -> Result<Self::Reply, Self::Error> {
        match operation {
            OwnerOperation::Create(CreateOperation::Role(payload)) => self.create_role(payload),
            OwnerOperation::Retire(RetireOperation::Role(payload)) => self.retire_role(payload),
            OwnerOperation::Retire(RetireOperation::Lane(payload)) => self.retire_lane(payload),
            OwnerOperation::Refresh(RefreshOperation::RepositoryIndex(payload)) => {
                self.refresh_repository_index(payload)
            }
            OwnerOperation::Register(RegisterOperation::Lane(payload)) => {
                self.register_lane(payload)
            }
            OwnerOperation::SetAuthority(SetAuthorityOperation::Lane(payload)) => {
                self.set_lane_authority(payload)
            }
        }
    }
}

// ─── ToSemaOperation projection (Layer 2 → Layer 3) ────────
//
// Required by the verb-spine rule
// (`skills/component-triad.md` §"Verbs come in three layers").
// The schema's namespace `OperationKind` enum is the Layer 1
// label; the daemon's Component Commands carry the Layer 2
// executable shape; this projection lowers them to Layer 3
// payloadless `SemaOperation` classification for cross-component
// observation.

pub fn project_to_sema(operation: &Operation) -> SemaOperation {
    match operation {
        Operation::Claim(_) => SemaOperation::Assert,
        Operation::Release(_) => SemaOperation::Retract,
        Operation::Handoff(_) => SemaOperation::Mutate,
        Operation::Observe(_) => SemaOperation::Match,
        Operation::Submit(_) => SemaOperation::Assert,
        Operation::Query(_) => SemaOperation::Match,
        Operation::Watch(_) => SemaOperation::Subscribe,
        Operation::Unwatch(_) => SemaOperation::Retract,
    }
}

pub fn project_owner_to_sema(operation: &OwnerOperation) -> SemaOperation {
    match operation {
        OwnerOperation::Create(_) => SemaOperation::Assert,
        OwnerOperation::Retire(_) => SemaOperation::Retract,
        OwnerOperation::Refresh(_) => SemaOperation::Mutate,
        OwnerOperation::Register(_) => SemaOperation::Assert,
        OwnerOperation::SetAuthority(_) => SemaOperation::Mutate,
    }
}

// ─── Storage descriptors — emitted for redb table layout ───
//
// Daemon-side: the schema also emits storage descriptors so the
// `<component>.redb` opened through `sema-engine` lays out tables
// matching the wire types. Each declaration whose schema engine
// is `Assert` or `Mutate` produces a typed sema table; storage
// fields outside the wire types are added by the daemon
// (timestamps, slot identifiers, indexes).
//
// The orchestrate runtime's existing `src/tables.rs` is the live
// hand-written version. A future macro pass emits the equivalent;
// the seam preserved is that `OperationLowering` (per
// `orchestrate/ARCHITECTURE.md` §"Migration history") remains the
// contract-to-Component-Command translation point.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SemaTable {
    Roles,
    Claims,
    Activities,
    Repositories,
    LaneRegistry,
    Divergences,
}

impl SemaTable {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Roles => "roles",
            Self::Claims => "claims",
            Self::Activities => "activities",
            Self::Repositories => "repositories",
            Self::LaneRegistry => "lane_registry",
            Self::Divergences => "divergences",
        }
    }
}

// ─── Smoke test — short-header dispatch round-trip ─────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ScopeReason;

    #[test]
    fn short_headers_assign_consecutive_root_slots() {
        let claim = Operation::Claim(ClaimOperation::Role(RoleClaim {
            role: RoleIdentifier::from_wire_token("designer").unwrap(),
            scopes: Vec::new(),
            reason: ScopeReason::from_text("port pilot").unwrap(),
        }));
        assert_eq!(claim.short_header(), ShortHeader::new(0, 0));

        let observe_lanes = Operation::Observe(ObserveOperation::Lanes);
        assert_eq!(observe_lanes.short_header(), ShortHeader::new(3, 1));

        let unwatch = Operation::Unwatch(UnwatchOperation::Token(ObservationToken::new(7)));
        assert_eq!(unwatch.short_header(), ShortHeader::new(7, 0));

        let retire_lane = OwnerOperation::Retire(RetireOperation::Lane(
            LaneIdentifier::from_wire_token("operator").unwrap(),
        ));
        assert_eq!(retire_lane.short_header(), ShortHeader::new(1, 1));
    }

    #[test]
    fn sema_projection_classifies_each_operation_root() {
        let claim = Operation::Claim(ClaimOperation::Role(RoleClaim {
            role: RoleIdentifier::from_wire_token("designer").unwrap(),
            scopes: Vec::new(),
            reason: ScopeReason::from_text("port pilot").unwrap(),
        }));
        assert_eq!(project_to_sema(&claim), SemaOperation::Assert);

        let watch = Operation::Watch(WatchOperation::Operations(ObservationSubscription {
            include_operations: true,
            include_sema_effects: true,
        }));
        assert_eq!(project_to_sema(&watch), SemaOperation::Subscribe);

        let retire_lane = OwnerOperation::Retire(RetireOperation::Lane(
            LaneIdentifier::from_wire_token("operator").unwrap(),
        ));
        assert_eq!(project_owner_to_sema(&retire_lane), SemaOperation::Retract);
    }

    #[test]
    fn dispatch_trait_routes_each_endpoint_to_its_method() {
        struct CountingDispatch {
            counts: [u32; 9],
        }
        impl OrdinaryDispatch for CountingDispatch {
            type Error = ();
            type Reply = u32;

            fn claim_role(&mut self, _payload: RoleClaim) -> Result<Self::Reply, Self::Error> {
                self.counts[0] += 1;
                Ok(self.counts[0])
            }
            fn release_role(&mut self, _payload: RoleRelease) -> Result<Self::Reply, Self::Error> {
                self.counts[1] += 1;
                Ok(self.counts[1])
            }
            fn handoff_role(&mut self, _payload: RoleHandoff) -> Result<Self::Reply, Self::Error> {
                self.counts[2] += 1;
                Ok(self.counts[2])
            }
            fn observe_roles(&mut self) -> Result<Self::Reply, Self::Error> {
                self.counts[3] += 1;
                Ok(self.counts[3])
            }
            fn observe_lanes(&mut self) -> Result<Self::Reply, Self::Error> {
                self.counts[4] += 1;
                Ok(self.counts[4])
            }
            fn submit_activity(
                &mut self,
                _payload: ActivitySubmission,
            ) -> Result<Self::Reply, Self::Error> {
                self.counts[5] += 1;
                Ok(self.counts[5])
            }
            fn query_activity(
                &mut self,
                _payload: ActivityQuery,
            ) -> Result<Self::Reply, Self::Error> {
                self.counts[6] += 1;
                Ok(self.counts[6])
            }
            fn watch_operations(
                &mut self,
                _payload: ObservationSubscription,
            ) -> Result<Self::Reply, Self::Error> {
                self.counts[7] += 1;
                Ok(self.counts[7])
            }
            fn unwatch_token(
                &mut self,
                _payload: ObservationToken,
            ) -> Result<Self::Reply, Self::Error> {
                self.counts[8] += 1;
                Ok(self.counts[8])
            }
        }

        let mut handler = CountingDispatch { counts: [0; 9] };

        handler
            .dispatch(Operation::Observe(ObserveOperation::Roles))
            .unwrap();
        handler
            .dispatch(Operation::Observe(ObserveOperation::Lanes))
            .unwrap();
        handler
            .dispatch(Operation::Observe(ObserveOperation::Lanes))
            .unwrap();
        handler
            .dispatch(Operation::Unwatch(UnwatchOperation::Token(
                ObservationToken::new(1),
            )))
            .unwrap();

        assert_eq!(handler.counts[3], 1, "Observe Roles dispatched once");
        assert_eq!(handler.counts[4], 2, "Observe Lanes dispatched twice");
        assert_eq!(handler.counts[8], 1, "Unwatch Token dispatched once");
    }
}
