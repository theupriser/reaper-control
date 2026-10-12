//! Starting the long-lived parts: the installation service and the link.

mod installation;
mod link;

pub use installation::start_installation;
pub use link::start_link;
