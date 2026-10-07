#![allow(clippy::unwrap_used, clippy::expect_used)]

use shared_kernel::SongId;

use super::*;

fn song(name: &str) -> SongId {
    SongId::new(name)
}

fn setlist() -> Setlist {
    Setlist::new(SetlistId::new("main"), "Main set").unwrap()
}

fn add(list: &mut Setlist, name: &str, at: Option<usize>) -> EntryId {
    let edit = Edit::Add {
        song: song(name),
        at,
    };
    match list.edit(list.rev(), edit).unwrap() {
        SetlistEvent::EntryAdded { entry, .. } => entry,
        other => panic!("unexpected {other:?}"),
    }
}

fn names(list: &Setlist) -> Vec<&str> {
    list.songs().map(SongId::as_str).collect()
}

#[test]
fn a_setlist_needs_a_name() {
    assert_eq!(
        Setlist::new(SetlistId::new("x"), "  ").err(),
        Some(InvalidSetlist::EmptyName)
    );
    assert_eq!(setlist().name(), "Main set");
    assert_eq!(setlist().rev(), Revision::INITIAL);
}

#[test]
fn add_inserts_and_appends() {
    let mut list = setlist();
    add(&mut list, "a", None);
    add(&mut list, "c", None);
    add(&mut list, "b", Some(1));
    assert_eq!(names(&list), ["a", "b", "c"]);
    assert_eq!(list.rev(), Revision::new(3));
}

#[test]
fn entry_ids_are_not_reused() {
    let mut list = setlist();
    let first = add(&mut list, "a", None);
    list.edit(list.rev(), Edit::Remove(first)).unwrap();
    assert_ne!(add(&mut list, "a", None), first);
}

#[test]
fn move_puts_the_entry_at_the_index_of_the_result() {
    let mut list = setlist();
    let a = add(&mut list, "a", None);
    add(&mut list, "b", None);
    add(&mut list, "c", None);
    let event = list.edit(list.rev(), Edit::Move { entry: a, to: 2 });
    assert_eq!(
        event,
        Ok(SetlistEvent::EntryMoved {
            entry: a,
            from: 0,
            to: 2
        })
    );
    assert_eq!(names(&list), ["b", "c", "a"]);
}

#[test]
fn refused_edits_change_nothing() {
    let mut list = setlist();
    let a = add(&mut list, "a", None);
    let before = list.clone();
    let rev = list.rev();
    let cases = [
        (Edit::Rename(" ".into()), Rejection::EmptyName),
        (Edit::Rename("Main set".into()), Rejection::NoChange),
        (Edit::Remove(EntryId::new(99)), Rejection::UnknownEntry),
        (
            Edit::Add {
                song: song("x"),
                at: Some(2),
            },
            Rejection::PositionOutOfRange,
        ),
        (
            Edit::Move { entry: a, to: 1 },
            Rejection::PositionOutOfRange,
        ),
        (Edit::Move { entry: a, to: 0 }, Rejection::NoChange),
    ];
    for (edit, why) in cases {
        assert_eq!(list.edit(rev, edit), Err(why));
        assert_eq!(list, before);
    }
}

#[test]
fn a_stale_edit_is_refused() {
    let mut list = setlist();
    add(&mut list, "a", None);
    let edit = Edit::Rename("Other".into());
    assert_eq!(
        list.edit(Revision::INITIAL, edit),
        Err(Rejection::Stale {
            expected: Revision::INITIAL,
            actual: Revision::new(1)
        })
    );
    assert_eq!(list.name(), "Main set");
}

#[test]
fn rename_trims() {
    let mut list = setlist();
    let event = list.edit(list.rev(), Edit::Rename("  Encore ".into()));
    assert_eq!(
        event,
        Ok(SetlistEvent::Renamed {
            name: "Encore".into()
        })
    );
    assert_eq!(list.name(), "Encore");
}

#[test]
fn dangling_entries_are_reported_not_removed() {
    let mut list = setlist();
    add(&mut list, "a", None);
    let gone = add(&mut list, "b", None);
    assert_eq!(list.dangling(|id| id.as_str() == "a"), [gone]);
    assert_eq!(names(&list), ["a", "b"]);
}

#[test]
fn restore_checks_ids_and_continues_after_the_highest() {
    let entry = |id| Entry::new(EntryId::new(id), song("s"));
    let duplicate = Setlist::restore(
        SetlistId::new("x"),
        "x",
        vec![entry(1), entry(1)],
        Revision::INITIAL,
    );
    assert_eq!(duplicate.err(), Some(InvalidSetlist::DuplicateEntry));
    let mut list = Setlist::restore(
        SetlistId::new("x"),
        "x",
        vec![entry(7), entry(2)],
        Revision::new(4),
    )
    .unwrap();
    assert_eq!(add(&mut list, "n", None), EntryId::new(8));
    assert_eq!(list.rev(), Revision::new(5));
}

#[test]
fn repository_saves_optimistically() {
    let mut repo = InMemorySetlistRepository::new();
    let mut list = setlist();
    assert_eq!(repo.save(list.clone(), None), Ok(()));
    assert_eq!(
        repo.save(list.clone(), None),
        Err(SaveError::Stale {
            expected: None,
            actual: Some(Revision::INITIAL)
        })
    );
    add(&mut list, "a", None);
    assert_eq!(repo.save(list.clone(), Some(Revision::INITIAL)), Ok(()));
    assert_eq!(
        repo.save(list.clone(), Some(Revision::INITIAL)),
        Err(SaveError::Stale {
            expected: Some(Revision::INITIAL),
            actual: Some(Revision::new(1)),
        })
    );
    assert_eq!(repo.load(list.id()), Some(list.clone()));
    assert_eq!(repo.list().len(), 1);
}

#[test]
fn repository_deletes_only_the_expected_revision() {
    let mut repo = InMemorySetlistRepository::new();
    let list = setlist();
    repo.save(list.clone(), None).unwrap();
    assert!(repo.delete(list.id(), Revision::new(3)).is_err());
    assert_eq!(repo.delete(list.id(), Revision::INITIAL), Ok(()));
    assert_eq!(repo.load(list.id()), None);
    assert!(repo.delete(list.id(), Revision::INITIAL).is_err());
}
