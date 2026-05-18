//! Signal contract — `orchestrate` CLI ↔ `persona-orchestrate`.
//!
//! Read this file as the public interface of the workspace
//! orchestration channel. The channel carries:
//!
//! - **Role claim/release/handoff** — the claim-flow today
//!   served by `tools/orchestrate` (a bash helper); migrating
//!   into `persona-orchestrate` per
//!   `~/primary/reports/designer/93-persona-orchestrate-rust-rewrite-and-activity-log.md`.
//! - **Role observation** — read the active claims for every
//!   role plus the most recent activity entries.
//! - **Activity submission** — append a typed activity record:
//!   who (role), what (path or task token), why (short reason).
//!   Time is store-stamped, never agent-supplied (per
//!   `~/primary/ESSENCE.md` §"Infrastructure mints identity,
//!   time, and sender").
//! - **Activity query** — read recent activity records,
//!   optionally filtered by role or scope.
//!
//! The channel is **request/reply** (every operation has a
//! typed reply). Subscription mode is a future extension —
//! see designer/93 §7.5.
//!
//! See `ARCHITECTURE.md` for the channel's role and
//! boundaries; `~/primary/skills/contract-repo.md` for the
//! contract-repo discipline this crate follows.

use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode, NotaEnum, NotaRecord, NotaTransparent};
use rkyv::{Archive, Deserialize as RkyvDeserialize, Serialize as RkyvSerialize};
use signal_core::signal_channel;

// ─── Identity ─────────────────────────────────────────────

/// The closed set of workspace roles. Adding a role is a
/// coordinated schema change — every consumer of this
/// contract recompiles together.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    NotaEnum,
)]
pub enum RoleName {
    Operator,
    OperatorAssistant,
    Designer,
    DesignerAssistant,
    SystemSpecialist,
    SystemAssistant,
    Poet,
    PoetAssistant,
}

// ─── Scope reference ──────────────────────────────────────

/// What's being claimed / observed / acted on.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq, Hash)]
pub enum ScopeReference {
    /// An absolute file or directory path.
    Path(WirePath),
    /// A bracketed task token like `[primary-f99]` (stored
    /// without brackets here).
    Task(TaskToken),
}

impl NotaEncode for ScopeReference {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::Path(path) => {
                encoder.start_record("Path")?;
                path.encode(encoder)?;
                encoder.end_record()
            }
            Self::Task(task) => {
                encoder.start_record("Task")?;
                task.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for ScopeReference {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "Path" => {
                decoder.expect_record_head("Path")?;
                let path = WirePath::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::Path(path))
            }
            "Task" => {
                decoder.expect_record_head("Task")?;
                let task = TaskToken::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::Task(task))
            }
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "ScopeReference",
                got: other.to_string(),
            }),
        }
    }
}

/// Absolute path, newtyped for cross-platform stability on
/// the wire (per `~/primary/skills/rust-discipline.md`
/// §"Newtype the wire form" — `PathBuf` archives
/// non-deterministically).
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct WirePath(String);

