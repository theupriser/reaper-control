//! Installation.

mod check;
mod view;
mod wizard;

pub use check::{CheckId, CheckItem, CheckStatus, ChecklistView};
pub use view::InstallationView;
pub use wizard::{WizardStep, WizardStepId, WizardStepStatus};
