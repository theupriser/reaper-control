//! Wire protocol between the REAPER extension and the app (SPEC §2.2).
//! The types here are the single source for the UI's TypeScript types (WP 0.4).

mod command;
mod configuration;
pub mod frame;
mod installation;
mod link;
pub mod message;
mod setlist;
mod song;
mod state;

#[cfg(test)]
mod generated;

pub use command::Command;
pub use configuration::ActionChoice;
pub use configuration::AppearanceChoice;
pub use configuration::ImportOffer;
pub use configuration::NoteMapping;
pub use configuration::Setting;
pub use configuration::Settings;
pub use configuration::SettingsView;
pub use installation::CheckId;
pub use installation::CheckItem;
pub use installation::CheckStatus;
pub use installation::ChecklistView;
pub use installation::InstallationView;
pub use installation::WizardStep;
pub use installation::WizardStepId;
pub use installation::WizardStepStatus;
pub use link::LinkProblem;
pub use link::LinkStatus;
pub use link::LinkView;
pub use setlist::EntryInfo;
pub use setlist::SetlistInfo;
pub use setlist::SetlistTransferView;
pub use song::Catalog;
pub use song::CueInfo;
pub use song::SongInfo;
pub use state::EventRecord;
pub use state::Live;
pub use state::Notice;
pub use state::NoticeLevel;
pub use state::Phase;
pub use state::SystemStats;
pub use state::Transport;
pub use state::WireEvent;
