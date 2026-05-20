//! Architectural-truth round-trip tests for the
//! `signal-persona-orchestrate` channel.
//!
//! Per `~/primary/skills/architectural-truth-tests.md`,
//! each variant of both enums has a witness test that
//! proves the macro-emitted type round-trips through a
//! length-prefixed Frame.

use signal_frame::{
    ExchangeIdentifier, ExchangeLane, LaneSequence, NonEmpty, Reply, RequestPayload, SessionEpoch,
    SubReply,
};
use signal_persona_orchestrate::{
    Activity, ActivityAcknowledgment, ActivityFilter, ActivityList, ActivityQuery,
    ActivitySubmission, ClaimAcceptance, ClaimEntry, ClaimRejection, EffectEmitted, Error,
    HandoffAcceptance, HandoffRejection, HandoffRejectionReason, HarnessKind, ObservationClosed,
    ObservationEvent, ObservationOpened, ObservationSubscription, ObservationToken, OperationKind,
    OperationReceived, OrchestrateEvent, OrchestrateFrame, OrchestrateFrameBody, OrchestrateReply,
    OrchestrateRequest, ReleaseAcknowledgment, RoleClaim, RoleHandoff, RoleName, RoleObservation,
    RoleRelease, RoleSnapshot, RoleStatus, ScopeConflict, ScopeReason, ScopeReference, TaskToken,
    TimestampNanos, WirePath,
};
use signal_sema::{SemaObservation, SemaOperation, SemaOutcome};

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
        reply: Reply::completed(NonEmpty::single(SubReply::Ok { payload: reply })),
    });
    let bytes = frame.encode_length_prefixed().expect("encode");
    let decoded = OrchestrateFrame::decode_length_prefixed(&bytes).expect("decode");
    match decoded.into_body() {
        OrchestrateFrameBody::Reply { reply, .. } => match reply {
            Reply::Accepted { per_operation, .. } => match per_operation.into_head() {
                SubReply::Ok { payload, .. } => payload,
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
    WirePath::from_absolute_path(
        "/git/github.com/LiGoldragon/signal-persona-orchestrate/src/lib.rs",
    )
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
    let request = OrchestrateRequest::Observe(RoleObservation);
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
fn observation_subscription_round_trips() {
    let request = OrchestrateRequest::Watch(ObservationSubscription {
        include_operations: true,
        include_sema_effects: true,
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
    let operation =
        OrchestrateEvent::Observed(ObservationEvent::OperationReceived(OperationReceived {
            operation: OperationKind::Claim,
        }));
    assert_eq!(round_trip_event(operation.clone()), operation);

    let effect = OrchestrateEvent::Observed(ObservationEvent::EffectEmitted(EffectEmitted {
        observation: SemaObservation::new(SemaOperation::Match, SemaOutcome::Matched),
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
            OrchestrateRequest::Observe(RoleObservation),
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
            OrchestrateRequest::Watch(ObservationSubscription {
                include_operations: true,
                include_sema_effects: false,
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
fn orchestrate_operations_encode_as_contract_local_nota_heads() {
    use nota_codec::{Decoder, Encoder, NotaDecode, NotaEncode};

    let request = OrchestrateRequest::Query(ActivityQuery {
        limit: 8,
        filters: vec![ActivityFilter::RoleFilter(operator())],
    });
    let mut encoder = Encoder::new();
    request.into_request().encode(&mut encoder).expect("encode");
    let text = encoder.into_string();

    assert!(text.starts_with("(Query "));
    assert!(!text.contains("Match"));
    assert!(!text.contains("Assert"));

    let mut decoder = Decoder::new(&text);
    let decoded = signal_persona_orchestrate::OrchestrateChannelRequest::decode(&mut decoder)
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
