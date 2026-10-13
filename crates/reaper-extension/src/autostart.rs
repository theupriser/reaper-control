//! Starting the app when a project that asks for it opens.

mod autostarter;
mod launcher;
mod process_launcher;

pub use autostarter::Autostarter;
pub use launcher::Launcher;
pub use process_launcher::ProcessLauncher;

#[cfg(test)]
mod tests;
