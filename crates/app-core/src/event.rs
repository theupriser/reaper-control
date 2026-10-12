//! Event.

mod app;
mod bus;
mod fault;
mod logger;
mod notice;

pub use app::AppEvent;
pub use bus::EventBus;
pub use fault::AppFault;
pub use logger::log_event;
pub use notice::{link_problem, notice_for_event};
