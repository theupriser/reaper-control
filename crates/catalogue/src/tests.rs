use std::path::PathBuf;

use serde::Deserialize;
use shared_kernel::{Bpm, Seconds, SongId};

use super::*;

#[derive(Deserialize)]
struct Vector {
    name: String,
    directives: Vec<String>,
    hidden: bool,
}

fn render(directive: &Directive) -> String {
    match directive {
        Directive::HardStop => "hard_stop".to_string(),
        Directive::Length(seconds) => format!("length={}", seconds.get()),
        Directive::Tempo(bpm) => format!("bpm={}", bpm.get()),
    }
}

#[test]
fn shared_vectors_parse_as_written() -> Result<(), Box<dyn std::error::Error>> {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../testing/vectors/markers.json");
    let vectors: Vec<Vector> = serde_json::from_str(&std::fs::read_to_string(path)?)?;
    assert!(vectors.len() > 40);
    for vector in vectors {
        let parsed = MarkerName::parse(&vector.name);
        let got: Vec<String> = parsed.directives().iter().map(render).collect();
        assert_eq!(got, vector.directives, "directives of {:?}", vector.name);
        assert_eq!(
            parsed.is_hidden(),
            vector.hidden,
            "hidden of {:?}",
            vector.name
        );
    }
    Ok(())
}

fn secs(value: f64) -> Result<Seconds, shared_kernel::InvalidValue> {
    Seconds::new(value)
}

fn song(start: f64, end: f64) -> Result<Song, Box<dyn std::error::Error>> {
    Ok(Song::new(
        SongId::new("{A}"),
        "Song",
        secs(start)?,
        secs(end)?,
    )?)
}

#[test]
fn song_needs_a_window() -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        song(5.0, 5.0).err().map(|e| e.to_string()),
        Some("song ends at or before its start".into())
    );
    assert!(song(9.0, 5.0).is_err());
    assert!(Song::new(SongId::new("x"), "s", secs(0.0)?, secs(1.0)?)?.contains(secs(1.0)?));
    Ok(())
}

#[test]
fn only_cues_inside_the_window_count_edges_included() -> Result<(), Box<dyn std::error::Error>> {
    let s = song(10.0, 20.0)?;
    let cues = [
        Cue::new("!length:99", secs(9.99)?),
        Cue::new("!bpm:120", secs(10.0)?),
        Cue::new("!1008", secs(20.0)?),
        Cue::new("!bpm:90", secs(15.0)?),
        Cue::new("!length:7", secs(15.0)?),
        Cue::new("!length:8", secs(16.0)?),
        Cue::new("!bpm:60", secs(20.01)?),
    ];
    let found = s.directives(&cues);
    assert!(found.hard_stop());
    assert_eq!(found.tempo(), Some(Bpm::new(120.0)?));
    assert_eq!(found.length(), Some(secs(7.0)?));
    assert_eq!(s.length(&found), secs(7.0)?);
    Ok(())
}

#[test]
fn length_falls_back_to_the_region() -> Result<(), Box<dyn std::error::Error>> {
    let s = song(10.0, 25.5)?;
    let found = s.directives(&[Cue::new("Chorus", secs(12.0)?)]);
    assert_eq!(found, Directives::default());
    assert_eq!(s.length(&found), secs(15.5)?);
    Ok(())
}

#[test]
fn cue_keeps_its_name_and_hides_command_only_names() -> Result<(), Box<dyn std::error::Error>> {
    let cue = Cue::new("!1008 !length:3", secs(1.0)?);
    assert_eq!(cue.name(), "!1008 !length:3");
    assert!(cue.parsed().is_hidden());
    assert!(!Cue::new("Bridge", secs(1.0)?).parsed().is_hidden());
    Ok(())
}

#[test]
fn the_hard_stop_remembers_where_its_marker_lies() -> Result<(), Box<dyn std::error::Error>> {
    let cues = [
        Cue::new("!1008 !length:50", secs(45.0)?),
        Cue::new("!hardstop", secs(30.0)?),
    ];
    let directives = song(0.0, 60.0)?.directives(&cues);
    assert!(directives.hard_stop());
    assert_eq!(directives.hard_stop_at(), Some(secs(30.0)?));
    Ok(())
}
