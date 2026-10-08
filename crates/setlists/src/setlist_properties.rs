#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::collections::HashSet;

use proptest::prelude::*;
use shared_kernel::SongId;

use crate::{Edit, EntryId, Revision, Setlist, SetlistEvent, SetlistId};

fn edit() -> impl Strategy<Value = Edit> {
    prop_oneof![
        3 => (0u8..4, proptest::option::of(0usize..12))
            .prop_map(|(song, at)| Edit::Add { song: SongId::new(format!("s{song}")), at }),
        2 => (0u64..14).prop_map(|id| Edit::Remove(EntryId::new(id))),
        3 => (0u64..14, 0usize..12)
            .prop_map(|(id, to)| Edit::Move { entry: EntryId::new(id), to }),
        1 => "[ a-c]{0,3}".prop_map(Edit::Rename),
    ]
}

fn ids(list: &Setlist) -> Vec<EntryId> {
    list.entries().iter().map(|entry| entry.id()).collect()
}

proptest! {
    #![proptest_config(ProptestConfig::with_cases(2000))]

    #[test]
    fn edits_keep_the_setlist_valid(
        steps in proptest::collection::vec((edit(), any::<bool>()), 0..80),
    ) {
        let mut list = Setlist::new(SetlistId::new("p"), "Start").unwrap();
        let mut ever_used = HashSet::new();
        for (edit, fresh) in steps {
            let before = list.clone();
            let expected = if fresh {
                list.revision()
            } else {
                Revision::new(list.revision().get() + 1)
            };
            match list.edit(expected, edit) {
                Ok(event) => {
                    prop_assert!(fresh);
                    prop_assert_eq!(list.revision().get(), before.revision().get() + 1);
                    prop_assert!(list.entries() != before.entries() || list.name() != before.name());
                    if let SetlistEvent::EntryAdded { entry, at, .. } = event {
                        prop_assert!(ever_used.insert(entry), "entry id reused");
                        let placed = ids(&list);
                        prop_assert_eq!(placed.get(at), Some(&entry));
                    }
                }
                Err(_) => {
                    prop_assert_eq!(&list, &before);
                }
            }
            let now = ids(&list);
            prop_assert_eq!(now.iter().collect::<HashSet<_>>().len(), now.len());
            prop_assert!(!list.name().is_empty());
            prop_assert_eq!(list.name(), list.name().trim());
        }
    }

    #[test]
    fn moves_and_adds_and_removes_only_change_what_they_say(
        steps in proptest::collection::vec(edit(), 0..60),
    ) {
        let mut list = Setlist::new(SetlistId::new("p"), "Start").unwrap();
        for edit in steps {
            let before = ids(&list);
            let Ok(event) = list.edit(list.revision(), edit) else { continue };
            let after = ids(&list);
            let mut expected = before.clone();
            match event {
                SetlistEvent::Renamed { .. } => {}
                SetlistEvent::EntryAdded { entry, at, .. } => expected.insert(at, entry),
                SetlistEvent::EntryRemoved { entry } => expected.retain(|e| *e != entry),
                SetlistEvent::EntryMoved { entry, from, to } => {
                    prop_assert_eq!(before.get(from), Some(&entry));
                    let moved = expected.remove(from);
                    expected.insert(to, moved);
                }
            }
            prop_assert_eq!(after, expected);
        }
    }
}
