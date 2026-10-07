//! The link between the REAPER extension and the app (SPEC §2.2, ADR-003): loopback TCP,
//! plain threads, no async runtime. [`LinkServer`] runs in the extension, [`LinkClient`] in the app.

mod client;
mod endpoint;
mod server;
mod wire;

pub use client::{ClientConfig, LinkClient, LinkEvent, SendError};
pub use endpoint::{Endpoint, EndpointError};
pub use server::{CommandHandler, LinkServer, ServerError};
