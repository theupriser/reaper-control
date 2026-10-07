use performance::Event;

/// The variant name of an event, for example "HandOverStarted".
pub(super) fn event_name(event: &Event) -> String {
    format!("{event:?}")
        .split(|c: char| !c.is_alphanumeric())
        .next()
        .unwrap_or_default()
        .to_string()
}
