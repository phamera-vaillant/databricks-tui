use databricks_tui::shape::Status;

#[test]
fn running_parses() {
    assert_eq!("RUNNING".parse::<Status>().unwrap(), Status::Running);
}

#[test]
fn terminated_maps_to_stopped() {
    assert_eq!("TERMINATED".parse::<Status>().unwrap(), Status::Stopped);
}

#[test]
fn starting_maps_to_pending() {
    assert_eq!("STARTING".parse::<Status>().unwrap(), Status::Pending);
}

#[test]
fn failed_parses() {
    assert_eq!("FAILED".parse::<Status>().unwrap(), Status::Failed);
}

#[test]
fn unknown_preserved() {
    assert_eq!(
        "SOME_NEW_STATE".parse::<Status>().unwrap(),
        Status::Unknown("SOME_NEW_STATE".to_string())
    );
}
