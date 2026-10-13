//! Starting the long-lived parts: the installation service and the link, and telling the extension
//! where the app is.

mod application_location;
mod installation;
mod link;

pub use application_location::{application_from, record_application_location};
pub use installation::start_installation;
pub use link::start_link;
