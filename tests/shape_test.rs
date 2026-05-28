use databricks_tui::shape::Status;

#[test]
fn running_parses() {
    assert_eq!(Status::from_str("RUNNING"), Status::Running);
}

#[test]
fn terminated_maps_to_stopped() {
    assert_eq!(Status::from_str("TERMINATED"), Status::Stopped);
}

#[test]
fn starting_maps_to_pending() {
    assert_eq!(Status::from_str("STARTING"), Status::Pending);
}

#[test]
fn failed_parses() {
    assert_eq!(Status::from_str("FAILED"), Status::Failed);
}

#[test]
fn unknown_preserved() {
    assert_eq!(
        Status::from_str("SOME_NEW_STATE"),
        Status::Unknown("SOME_NEW_STATE".to_string())
    );
}