impl WirePath {
    pub fn new(path: impl Into<String>) -> Self {
        Self(path.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for WirePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// A bracketed task identifier (stored without brackets).
/// Bracketed form like `[primary-f99]` is the human surface;
/// the wire carries the raw token.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct TaskToken(String);

impl TaskToken {
    pub fn new(token: impl Into<String>) -> Self {
        Self(token.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for TaskToken {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ─── Reason ───────────────────────────────────────────────

/// A short reason string. Provisional per
/// `~/primary/reports/designer/92-sema-as-database-library-architecture-revamp.md`
/// §4 — strings allowed here until the typed Nexus record
/// shape for "intent" is named.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaTransparent, Debug, Clone, PartialEq, Eq, Hash,
)]
pub struct ScopeReason(String);

impl ScopeReason {
    pub fn new(reason: impl Into<String>) -> Self {
        Self(reason.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl AsRef<str> for ScopeReason {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

// ─── Time ─────────────────────────────────────────────────

/// Nanoseconds since the UNIX epoch. Store-supplied at
/// commit time; never agent-supplied.
#[derive(
    Archive,
    RkyvSerialize,
    RkyvDeserialize,
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    Hash,
    PartialOrd,
    Ord,
    NotaTransparent,
)]
pub struct TimestampNanos(u64);

impl TimestampNanos {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn value(self) -> u64 {
        self.0
    }
}

// ─── Claim verbs ──────────────────────────────────────────

/// A role asks to claim one or more scopes with a short
/// reason. Reply: `ClaimAcceptance` on success, `ClaimRejection`
/// listing every conflict on failure.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleClaim {
    pub role: RoleName,
    pub scopes: Vec<ScopeReference>,
    pub reason: ScopeReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimAcceptance {
    pub role: RoleName,
    pub scopes: Vec<ScopeReference>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimRejection {
    pub role: RoleName,
    pub conflicts: Vec<ScopeConflict>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ScopeConflict {
    pub scope: ScopeReference,
    pub held_by: RoleName,
    pub held_reason: ScopeReason,
}

// ─── Release verbs ────────────────────────────────────────

/// A role releases all of its currently-held scopes.
/// Reply: `ReleaseAcknowledgment` listing what was released.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct RoleRelease {
    pub role: RoleName,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ReleaseAcknowledgment {
    pub role: RoleName,
    pub released_scopes: Vec<ScopeReference>,
}

// ─── Handoff verbs ────────────────────────────────────────

/// One role hands a set of scopes to another role atomically.
/// Reply: `HandoffAcceptance` on success, `HandoffRejection`
/// with a typed reason on failure.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleHandoff {
    pub from: RoleName,
    pub to: RoleName,
    pub scopes: Vec<ScopeReference>,
    pub reason: ScopeReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct HandoffAcceptance {
    pub from: RoleName,
    pub to: RoleName,
    pub scopes: Vec<ScopeReference>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct HandoffRejection {
    pub from: RoleName,
    pub to: RoleName,
    pub reason: HandoffRejectionReason,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum HandoffRejectionReason {
    /// The `from` role doesn't currently hold the named scopes.
    SourceRoleDoesNotHold,
    /// The `to` role's existing claims conflict with the
    /// scopes being handed off (the conflict list names which
    /// scopes and which existing holders).
    TargetRoleConflict(Vec<ScopeConflict>),
}

impl NotaEncode for HandoffRejectionReason {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::SourceRoleDoesNotHold => {
                encoder.start_record("SourceRoleDoesNotHold")?;
                encoder.end_record()
            }
            Self::TargetRoleConflict(conflicts) => {
                encoder.start_record("TargetRoleConflict")?;
                conflicts.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for HandoffRejectionReason {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "SourceRoleDoesNotHold" => {
                decoder.expect_record_head("SourceRoleDoesNotHold")?;
                decoder.expect_record_end()?;
                Ok(Self::SourceRoleDoesNotHold)
            }
            "TargetRoleConflict" => {
                decoder.expect_record_head("TargetRoleConflict")?;
                let conflicts = Vec::<ScopeConflict>::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::TargetRoleConflict(conflicts))
            }
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "HandoffRejectionReason",
                got: other.to_string(),
            }),
        }
    }
}

// ─── Observation ──────────────────────────────────────────

/// Request a snapshot of every role's active claims plus the
/// most recent activity entries. Reply: `RoleSnapshot`.
#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct RoleObservation;

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleSnapshot {
    pub roles: Vec<RoleStatus>,
    pub recent_activity: Vec<Activity>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct RoleStatus {
    pub role: RoleName,
    pub claims: Vec<ClaimEntry>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ClaimEntry {
    pub scope: ScopeReference,
    pub reason: ScopeReason,
}

// ─── Activity log ─────────────────────────────────────────

/// One activity record: who touched what and why. Time is
/// store-supplied (per ESSENCE infrastructure-mints rule —
/// the agent never invents timestamps).
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct Activity {
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
    pub stamped_at: TimestampNanos,
}

/// Submit a new activity record. The store assigns
/// `stamped_at` on commit. Reply: `ActivityAcknowledgment`
/// carrying the slot the record landed in.
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivitySubmission {
    pub role: RoleName,
    pub scope: ScopeReference,
    pub reason: ScopeReason,
}

#[derive(
    Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, Copy, PartialEq, Eq,
)]
pub struct ActivityAcknowledgment {
    /// The slot (sequential u64) the record was assigned.
    pub slot: u64,
}

/// Query the activity log. Limit caps how many records come
/// back; filters narrow by role or scope. Empty filter list
/// = "all".
#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivityQuery {
    pub limit: u32,
    pub filters: Vec<ActivityFilter>,
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, Debug, Clone, PartialEq, Eq)]
pub enum ActivityFilter {
    /// Only entries from this role.
    RoleFilter(RoleName),
    /// Only entries whose scope is `Path(p)` where `p`
    /// starts with this prefix.
    PathPrefix(WirePath),
    /// Only entries whose scope is the exact-match
    /// `Task(token)`.
    TaskToken(TaskToken),
}

impl NotaEncode for ActivityFilter {
    fn encode(&self, encoder: &mut Encoder) -> nota_codec::Result<()> {
        match self {
            Self::RoleFilter(role) => {
                encoder.start_record("RoleFilter")?;
                role.encode(encoder)?;
                encoder.end_record()
            }
            Self::PathPrefix(path) => {
                encoder.start_record("PathPrefix")?;
                path.encode(encoder)?;
                encoder.end_record()
            }
            Self::TaskToken(token) => {
                encoder.start_record("TaskToken")?;
                token.encode(encoder)?;
                encoder.end_record()
            }
        }
    }
}

impl NotaDecode for ActivityFilter {
    fn decode(decoder: &mut Decoder<'_>) -> nota_codec::Result<Self> {
        let head = decoder.peek_record_head()?;
        match head.as_str() {
            "RoleFilter" => {
                decoder.expect_record_head("RoleFilter")?;
                let role = RoleName::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::RoleFilter(role))
            }
            "PathPrefix" => {
                decoder.expect_record_head("PathPrefix")?;
                let path = WirePath::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::PathPrefix(path))
            }
            "TaskToken" => {
                decoder.expect_record_head("TaskToken")?;
                let token = TaskToken::decode(decoder)?;
                decoder.expect_record_end()?;
                Ok(Self::TaskToken(token))
            }
            other => Err(nota_codec::Error::UnknownKindForVerb {
                verb: "ActivityFilter",
                got: other.to_string(),
            }),
        }
    }
}

#[derive(Archive, RkyvSerialize, RkyvDeserialize, NotaRecord, Debug, Clone, PartialEq, Eq)]
pub struct ActivityList {
    /// Ordered most-recent first.
    pub records: Vec<Activity>,
}

// ─── Channel declaration ──────────────────────────────────

signal_channel! {
    channel Orchestrate {
        request OrchestrateRequest {
            Assert RoleClaim(RoleClaim),
            Retract RoleRelease(RoleRelease),
            Mutate RoleHandoff(RoleHandoff),
            Match RoleObservation(RoleObservation),
            Assert ActivitySubmission(ActivitySubmission),
            Match ActivityQuery(ActivityQuery),
        }
        reply OrchestrateReply {
            ClaimAcceptance(ClaimAcceptance),
            ClaimRejection(ClaimRejection),
            ReleaseAcknowledgment(ReleaseAcknowledgment),
            HandoffAcceptance(HandoffAcceptance),
            HandoffRejection(HandoffRejection),
            RoleSnapshot(RoleSnapshot),
            ActivityAcknowledgment(ActivityAcknowledgment),
            ActivityList(ActivityList),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OrchestrateOperationKind {
    RoleClaim,
    RoleRelease,
    RoleHandoff,
    RoleObservation,
    ActivitySubmission,
    ActivityQuery,
}

impl OrchestrateRequest {
    pub fn operation_kind(&self) -> OrchestrateOperationKind {
        match self {
            Self::RoleClaim(_) => OrchestrateOperationKind::RoleClaim,
            Self::RoleRelease(_) => OrchestrateOperationKind::RoleRelease,
            Self::RoleHandoff(_) => OrchestrateOperationKind::RoleHandoff,
            Self::RoleObservation(_) => OrchestrateOperationKind::RoleObservation,
            Self::ActivitySubmission(_) => OrchestrateOperationKind::ActivitySubmission,
            Self::ActivityQuery(_) => OrchestrateOperationKind::ActivityQuery,
        }
    }
}
