//! The marker helper reads a marker or region name the way the extension does.

use app::read_marker_name;

#[test]
fn a_name_with_every_directive_is_read_in_full() {
    let reading = read_marker_name("!length:45 !bpm:140 !1008");
    assert!(reading.hard_stop);
    assert_eq!(reading.length_seconds, Some(45.0));
    assert_eq!(reading.tempo_bpm, Some(140.0));
    assert!(reading.hidden);
}

#[test]
fn the_hardstop_alias_counts_as_a_hard_stop() {
    assert!(read_marker_name("Song A !hardstop").hard_stop);
}

#[test]
fn a_label_keeps_the_marker_visible() {
    let reading = read_marker_name("Chorus !bpm:120");
    assert_eq!(reading.tempo_bpm, Some(120.0));
    assert!(!reading.hidden);
}

#[test]
fn a_malformed_number_is_plain_text() {
    let reading = read_marker_name("!length:abc !bpm:");
    assert_eq!(reading.length_seconds, None);
    assert_eq!(reading.tempo_bpm, None);
    assert!(!reading.hard_stop);
}

#[test]
fn an_empty_name_means_nothing() {
    assert_eq!(read_marker_name(""), Default::default());
}
