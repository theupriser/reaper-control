//! Installing the REAPER extension: the plan, the items, the setup wizard's steps and the checks
//! that need no app state. Pure of Tauri and of the app's config.

mod action;
mod binary_architecture;
mod error;
mod extension;
mod item;
mod plan;
mod platform;
mod process;
mod reaper;
mod report;
mod service;
mod stamp;
mod status;
mod verification;
mod wizard;

pub use action::InstallAction;
pub use binary_architecture::BinaryArchitecture;
pub use error::InstallError;
pub use extension::{
    ExtensionInstaller, ExtensionPreparer, MacExtensionPreparer, PlainExtensionPreparer,
    bundled_extension,
};
pub use item::{InstallItem, InstallItemKind};
pub use plan::InstallPlan;
pub use platform::InstallPlatform;
pub use process::{FakeProcessCheck, ProcessCheck, SystemProcessCheck};
pub use reaper::{
    ReaperInstall, endpoint_file, endpoint_file_from, extension_directory, fault_file,
    reaper_folder_argument,
};
pub use report::InstallReport;
pub use service::InstallationService;
pub use stamp::{stamp_matches, stamp_path, write_stamp};
pub use status::InstallStatus;
pub use verification::InstallVerification;
pub use wizard::{check_list, wizard_step_list};
