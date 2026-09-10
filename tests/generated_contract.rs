use signal_orchestrate::{
    LockRequest, Query, ReleaseRejection, Response, archive_query, archive_response, restore_query,
    restore_response,
};

#[test]
fn query_and_response_round_trip_as_portable_frames() {
    let query = Query::Lock(LockRequest {
        lock_name: "lock".into(),
        flow_id: "flow".into(),
        lock_path_vector: vec!["/tmp/path".into()],
        lock_reason: "test".into(),
    });
    assert_eq!(
        restore_query(&archive_query(&query)).expect("restore query"),
        query
    );
    let response = Response::ReleaseRejected(ReleaseRejection::UnknownLockId);
    assert_eq!(
        restore_response(&archive_response(&response)).expect("restore response"),
        response
    );
}
