use super::Fault;

#[test]
fn a_new_fault_has_not_tripped() {
    assert!(!Fault::new().is_faulted());
}

#[test]
fn guard_returns_the_result_of_work_that_succeeds() {
    let fault = Fault::new();
    assert_eq!(fault.guard(|| 7), Some(7));
    assert!(!fault.is_faulted());
}

#[test]
#[allow(clippy::panic)] // the panic is what is being contained
fn guard_contains_a_panic_and_trips_the_fault() {
    let fault = Fault::new();
    let result: Option<()> = fault.guard(|| panic!("injected"));
    assert_eq!(result, None);
    assert!(fault.is_faulted());
}

#[test]
fn once_faulted_guard_runs_nothing_more() {
    let fault = Fault::new();
    fault.trip();
    let mut ran = false;
    assert_eq!(
        fault.guard(|| {
            ran = true;
        }),
        None
    );
    assert!(!ran);
}
