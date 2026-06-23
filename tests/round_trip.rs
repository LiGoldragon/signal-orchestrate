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
use signal_orchestrate::{
    Activity, ActivityAcknowledgment, ActivityFilter, ActivityList, ActivityQuery,
    ActivitySubmission, ApplicationFailure, ApplicationFailureReason, ApplicationSuccess,
    ClaimAcceptance, ClaimEntry, ClaimRejection, CombinationRule, DownstreamComponent,
    EffectEmitted, EffectOutcome, Error, HandoffAcceptance, HandoffRejection,
    HandoffRejectionReason, HarnessKind, HostName, LaneAuthority, LaneIdentifier, LaneRegistration,
    LanesObserved, ModelAttestation, ModelName, Observation, ObservationClosed, ObservationEvent,
    ObservationOpened, ObservationSubscription, ObservationToken, OperationKind, OperationReceived,
    OrchestrateEvent, OrchestrateFrame, OrchestrateFrameBody, OrchestrateReply, OrchestrateRequest,
    PartialApplied, ProviderName, ReleaseAcknowledgment, Role, RoleClaim, RoleHandoff, RoleName,
    RoleRelease, RoleSnapshot, RoleStatus, RoleToken, ScopeConflict, ScopeReason, ScopeReference,
    StepLog, StepOutcome, StepThreshold, TaskToken, TimestampNanos, WirePath, WorkflowDefinition,
    WorkflowReceiptProduced, WorkflowRunAccepted, WorkflowRunDigest, WorkflowRunHandle,
    WorkflowRunLog, WorkflowRunLogReported, WorkflowRunObservation, WorkflowRunObservationClosed,
    WorkflowRunObservationOpened, WorkflowRunObservationToken, WorkflowRunRequest,
    WorkflowRunSnapshot, WorkflowRunUpdate, WorkflowStep, WorkflowStepName, Worktree,
    WorktreeStatus, WorktreesObserved,
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
        component: ComponentKind::Spirit,
        digest: operation_digest().object_digest().clone(),
        kind: AuthorizedObjectKind::Head,
    }
}

fn workflow_receipt() -> WorkflowReceipt {
    WorkflowReceipt {
        workflow: workflow_digest(),
        operation: operation_digest(),
        outcome: EvaluationDecision::Authorized,
        provenance: WorkflowProvenanceDigest::from_bytes(b"workflow-provenance"),
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
fn lane_observation_round_trips() {
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
    let request = OrchestrateRequest::RunWorkflow(WorkflowRunRequest {
        workflow: workflow_digest(),
        operation: authorized_object_reference(),
        contract: contract_digest(),
    });
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
fn lane_registry_records_round_trip() {
    let reply = OrchestrateReply::LanesObserved(LanesObserved {
        lanes: vec![
            LaneRegistration {
                lane: lane_identifier("designer"),
                role: role_vector(&["Designer"]),
                authority: LaneAuthority::Structural,
            },
            LaneRegistration {
                lane: lane_identifier("persona-signal-designer-assistant"),
                role: role_vector(&["PersonaSignal", "Designer"]),
                authority: LaneAuthority::Support,
            },
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

#[test]
fn generated_worktree_mirror_uses_canonical_status_field_name() {
    use signal_orchestrate::schema::lib as generated;

    let worktree = generated::Worktree {
        repository: generated::RepositoryName::new("signal-orchestrate"),
        branch: generated::BranchName::new("main"),
        path: generated::WirePath::new(
            "/home/li/wt/github.com/LiGoldragon/signal-orchestrate/main",
        ),
        owning_lane: generated::LaneName::new("operator"),
        status: generated::WorktreeStatus::Active,
        purpose: generated::PurposeText::new("schema mirror should match canonical field names"),
        last_activity: generated::TimestampNanos::new(1_730_000_002_000_000_000),
        pushed_state: generated::PushedState::Pushed,
    };

    assert_eq!(worktree.status, generated::WorktreeStatus::Active);
}

#[test]
fn role_vector_round_trips_through_nota() {
    use nota_next::{NotaEncode, NotaSource};

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

    let receipt = OrchestrateReply::WorkflowReceiptProduced(WorkflowReceiptProduced {
        handle: workflow_run_handle(),
        receipt: workflow_receipt(),
    });
    assert_eq!(round_trip_reply(receipt.clone()), receipt);

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
    use nota_next::{NotaEncode, NotaSource};

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
}
