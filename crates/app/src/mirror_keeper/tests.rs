use protocol::{Catalog, EntryInfo, LinkView, SetlistInfo};

use super::*;
use crate::fake_mirror_repository::FakeMirrorRepository;

fn setlist(songs: usize) -> SetlistInfo {
    SetlistInfo {
        id: "s".into(),
        name: "Friday".into(),
        revision: 1,
        entries: (0..songs as u64)
            .map(|id| EntryInfo {
                id,
                song_id: format!("{{{id}}}"),
            })
            .collect(),
    }
}

fn view(project_id: &str, setlists: Vec<SetlistInfo>) -> LinkView {
    LinkView {
        catalog: Box::new(Catalog {
            project_id: project_id.into(),
            setlists,
            ..Catalog::default()
        }),
        ..LinkView::default()
    }
}

fn keeper() -> (MirrorKeeper, Arc<FakeMirrorRepository>) {
    let repository = Arc::new(FakeMirrorRepository::default());
    (MirrorKeeper::new(repository.clone()), repository)
}

#[test]
fn a_change_is_written_once_and_an_unchanged_view_is_not_written_again() {
    let (keeper, repository) = keeper();
    keeper.observe(&view("project-1", vec![setlist(1)]));
    keeper.observe(&view("project-1", vec![setlist(1)]));
    assert_eq!(repository.writes(), 1);
    keeper.observe(&view("project-1", vec![setlist(2)]));
    assert_eq!(repository.writes(), 2);
    assert_eq!(repository.restore("project-1").ok(), Some(vec![setlist(2)]));
}

#[test]
fn an_empty_list_or_an_unknown_project_never_overwrites_the_copy() {
    let (keeper, repository) = keeper();
    keeper.observe(&view("project-1", vec![setlist(1)]));
    keeper.observe(&view("project-1", Vec::new()));
    keeper.observe(&view("", vec![setlist(3)]));
    assert_eq!(repository.writes(), 1);
    assert_eq!(repository.restore("project-1").ok(), Some(vec![setlist(1)]));
}

#[test]
fn a_failed_write_is_tried_again_with_the_next_view() {
    let (keeper, repository) = keeper();
    repository.refuse_writes(true);
    keeper.observe(&view("project-1", vec![setlist(1)]));
    assert_eq!(repository.writes(), 0);
    repository.refuse_writes(false);
    keeper.observe(&view("project-1", vec![setlist(1)]));
    assert_eq!(repository.writes(), 1);
}
