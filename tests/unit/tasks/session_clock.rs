use super::{TaskSession, Value, admitted, end};

#[tokio::test]
async fn published_task_state_matches_the_browser_wire_fixture() {
    let fixture: Value =
        serde_json::from_str(include_str!("../../fixtures/task-state.json")).unwrap();
    assert_eq!(fixture["topic"], crate::runtime::TOPIC_TASK_STATE);
    let mut session = TaskSession::new(admitted().await, 100);
    assert_eq!(session.snapshot(100), fixture["cases"][0]["payload"]);
    session.start(100);
    session
        .end(end("hide", "invalid", "visibility"), 292)
        .unwrap();
    assert_eq!(session.snapshot(292), fixture["cases"][1]["payload"]);
}

#[tokio::test]
async fn snapshot_stamps_injected_server_time_without_changing_the_deadline() {
    let mut session = TaskSession::new(admitted().await, 100);
    assert_eq!(session.snapshot(100)["serverNow"], "1970-01-01T00:01:40Z");
    assert!(session.snapshot(100)["deadlineAt"].is_null());
    session.start(100);
    let first = session.snapshot(101);
    let later = session.snapshot(161);
    assert_eq!(first["serverNow"], "1970-01-01T00:01:41Z");
    assert_eq!(later["serverNow"], "1970-01-01T00:02:41Z");
    assert_eq!(first["deadlineAt"], "1970-01-01T00:16:40Z");
    assert_eq!(later["deadlineAt"], first["deadlineAt"]);
    assert_eq!(later["seq"], first["seq"]);
}

#[tokio::test]
async fn published_snapshot_is_stamped_with_the_current_clock() {
    let session = TaskSession::new(admitted().await, 100);
    let before = crate::delivery::rfc3339(crate::current_epoch_seconds_i64());
    let stamped = session.snapshot_now()["serverNow"]
        .as_str()
        .unwrap()
        .to_owned();
    let after = crate::delivery::rfc3339(crate::current_epoch_seconds_i64());
    // RFC 3339 in UTC sorts as time does.
    assert!(
        before <= stamped && stamped <= after,
        "{before} {stamped} {after}"
    );
}
