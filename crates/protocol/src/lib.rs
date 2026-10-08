//! Wire protocol between the REAPER extension and the app (SPEC §2.2).
//! The types here are the single source for the UI's TypeScript types (WP 0.4).

mod catalog;
mod command;
mod cue_info;
mod entry_info;
mod event_record;
pub mod frame;
mod import_offer;
mod link_status;
mod link_view;
mod live;
pub mod message;
mod note_mapping;
mod notice;
mod notice_level;
mod phase;
mod setlist_info;
mod setlist_transfer_view;
mod setting;
mod settings;
mod settings_view;
mod song_info;
mod transport;
mod wire_event;

#[cfg(test)]
mod generated;

pub use catalog::Catalog;
pub use command::Command;
pub use cue_info::CueInfo;
pub use entry_info::EntryInfo;
pub use event_record::EventRecord;
pub use import_offer::ImportOffer;
pub use link_status::LinkStatus;
pub use link_view::LinkView;
pub use live::Live;
pub use note_mapping::NoteMapping;
pub use notice::Notice;
pub use notice_level::NoticeLevel;
pub use phase::Phase;
pub use setlist_info::SetlistInfo;
pub use setlist_transfer_view::SetlistTransferView;
pub use setting::Setting;
pub use settings::Settings;
pub use settings_view::SettingsView;
pub use song_info::SongInfo;
pub use transport::Transport;
pub use wire_event::WireEvent;
