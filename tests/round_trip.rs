//! Architectural-truth round-trip tests for the
//! `signal-orchestrate` channel.
//!
//! Per `~/primary/skills/architectural-truth-tests.md`,
//! each variant of both enums has a witness test that
//! proves the macro-emitted type round-trips through a
//! length-prefixed Frame.

use signal_criome::{
    AuthorizedObjectKind, AuthorizedObjectReference, ComponentKind, ContractDigest,
    EscalationTarget, EvaluationDecision, ObjectDigest, OperationDigest, WorkflowDigest,
    WorkflowProvenanceDigest, WorkflowReceipt,
};
use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SignalOperationHeads, SubReply,
};
use signal_harness::HarnessKind as ResolvedHarnessKind;
use signal_orchestrate::{
    Activity, ActivityAcknowledgment, ActivityFilter, ActivityList, ActivityQuery,
    ActivitySubmission, ApplicationFailure, ApplicationFailureReason, ApplicationSuccess,
    CapabilityProfile, ClaimAcceptance, ClaimEntry, ClaimRejection, CodexContinuationIdentifier,
    CombinationRule, ContinuationHandle, ContinuationRequest, DownstreamComponent, DurationNanos,
    EffectEmitted, EffectOutcome, EffortRequest, Error, HandoffAcceptance, HandoffRejection,
    HandoffRejectionReason, HarnessKind, HarnessName, HostName, LaneAssignment, LaneAuthority,
    LaneDetails, LaneIdentifier, LaneOwner, LaneProjection, LaneRegistration, LaneResourceClaim,
    LaneStatus, LanesObserved, ModelAttestation, ModelName, ModelRequest, ModelResolutionRequest,
    ModelResolved, ModelSelector, ModelUnavailable, ModelUnavailableReason, NamedModel,
    Observation, ObservationClosed, ObservationEvent, ObservationOpened, ObservationSubscription,
    ObservationToken, OperationKind, OperationReceived, OrchestrateEvent, OrchestrateFrame,
    OrchestrateFrameBody, OrchestrateReply, OrchestrateRequest, PartialApplied, ProviderName,
    ReleaseAcknowledgment, ResolvedWorkflowRunRequest, Role, RoleClaim, RoleHandoff, RoleName,
    RoleRelease, RoleSnapshot, RoleStatus, RoleToken, ScopeConflict, ScopeReason, ScopeReference,
    SessionIdentifier, SessionProjection, SessionsObserved, StepLog, StepOutcome, StepThreshold,
    TaskToken, TimestampNanos, WirePath, WorkflowDefinition, WorkflowReceiptProduced,
    WorkflowResolutionUnavailable, WorkflowResolvedReceiptProduced, WorkflowRunAccepted,
    WorkflowRunDigest, WorkflowRunHandle, WorkflowRunLog, WorkflowRunLogReported,
    WorkflowRunObservation, WorkflowRunObservationClosed, WorkflowRunObservationOpened,
    WorkflowRunObservationToken, WorkflowRunRequest, WorkflowRunResolution, WorkflowRunSnapshot,
    WorkflowRunUpdate, WorkflowStep, WorkflowStepName, Worktree, WorktreeConclusion,
    WorktreeConclusionRequest, WorktreeConcluded, WorktreeRequest, WorktreeRequestRejected,
    WorktreeRequestRejection, WorktreeScaffolded, WorktreeStatus, WorktreeTeardownRefused,
    WorktreesObserved, TeardownRefusal,
};
use signal_orchestrate::{
    AgentDirectory, AgentRegistered, AgentRegistrationRejected, AgentRegistrationRejectionReason,
    MissionDescription, OrchestratorAgentIdentifier, OrchestratorAgentRegistration,
    OrchestratorAgentStatus, OrchestratorAgentSummary, OrchestratorTopic, OrchestratorTopicPath,
    TopicAssignmentSource, TopicDetail, TopicName, TopicSelection, TopicTree,
};

// ─── Helpers ──────────────────────────────────────────────

fn exchange() -> ExchangeIdentifier {
    ExchangeIdentifier::new(
        SessionEpoch::new(1),
        ExchangeLane::Connector,
        LaneSequence::first(),
    )
}

