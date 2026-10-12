use protocol::Command;

use super::*;

fn seek(position: f64) -> Command {
    Command::Seek {
        position,
        count_in: false,
    }
}

fn save(id: &str, name: &str) -> Command {
    Command::SaveSetlist {
        id: id.into(),
        name: name.into(),
        entries: Vec::new(),
        expected_revision: 0,
    }
}

#[test]
fn a_seek_needs_a_position_that_is_a_number_and_not_negative() {
    assert_eq!(check(&seek(0.0)), Ok(()));
    assert_eq!(check(&seek(12.5)), Ok(()));
    for bad in [-0.1, f64::NAN, f64::INFINITY] {
        assert_eq!(check(&seek(bad)), Err(CommandRefusal::InvalidPosition));
    }
}

#[test]
fn a_setlist_needs_an_id_and_a_name() {
    assert_eq!(check(&save("friday", "Friday")), Ok(()));
    assert_eq!(
        check(&save(" ", "Friday")),
        Err(CommandRefusal::EmptySetlistId)
    );
    assert_eq!(
        check(&save("friday", "")),
        Err(CommandRefusal::EmptySetlistName)
    );
}

#[test]
fn the_other_commands_always_pass() {
    for command in [
        Command::Play,
        Command::Next,
        Command::SetActiveSetlist { id: None },
    ] {
        assert_eq!(check(&command), Ok(()));
    }
}
