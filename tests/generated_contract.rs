use signal_orchestrate::{LockRequest, Query, ReleaseRejection, Response, SignalArchive};

fn lock_query() -> Query {
    Query::Lock(LockRequest {
        lock_name: "lock".into(),
        flow_id: "flow".into(),
        lock_path_vector: vec!["/tmp/path".into()],
        lock_reason: "test".into(),
    })
}

#[test]
fn query_and_response_round_trip_as_portable_frames() {
    let query = lock_query();
    assert_eq!(
        Query::restore(&query.archive()).expect("restore query"),
        query
    );
    let response = Response::ReleaseRejected(ReleaseRejection::UnknownLockId);
    assert_eq!(
        Response::restore(&response.archive()).expect("restore response"),
        response
    );
}

#[cfg(feature = "datom")]
#[test]
fn query_round_trips_as_datom_text() {
    use datom_codec::{Actualizing, Budget, Datomizable, Potential};
    use protos::{Protosizable, ReaderBudget, Textualizable};

    let query = lock_query();
    let rendered = query.clone().datomize(vec![]).protosize().textualize();
    let mut pending = Potential::<Query>::from(rendered);
    let decoded = pending
        .actualize(&mut Budget {
            remaining: 1_024,
            reader: ReaderBudget { remaining: 1_024 },
            depth: 0,
            maximum_depth: 1_024,
        })
        .expect("restore datom query");
    assert_eq!(decoded, query);
}