fn round_trip_request(request: OrchestrateRequest) -> OrchestrateRequest {
    let frame = OrchestrateFrame::new(OrchestrateFrameBody::Request {
        exchange: exchange(),
        request: request.into_request(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = OrchestrateFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        OrchestrateFrameBody::Request { request, .. } => {
            let operation = request.payloads().head();
            operation.clone()
        }
        other => panic!("expected request operation, got {other:?}"),
    }
}

fn round_trip_reply(reply: OrchestrateReply) -> OrchestrateReply {
    let frame = OrchestrateFrame::new(OrchestrateFrameBody::Reply {
        exchange: exchange(),
        reply: Reply::committed(NonEmpty::single(SubReply::Ok(reply))),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = OrchestrateFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        OrchestrateFrameBody::Reply { reply, .. } => match reply {
            Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok(payload) => payload,
                other => panic!("expected accepted reply payload, got {other:?}"),
            },
            other => panic!("expected accepted reply, got {other:?}"),
        },
        other => panic!("expected reply operation, got {other:?}"),
    }
}

fn round_trip_event(event: OrchestrateEvent) -> OrchestrateEvent {
    let frame = OrchestrateFrame::new(OrchestrateFrameBody::SubscriptionEvent {
        event_identifier: signal_frame::StreamEventIdentifier::new(
            SessionEpoch::new(1),
            ExchangeLane::Acceptor,
            LaneSequence::first(),
        ),
        token: signal_frame::SubscriptionTokenInner::new(7),
        event: event.clone(),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = OrchestrateFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        OrchestrateFrameBody::SubscriptionEvent { event, .. } => event,
        other => panic!("expected subscription event, got {other:?}"),
    }
}

fn sample_path() -> WirePath {
    WirePath::from_absolute_path("/git/github.com/LiGoldragon/signal-orchestrate/src/lib.rs")
        .expect("sample path")
}

fn sample_task() -> TaskToken {
    TaskToken::from_wire_token("primary-f99").expect("sample task token")
}

fn sample_reason() -> ScopeReason {
    ScopeReason::from_text("design-cascade per /93").expect("sample reason")
}

fn role(token: &str) -> RoleName {
    RoleName::from_wire_token(token).expect("role")
}

fn operator() -> RoleName {
    role("operator")
}

fn operator_assistant() -> RoleName {
    role("operator-assistant")
}

fn second_operator_assistant() -> RoleName {
    role("second-operator-assistant")
}

fn designer() -> RoleName {
    role("designer")
}

fn designer_assistant() -> RoleName {
    role("designer-assistant")
}

fn second_designer_assistant() -> RoleName {
    role("second-designer-assistant")
}

fn system_specialist() -> RoleName {
    role("system-specialist")
}

fn system_assistant() -> RoleName {
    role("system-assistant")
}

fn second_system_assistant() -> RoleName {
    role("second-system-assistant")
}

fn poet() -> RoleName {
    role("poet")
}

fn poet_assistant() -> RoleName {
    role("poet-assistant")
}

fn sample_path_scope() -> ScopeReference {
    ScopeReference::Path(sample_path())
}

fn sample_task_scope() -> ScopeReference {
    ScopeReference::Task(sample_task())
}

fn role_token(token: &str) -> RoleToken {
    RoleToken::from_text(token).expect("role token")
}

fn role_vector(tokens: &[&str]) -> Role {
    Role::try_new(tokens.iter().map(|token| role_token(token)).collect()).expect("role vector")
}

fn lane_identifier(token: &str) -> LaneIdentifier {
    LaneIdentifier::from_wire_token(token).expect("lane identifier")
}

fn session_identifier(token: &str) -> SessionIdentifier {
    SessionIdentifier::from_camel_case_name(token).expect("session identifier")
}

fn lane_details(text: &str) -> LaneDetails {
    LaneDetails::from_text(text).expect("lane details")
}

fn lane_owner(tokens: &[&str], authority: LaneAuthority) -> LaneOwner {
    LaneOwner {
        role: role_vector(tokens),
        authority,
    }
}

fn lane_registration(
    session: &str,
    lane: &str,
    role_tokens: &[&str],
    authority: LaneAuthority,
) -> LaneRegistration {
    LaneRegistration {
        assignment: LaneAssignment {
            session: session_identifier(session),
            lane: lane_identifier(lane),
            owner: lane_owner(role_tokens, authority),
            details: lane_details("orchestrator assigned lane"),
        },
        registered_at: TimestampNanos::new(1_730_000_010_000_000_000),
        status: LaneStatus::Active,
    }
}

fn lane_projection(
    session: &str,
    lane: &str,
    role_tokens: &[&str],
    authority: LaneAuthority,
) -> LaneProjection {
    LaneProjection {
        registration: lane_registration(session, lane, role_tokens, authority),
        resource_claims: vec![LaneResourceClaim {
            scope: sample_path_scope(),
            reason: sample_reason(),
            claimed_at: TimestampNanos::new(1_730_000_011_000_000_000),
            age: DurationNanos::new(1_000_000_000),
        }],
        observed_at: TimestampNanos::new(1_730_000_012_000_000_000),
        age: DurationNanos::new(2_000_000_000),
    }
}

fn object_digest(text: &str) -> ObjectDigest {
    ObjectDigest::from_bytes(text.as_bytes())
}

fn workflow_digest() -> WorkflowDigest {
    WorkflowDigest::from_bytes(b"guardian-workflow")
}

fn contract_digest() -> ContractDigest {
    ContractDigest::from_bytes(b"criome-contract")
}

fn operation_digest() -> OperationDigest {
    OperationDigest::from_bytes(b"spirit-head-record")
}

fn workflow_run_digest() -> WorkflowRunDigest {
    WorkflowRunDigest::from_wire_token("workflow-run-1").expect("workflow run digest")
}

fn workflow_run_handle() -> WorkflowRunHandle {
    WorkflowRunHandle {
        run: workflow_run_digest(),
    }
}

fn authorized_object_reference() -> AuthorizedObjectReference {
    AuthorizedObjectReference {
        component_kind: ComponentKind::Spirit,
        object_digest: operation_digest().object_digest().clone(),
        authorized_object_kind: AuthorizedObjectKind::Head,
    }
}

fn workflow_receipt() -> WorkflowReceipt {
    WorkflowReceipt {
        workflow_digest: workflow_digest(),
        operation_digest: operation_digest(),
        evaluation_decision: EvaluationDecision::Authorized,
        workflow_provenance_digest: WorkflowProvenanceDigest::from_bytes(b"workflow-provenance"),
    }
}

fn model_attestation() -> ModelAttestation {
    ModelAttestation {
        provider: ProviderName::from_wire_token("local-provider").expect("provider"),
        model: ModelName::from_wire_token("guardian-model").expect("model"),
        host: HostName::from_wire_token("localhost").expect("host"),
        call: OperationDigest::from_bytes(b"llm-call"),
    }
}

fn step_log() -> StepLog {
    StepLog {
        step: WorkflowStepName::from_wire_token("guardian").expect("step"),
        attestation: model_attestation(),
        outcome: StepOutcome::Produced(EvaluationDecision::Authorized),
    }
}

fn workflow_run_log() -> WorkflowRunLog {
    WorkflowRunLog {
        run: workflow_run_digest(),
        step_logs: vec![step_log()],
    }
}

fn workflow_run_request() -> WorkflowRunRequest {
    WorkflowRunRequest {
        workflow: workflow_digest(),
        operation: authorized_object_reference(),
        contract: contract_digest(),
    }
}

fn model_resolution_request() -> ModelResolutionRequest {
    ModelResolutionRequest {
        model: ModelRequest {
            selector: ModelSelector::CapabilityProfile(CapabilityProfile::new("orchestrator")),
            effort: EffortRequest::High,
        },
        continuation: ContinuationRequest::Prefer(ContinuationHandle::Codex(
            CodexContinuationIdentifier::new("codex-turn-7"),
        )),
    }
}

fn resolved_workflow_run_request() -> ResolvedWorkflowRunRequest {
    ResolvedWorkflowRunRequest {
        workflow_run: workflow_run_request(),
        model_resolution: model_resolution_request(),
    }
}

fn model_resolved() -> ModelResolved {
    ModelResolved {
        harness: HarnessName::new("codex-main"),
        harness_kind: ResolvedHarnessKind::Codex,
        model: NamedModel::new("gpt-5-codex"),
        effort: EffortRequest::High,
        continuation: ContinuationHandle::Codex(CodexContinuationIdentifier::new("codex-turn-8")),
    }
}

fn model_unavailable() -> ModelUnavailable {
    ModelUnavailable {
        request: model_resolution_request(),
        reason: ModelUnavailableReason::CapabilityUnsupported,
    }
}

fn workflow_run_resolution() -> WorkflowRunResolution {
    WorkflowRunResolution {
        handle: workflow_run_handle(),
        resolution: model_resolved(),
    }
}

// ─── Request variants ─────────────────────────────────────

#[test]
fn role_claim_with_paths_round_trips() {
    let request = OrchestrateRequest::Claim(RoleClaim {
        role: designer(),
        scopes: vec![sample_path_scope(), sample_task_scope()],
        reason: sample_reason(),
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn role_release_round_trips() {
    let request = OrchestrateRequest::Release(RoleRelease { role: operator() });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn role_handoff_round_trips() {
    let request = OrchestrateRequest::Handoff(RoleHandoff {
        from: designer(),
        to: operator(),
        scopes: vec![sample_path_scope()],
        reason: ScopeReason::from_text("router migration handoff").expect("sample reason"),
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn role_observation_round_trips() {
    let request = OrchestrateRequest::Observe(Observation::Roles);
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn sessions_observation_round_trips() {
    let request = OrchestrateRequest::Observe(Observation::Sessions);
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn session_lanes_observation_round_trips() {
    let request = OrchestrateRequest::Observe(Observation::SessionLanes(session_identifier(
        "SessionLaneProtocolContracts",
    )));
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn all_lanes_observation_round_trips() {
    let request = OrchestrateRequest::Observe(Observation::Lanes);
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn activity_submission_round_trips() {
    let request = OrchestrateRequest::Submit(ActivitySubmission {
        role: operator_assistant(),
        scope: sample_path_scope(),
        reason: ScopeReason::from_text("audit signal-persona-system integration")
            .expect("sample reason"),
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn activity_query_unfiltered_round_trips() {
    let request = OrchestrateRequest::Query(ActivityQuery {
        limit: 25,
        filters: vec![],
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn activity_query_with_role_filter_round_trips() {
    let request = OrchestrateRequest::Query(ActivityQuery {
        limit: 50,
        filters: vec![ActivityFilter::RoleFilter(operator())],
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn activity_query_with_path_prefix_round_trips() {
    let request = OrchestrateRequest::Query(ActivityQuery {
        limit: 10,
        filters: vec![ActivityFilter::PathPrefix(
            WirePath::from_absolute_path("/git/github.com/LiGoldragon/persona-router")
                .expect("sample path"),
        )],
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn activity_query_with_task_filter_round_trips() {
    let request = OrchestrateRequest::Query(ActivityQuery {
        limit: 100,
        filters: vec![ActivityFilter::TaskToken(sample_task())],
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn workflow_run_request_round_trips() {
    let request = OrchestrateRequest::RunWorkflow(workflow_run_request());
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn resolved_workflow_run_request_round_trips() {
    let request = OrchestrateRequest::RunResolvedWorkflow(resolved_workflow_run_request());
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn workflow_run_observation_round_trips() {
    let request = OrchestrateRequest::ObserveWorkflowRun(WorkflowRunObservation {
        run: workflow_run_digest(),
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);

    let request =
        OrchestrateRequest::WorkflowRunObservationRetraction(WorkflowRunObservationToken {
            run: workflow_run_digest(),
        });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn observation_subscription_round_trips() {
    let request = OrchestrateRequest::Watch(ObservationSubscription {
        include_operations: true,
        include_effects: true,
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);

    let request = OrchestrateRequest::Unwatch(ObservationToken::new(7));
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

// ─── Reply variants ───────────────────────────────────────

#[test]
fn claim_acceptance_round_trips() {
    let reply = OrchestrateReply::ClaimAcceptance(ClaimAcceptance {
        role: designer(),
        scopes: vec![sample_path_scope()],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn claim_rejection_round_trips() {
    let reply = OrchestrateReply::ClaimRejection(ClaimRejection {
        role: designer(),
        conflicts: vec![ScopeConflict {
            scope: sample_path_scope(),
            held_by: operator(),
            held_reason: ScopeReason::from_text("Persona-prefix sweep").expect("sample reason"),
        }],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn release_acknowledgment_round_trips() {
    let reply = OrchestrateReply::ReleaseAcknowledgment(ReleaseAcknowledgment {
        role: designer(),
        released_scopes: vec![sample_path_scope(), sample_task_scope()],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn handoff_acceptance_round_trips() {
    let reply = OrchestrateReply::HandoffAcceptance(HandoffAcceptance {
        from: designer(),
        to: operator(),
        scopes: vec![sample_path_scope()],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn handoff_rejection_source_does_not_hold_round_trips() {
    let reply = OrchestrateReply::HandoffRejection(HandoffRejection {
        from: designer(),
        to: operator(),
        reason: HandoffRejectionReason::SourceRoleDoesNotHold,
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn handoff_rejection_target_conflict_round_trips() {
    let reply = OrchestrateReply::HandoffRejection(HandoffRejection {
        from: designer(),
        to: operator(),
        reason: HandoffRejectionReason::TargetRoleConflict(vec![ScopeConflict {
            scope: sample_path_scope(),
            held_by: operator_assistant(),
            held_reason: ScopeReason::from_text("audit pass").expect("sample reason"),
        }]),
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn role_snapshot_round_trips() {
    let reply = OrchestrateReply::RoleSnapshot(RoleSnapshot {
        roles: vec![
            RoleStatus {
                role: designer(),
                harness: HarnessKind::Claude,
                claims: vec![ClaimEntry {
                    scope: sample_path_scope(),
                    reason: sample_reason(),
                    claimed_at: TimestampNanos::new(1_730_000_011_000_000_000),
                    age: DurationNanos::new(1_000_000_000),
                }],
            },
            RoleStatus {
                role: operator(),
                harness: HarnessKind::Codex,
                claims: vec![],
            },
        ],
        recent_activity: vec![Activity {
            role: designer(),
            scope: sample_path_scope(),
            reason: sample_reason(),
            stamped_at: TimestampNanos::new(1_730_000_000_000_000_000),
        }],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn session_registry_records_round_trip() {
    let reply = OrchestrateReply::SessionsObserved(SessionsObserved {
        sessions: vec![SessionProjection {
            session: session_identifier("SessionLaneProtocolContracts"),
            active_lanes: 2,
        }],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn lane_projection_records_round_trip_with_age_status_and_claims() {
    let reply = OrchestrateReply::LanesObserved(LanesObserved {
        lanes: vec![
            lane_projection(
                "SessionLaneProtocolContracts",
                "contract-implementation",
                &["Designer"],
                LaneAuthority::Structural,
            ),
            lane_projection(
                "SessionLaneProtocolContracts",
                "contract-review",
                &["PersonaSignal", "Designer"],
                LaneAuthority::Support,
            ),
        ],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn worktree_registry_records_round_trip() {
    let reply = OrchestrateReply::WorktreesObserved(WorktreesObserved {
        worktrees: vec![Worktree {
            repository: signal_orchestrate::RepositoryName::from_text("signal-orchestrate")
                .expect("repository name"),
            branch: signal_orchestrate::BranchName::from_text("main").expect("branch name"),
            path: WirePath::from_absolute_path(
                "/home/li/wt/github.com/LiGoldragon/signal-orchestrate/main",
            )
            .expect("worktree path"),
            owning_lane: signal_orchestrate::LaneName::from_text("operator").expect("owning lane"),
            status: WorktreeStatus::Active,
            purpose: signal_orchestrate::PurposeText::from_text(
                "signal-orchestrate worktree contract fidelity",
            )
            .expect("purpose"),
            last_activity: TimestampNanos::new(1_730_000_002_000_000_000),
            pushed_state: signal_orchestrate::PushedState::Pushed,
        }],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

fn sample_worktree(status: WorktreeStatus) -> Worktree {
    Worktree {
        repository: signal_orchestrate::RepositoryName::from_text("orchestrate")
            .expect("repository name"),
        branch: signal_orchestrate::BranchName::from_text("worktree-lifecycle")
            .expect("branch name"),
        path: WirePath::from_absolute_path(
            "/home/li/wt/github.com/LiGoldragon/orchestrate/worktree-lifecycle",
        )
        .expect("worktree path"),
        owning_lane: signal_orchestrate::LaneName::from_text("OrchestratorWorktreeProtocol")
            .expect("owning lane"),
        status,
        purpose: signal_orchestrate::PurposeText::from_text("worktree lifecycle protocol")
            .expect("purpose"),
        last_activity: TimestampNanos::new(1_730_000_003_000_000_000),
        pushed_state: signal_orchestrate::PushedState::AncestorOfMain,
    }
}

#[test]
fn request_worktree_round_trips() {
    let request = OrchestrateRequest::RequestWorktree(WorktreeRequest {
        repository: signal_orchestrate::RepositoryName::from_text("orchestrate")
            .expect("repository name"),
        branch: signal_orchestrate::BranchName::from_text("worktree-lifecycle")
            .expect("branch name"),
        owning_lane: signal_orchestrate::LaneName::from_text("OrchestratorWorktreeProtocol")
            .expect("owning lane"),
        purpose: signal_orchestrate::PurposeText::from_text("worktree lifecycle protocol")
            .expect("purpose"),
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
    assert_eq!(request.operation_kind(), OperationKind::RequestWorktree);
}

#[test]
fn conclude_worktree_round_trips_each_disposition() {
    for disposition in [WorktreeConclusion::Merged, WorktreeConclusion::Rejected] {
        let request = OrchestrateRequest::ConcludeWorktree(WorktreeConclusionRequest {
            owning_lane: signal_orchestrate::LaneName::from_text("OrchestratorWorktreeProtocol")
                .expect("owning lane"),
            disposition,
        });
        let decoded = round_trip_request(request.clone());
        assert_eq!(decoded, request);
        assert_eq!(request.operation_kind(), OperationKind::ConcludeWorktree);
    }
}

#[test]
fn worktree_scaffolded_round_trips() {
    let reply = OrchestrateReply::WorktreeScaffolded(WorktreeScaffolded {
        worktree: sample_worktree(WorktreeStatus::Active),
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn worktree_request_rejected_round_trips() {
    let reply = OrchestrateReply::WorktreeRequestRejected(WorktreeRequestRejected {
        reason: WorktreeRequestRejection::WorktreeAlreadyExists,
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn worktree_concluded_round_trips_with_abandoned_status() {
    let reply = OrchestrateReply::WorktreeConcluded(WorktreeConcluded {
        worktree: sample_worktree(WorktreeStatus::Abandoned),
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn worktree_teardown_refused_round_trips() {
    let reply = OrchestrateReply::WorktreeTeardownRefused(WorktreeTeardownRefused {
        worktree: sample_worktree(WorktreeStatus::Active),
        reason: TeardownRefusal::UnmergedWorkPresent,
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn generated_worktree_mirror_uses_canonical_status_field_name() {
    use signal_orchestrate::schema::lib as generated;

    let worktree = generated::Worktree {
        repository_name: generated::RepositoryName::new("signal-orchestrate"),
        branch_name: generated::BranchName::new("main"),
        wire_path: generated::WirePath::new(
            "/home/li/wt/github.com/LiGoldragon/signal-orchestrate/main",
        ),
        lane_name: generated::LaneName::new("operator"),
        worktree_status: generated::WorktreeStatus::Active,
        purpose_text: generated::PurposeText::new(
            "schema mirror should match canonical field names",
        ),
        timestamp_nanos: generated::TimestampNanos::new(1_730_000_002_000_000_000),
        pushed_state: generated::PushedState::Pushed,
    };

    assert_eq!(worktree.worktree_status, generated::WorktreeStatus::Active);
}

#[test]
fn role_vector_round_trips_through_nota() {
    use nota::{NotaEncode, NotaSource};

    let role = role_vector(&["PersonaSignal", "Designer"]);
    let text = role.to_nota();

    let decoded = NotaSource::new(&text).parse::<Role>().expect("decode role");
    assert_eq!(decoded, role);
}

#[test]
fn activity_acknowledgment_round_trips() {
    let reply = OrchestrateReply::ActivityAcknowledgment(ActivityAcknowledgment { slot: 42 });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn activity_list_round_trips() {
    let reply = OrchestrateReply::ActivityList(ActivityList {
        records: vec![
            Activity {
                role: designer(),
                scope: sample_path_scope(),
                reason: ScopeReason::from_text("rescope per /91 §3.1").expect("sample reason"),
                stamped_at: TimestampNanos::new(1_730_000_000_000_000_000),
            },
            Activity {
                role: operator(),
                scope: sample_task_scope(),
                reason: ScopeReason::from_text("ractor adoption").expect("sample reason"),
                stamped_at: TimestampNanos::new(1_730_000_001_000_000_000),
            },
        ],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn workflow_run_replies_round_trip() {
    let accepted = OrchestrateReply::WorkflowRunAccepted(WorkflowRunAccepted {
        handle: workflow_run_handle(),
    });
    assert_eq!(round_trip_reply(accepted.clone()), accepted);

    let resolution = OrchestrateReply::WorkflowResolutionAccepted(workflow_run_resolution());
    assert_eq!(round_trip_reply(resolution.clone()), resolution);

    let unavailable =
        OrchestrateReply::WorkflowResolutionUnavailable(WorkflowResolutionUnavailable {
            handle: workflow_run_handle(),
            request: resolved_workflow_run_request(),
            unavailable: model_unavailable(),
        });
    assert_eq!(round_trip_reply(unavailable.clone()), unavailable);

    let receipt = OrchestrateReply::WorkflowReceiptProduced(WorkflowReceiptProduced {
        handle: workflow_run_handle(),
        receipt: workflow_receipt(),
    });
    assert_eq!(round_trip_reply(receipt.clone()), receipt);

    let resolved_receipt =
        OrchestrateReply::WorkflowResolvedReceiptProduced(WorkflowResolvedReceiptProduced {
            run: workflow_run_resolution(),
            receipt: workflow_receipt(),
        });
    assert_eq!(round_trip_reply(resolved_receipt.clone()), resolved_receipt);

    let log = OrchestrateReply::WorkflowRunLogReported(WorkflowRunLogReported {
        log: workflow_run_log(),
    });
    assert_eq!(round_trip_reply(log.clone()), log);
}

#[test]
fn workflow_run_observation_replies_round_trip() {
    let opened = OrchestrateReply::WorkflowRunObservationOpened(WorkflowRunObservationOpened {
        token: WorkflowRunObservationToken {
            run: workflow_run_digest(),
        },
        snapshot: WorkflowRunSnapshot {
            handle: workflow_run_handle(),
            latest_log: Some(workflow_run_log()),
            receipt: Some(workflow_receipt()),
        },
    });
    assert_eq!(round_trip_reply(opened.clone()), opened);

    let closed = OrchestrateReply::WorkflowRunObservationClosed(WorkflowRunObservationClosed {
        token: WorkflowRunObservationToken {
            run: workflow_run_digest(),
        },
    });
    assert_eq!(round_trip_reply(closed.clone()), closed);
}

#[test]
fn workflow_definition_carries_dag_and_escalation_shape() {
    let definition = WorkflowDefinition {
        steps: vec![
            WorkflowStep {
                name: WorkflowStepName::from_wire_token("guardian").expect("step"),
                prompt: object_digest("guardian-prompt-template"),
                provider: Some(ProviderName::from_wire_token("local-provider").expect("provider")),
                dependencies: vec![],
            },
            WorkflowStep {
                name: WorkflowStepName::from_wire_token("psyche-check").expect("step"),
                prompt: object_digest("psyche-escalation-template"),
                provider: None,
                dependencies: vec![WorkflowStepName::from_wire_token("guardian").expect("step")],
            },
        ],
        combination: CombinationRule::Threshold(StepThreshold::new(2)),
        escalation: Some(EscalationTarget::Psyche),
    };

    assert_eq!(definition.steps.len(), 2);
    assert_eq!(
        definition.combination,
        CombinationRule::Threshold(StepThreshold::new(2))
    );
    assert_eq!(definition.escalation, Some(EscalationTarget::Psyche));
}

#[test]
fn partial_applied_round_trips() {
    let reply = OrchestrateReply::PartialApplied(PartialApplied {
        succeeded: vec![ApplicationSuccess {
            component: DownstreamComponent::Router,
            detail: ScopeReason::from_text("channel 42 installed").expect("success detail"),
        }],
        failed: vec![ApplicationFailure {
            component: DownstreamComponent::Harness,
            reason: ApplicationFailureReason::Unreachable,
            detail: ScopeReason::from_text("codex-7 transcript is gone").expect("failure detail"),
        }],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn observation_replies_round_trip() {
    let opened = OrchestrateReply::ObservationOpened(ObservationOpened {
        token: ObservationToken::new(7),
    });
    assert_eq!(round_trip_reply(opened.clone()), opened);

    let closed = OrchestrateReply::ObservationClosed(ObservationClosed {
        token: ObservationToken::new(7),
    });
    assert_eq!(round_trip_reply(closed.clone()), closed);
}

#[test]
fn observation_events_round_trip() {
    let workflow = OrchestrateEvent::WorkflowRunUpdated(WorkflowRunUpdate {
        run: workflow_run_digest(),
        log: Some(workflow_run_log()),
        receipt: Some(workflow_receipt()),
    });
    assert_eq!(round_trip_event(workflow.clone()), workflow);

    let operation =
        OrchestrateEvent::Observed(ObservationEvent::OperationReceived(OperationReceived {
            operation: OperationKind::Claim,
        }));
    assert_eq!(round_trip_event(operation.clone()), operation);

    let effect = OrchestrateEvent::Observed(ObservationEvent::EffectEmitted(EffectEmitted {
        operation: OperationKind::Query,
        outcome: EffectOutcome::Observed,
    }));
    assert_eq!(round_trip_event(effect.clone()), effect);
}

// ─── Scope-reference variants ─────────────────────────────

#[test]
fn role_name_parses_workspace_coordination_tokens() {
    let cases = [
        ("operator", operator()),
        ("operator-assistant", operator_assistant()),
        ("second-operator-assistant", second_operator_assistant()),
        ("designer", designer()),
        ("designer-assistant", designer_assistant()),
        ("second-designer-assistant", second_designer_assistant()),
        ("system-specialist", system_specialist()),
        ("system-assistant", system_assistant()),
        ("second-system-assistant", second_system_assistant()),
        ("poet", poet()),
        ("poet-assistant", poet_assistant()),
    ];

    assert_eq!(RoleName::CURRENT_WORKSPACE_ROLE_TOKENS.len(), cases.len());
    for (token, role) in cases {
        assert_eq!(RoleName::from_wire_token(token), Ok(role.clone()));
        assert_eq!(token.parse::<RoleName>(), Ok(role.clone()));
        assert_eq!(role.as_wire_token(), token);
        assert_eq!(role.to_string(), token);
    }
}

#[test]
fn path_scope_round_trips() {
    let request = OrchestrateRequest::Claim(RoleClaim {
        role: designer(),
        scopes: vec![ScopeReference::Path(sample_path())],
        reason: sample_reason(),
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn task_scope_round_trips() {
    let request = OrchestrateRequest::Claim(RoleClaim {
        role: designer(),
        scopes: vec![ScopeReference::Task(sample_task())],
        reason: sample_reason(),
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn orchestrate_request_exposes_contract_owned_operation_kind() {
    let cases = vec![
        (
            OrchestrateRequest::Claim(RoleClaim {
                role: designer(),
                scopes: vec![sample_path_scope()],
                reason: sample_reason(),
            }),
            OperationKind::Claim,
        ),
        (
            OrchestrateRequest::Release(RoleRelease { role: operator() }),
            OperationKind::Release,
        ),
        (
            OrchestrateRequest::Handoff(RoleHandoff {
                from: designer(),
                to: operator(),
                scopes: vec![sample_path_scope()],
                reason: sample_reason(),
            }),
            OperationKind::Handoff,
        ),
        (
            OrchestrateRequest::Observe(Observation::Roles),
            OperationKind::Observe,
        ),
        (
            OrchestrateRequest::Submit(ActivitySubmission {
                role: operator(),
                scope: sample_path_scope(),
                reason: sample_reason(),
            }),
            OperationKind::Submit,
        ),
        (
            OrchestrateRequest::Query(ActivityQuery {
                limit: 10,
                filters: vec![ActivityFilter::RoleFilter(operator())],
            }),
            OperationKind::Query,
        ),
        (
            OrchestrateRequest::RunWorkflow(WorkflowRunRequest {
                workflow: workflow_digest(),
                operation: authorized_object_reference(),
                contract: contract_digest(),
            }),
            OperationKind::RunWorkflow,
        ),
        (
            OrchestrateRequest::ObserveWorkflowRun(WorkflowRunObservation {
                run: workflow_run_digest(),
            }),
            OperationKind::ObserveWorkflowRun,
        ),
        (
            OrchestrateRequest::WorkflowRunObservationRetraction(WorkflowRunObservationToken {
                run: workflow_run_digest(),
            }),
            OperationKind::WorkflowRunObservationRetraction,
        ),
        (
            OrchestrateRequest::Watch(ObservationSubscription {
                include_operations: true,
                include_effects: false,
            }),
            OperationKind::Watch,
        ),
        (
            OrchestrateRequest::Unwatch(ObservationToken::new(3)),
            OperationKind::Unwatch,
        ),
    ];

    for (request, operation) in cases {
        assert_eq!(request.operation_kind(), operation);
    }
}

#[test]
fn orchestrate_contract_has_no_sema_observation_or_classification_roots() {
    let manifest = include_str!("../Cargo.toml");
    assert!(
        !manifest.contains("signal-sema"),
        "ordinary signal contracts must not depend on signal-sema for public wire vocabulary"
    );

    for source in [
        include_str!("../src/lib.rs"),
        include_str!("../schema/lib.schema"),
        include_str!("../schema/signal-orchestrate.concept.schema"),
    ] {
        assert!(
            !source.contains("SemaObservation"),
            "EffectEmitted must stay a contract-owned operation/outcome event, not a SemaObservation payload"
        );
    }

    let heads = <OrchestrateRequest as SignalOperationHeads>::HEADS;
    for forbidden in [
        "Assert",
        "Mutate",
        "Retract",
        "Match",
        "Subscribe",
        "Validate",
    ] {
        assert!(
            !heads.contains(&forbidden),
            "Sema classification root {forbidden} must not appear on the public orchestrate wire"
        );
    }
}

#[test]
#[cfg(feature = "nota-text")]
fn orchestrate_operations_encode_as_contract_local_nota_heads() {
    use nota::{NotaEncode, NotaSource};

    let request = OrchestrateRequest::Query(ActivityQuery {
        limit: 8,
        filters: vec![ActivityFilter::RoleFilter(operator())],
    });
    let text = request.into_request().to_nota();

    assert!(text.starts_with("(Query "));
    assert!(!text.contains("Match"));
    assert!(!text.contains("Assert"));

    let decoded = NotaSource::new(&text)
        .parse::<signal_orchestrate::OrchestrateChannelRequest>()
        .expect("decode request");
    assert_eq!(
        decoded.payloads().head().operation_kind(),
        OperationKind::Query
    );
}

#[test]
fn scope_primitives_reject_invalid_values() {
    assert!(matches!(
        WirePath::from_absolute_path("relative/path"),
        Err(Error::InvalidWirePath { .. })
    ));
    assert!(matches!(
        TaskToken::from_wire_token("primary hrhz"),
        Err(Error::InvalidTaskToken { .. })
    ));
    assert!(matches!(
        ScopeReason::from_text(""),
        Err(Error::InvalidScopeReason { .. })
    ));
    assert!(matches!(
        RoleName::from_wire_token("bad role"),
        Err(Error::InvalidRoleIdentifier { .. })
    ));
    assert!(matches!(
        SessionIdentifier::from_camel_case_name("notCamelCase"),
        Err(Error::InvalidSessionIdentifier { .. })
    ));
}

// ─── Orchestrator seat ────────────────────────────────────

fn agent_identifier(token: &str) -> OrchestratorAgentIdentifier {
    OrchestratorAgentIdentifier::from_wire_token(token).expect("agent identifier")
}

fn topic_path(path: &str) -> OrchestratorTopicPath {
    OrchestratorTopicPath::from_wire_token(path).expect("topic path")
}

fn mission(text: &str) -> MissionDescription {
    MissionDescription::from_text(text).expect("mission")
}

fn topic(path: &str, name: &str, parent: Option<&str>) -> OrchestratorTopic {
    OrchestratorTopic {
        path: topic_path(path),
        name: TopicName::from_text(name).expect("topic name"),
        parent: parent.map(topic_path),
    }
}

fn sample_topics() -> Vec<OrchestratorTopic> {
    vec![
        topic("infrastructure", "Infrastructure", None),
        topic("infrastructure/nix", "Nix", Some("infrastructure")),
    ]
}

#[test]
fn register_agent_automatic_round_trips() {
    let request = OrchestrateRequest::RegisterAgent(OrchestratorAgentRegistration {
        session: session_identifier("OrchestratorMessaging"),
        mission: mission("Design the orchestrator messaging wire contracts."),
        harness: HarnessKind::Claude,
        topic_selection: TopicSelection::Automatic,
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn register_agent_explicit_round_trips() {
    let request = OrchestrateRequest::RegisterAgent(OrchestratorAgentRegistration {
        session: session_identifier("OrchestratorMessaging"),
        mission: mission("Author the judge crate."),
        harness: HarnessKind::Codex,
        topic_selection: TopicSelection::Explicit(vec![
            topic_path("infrastructure/nix"),
            topic_path("contracts"),
        ]),
    });
    let decoded = round_trip_request(request.clone());
    assert_eq!(decoded, request);
}

#[test]
fn topic_and_agent_observations_round_trip() {
    for observation in [
        Observation::Topics,
        Observation::Topic(topic_path("infrastructure/nix")),
        Observation::Agents,
    ] {
        let request = OrchestrateRequest::Observe(observation);
        let decoded = round_trip_request(request.clone());
        assert_eq!(decoded, request);
    }
}

#[test]
fn register_agent_exposes_operation_kind() {
    let request = OrchestrateRequest::RegisterAgent(OrchestratorAgentRegistration {
        session: session_identifier("OrchestratorMessaging"),
        mission: mission("kind witness"),
        harness: HarnessKind::Claude,
        topic_selection: TopicSelection::Automatic,
    });
    assert_eq!(request.operation_kind(), OperationKind::RegisterAgent);
}

#[test]
fn agent_registered_round_trips() {
    let reply = OrchestrateReply::AgentRegistered(AgentRegistered {
        agent_identifier: agent_identifier("4zqk"),
        assigned_topics: sample_topics(),
        assignment_source: TopicAssignmentSource::Judge,
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);

    let explicit = OrchestrateReply::AgentRegistered(AgentRegistered {
        agent_identifier: agent_identifier("7mtp"),
        assigned_topics: vec![topic("contracts", "Contracts", None)],
        assignment_source: TopicAssignmentSource::Explicit,
    });
    let decoded = round_trip_reply(explicit.clone());
    assert_eq!(decoded, explicit);
}

#[test]
fn agent_registration_rejected_round_trips_for_every_reason() {
    let reasons = [
        AgentRegistrationRejectionReason::MissionEmpty,
        AgentRegistrationRejectionReason::MissionTooVague,
        AgentRegistrationRejectionReason::UnknownTopic,
        AgentRegistrationRejectionReason::JudgeUnavailable,
        AgentRegistrationRejectionReason::JudgeMalformed,
        AgentRegistrationRejectionReason::JudgeTimedOut,
    ];
    for reason in reasons {
        let reply = OrchestrateReply::AgentRegistrationRejected(AgentRegistrationRejected {
            reason,
            available_topics: sample_topics(),
        });
        let decoded = round_trip_reply(reply.clone());
        assert_eq!(decoded, reply);
    }
}

#[test]
fn topic_tree_round_trips() {
    let reply = OrchestrateReply::TopicTree(TopicTree {
        topics: sample_topics(),
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn topic_detail_round_trips() {
    let reply = OrchestrateReply::TopicDetail(TopicDetail {
        topic: topic("infrastructure/nix", "Nix", Some("infrastructure")),
        member_agent_identifiers: vec![agent_identifier("4zqk"), agent_identifier("7mtp")],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn agent_directory_round_trips() {
    let reply = OrchestrateReply::AgentDirectory(AgentDirectory {
        agents: vec![
            OrchestratorAgentSummary {
                agent_identifier: agent_identifier("4zqk"),
                mission: mission("Author contracts."),
                topics: vec![topic_path("contracts")],
                status: OrchestratorAgentStatus::Active,
            },
            OrchestratorAgentSummary {
                agent_identifier: agent_identifier("7mtp"),
                mission: mission("Retired worker."),
                topics: vec![],
                status: OrchestratorAgentStatus::Retired,
            },
            OrchestratorAgentSummary {
                agent_identifier: agent_identifier("9wce"),
                mission: mission("Crashed worker."),
                topics: vec![],
                status: OrchestratorAgentStatus::Dead,
            },
        ],
    });
    let decoded = round_trip_reply(reply.clone());
    assert_eq!(decoded, reply);
}

#[test]
fn dead_agent_status_is_terminal() {
    assert!(OrchestratorAgentStatus::Dead.is_terminal());
    assert!(OrchestratorAgentStatus::Retired.is_terminal());
    assert!(!OrchestratorAgentStatus::Active.is_terminal());
}

#[test]
fn single_segment_path_yields_one_root_topic() {
    let lineage = topic_path("engineering").lineage().expect("lineage");
    assert_eq!(lineage, vec![topic("engineering", "engineering", None)]);
}

#[test]
fn nested_path_yields_each_implied_parent_root_first() {
    let lineage = topic_path("coordination/messaging")
        .lineage()
        .expect("lineage");
    assert_eq!(
        lineage,
        vec![
            topic("coordination", "coordination", None),
            topic("coordination/messaging", "messaging", Some("coordination")),
        ]
    );
}

#[test]
fn deep_path_names_each_topic_after_its_own_final_segment() {
    let lineage = topic_path("a/b/c").lineage().expect("lineage");
    assert_eq!(
        lineage,
        vec![
            topic("a", "a", None),
            topic("a/b", "b", Some("a")),
            topic("a/b/c", "c", Some("a/b")),
        ]
    );
}

#[test]
fn stray_slashes_never_mint_a_nameless_topic() {
    // Leading, trailing, and doubled slashes are empty segments and skipped;
    // the lineage matches the clean nested path.
    let lineage = topic_path("/coordination//messaging/")
        .lineage()
        .expect("lineage");
    assert_eq!(
        lineage,
        vec![
            topic("coordination", "coordination", None),
            topic("coordination/messaging", "messaging", Some("coordination")),
        ]
    );
}
