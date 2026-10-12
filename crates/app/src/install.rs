//! Install.

mod action;
mod binary_architecture;
mod error;
mod extension;
mod item;
mod plan;
mod platform;
mod reaper;
mod report;
mod service;
mod stamp;
mod start;
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
pub use reaper::{
    ReaperInstall, endpoint_file, endpoint_file_from, extension_directory, fault_file,
    reaper_folder_argument,
};
pub use report::InstallReport;
pub use service::InstallationService;
pub use stamp::{stamp_matches, stamp_path, write_stamp};
pub use start::start_installation;
pub use status::InstallStatus;
pub use verification::InstallVerification;
pub use wizard::{check_list, wizard_step_list};
