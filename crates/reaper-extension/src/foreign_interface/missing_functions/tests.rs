use super::*;

#[test]
fn nothing_is_reported_when_every_function_is_there() {
    assert_eq!(
        MissingFunctions::among(REQUIRED_FUNCTIONS, |_| true),
        Ok(())
    );
}

#[test]
fn a_function_the_reaper_lacks_is_named() {
    let names = ["GetPlayStateEx", "NoSuchFunction", "AlsoMissing"];
    let error = MissingFunctions::among(&names, |name| name == "GetPlayStateEx");
    assert_eq!(
        error,
        Err(MissingFunctions {
            names: vec!["NoSuchFunction", "AlsoMissing"]
        })
    );
    let Err(error) = error else {
        return;
    };
    assert_eq!(
        error.to_string(),
        "this REAPER lacks functions the extension needs: NoSuchFunction, AlsoMissing"
    );
}

#[test]
fn the_list_has_no_duplicates() {
    let mut sorted = REQUIRED_FUNCTIONS.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), REQUIRED_FUNCTIONS.len());
}
