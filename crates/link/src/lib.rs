//! The link between the REAPER extension and the app (SPEC §2.2, ADR-003): loopback TCP,
//! plain threads, no async runtime. [`LinkServer`] runs in the extension, [`LinkClient`] in the app.

mod client;
mod endpoint;
mod endpoint_error;
mod read_error;
mod reader;
mod server;

pub use client::{ClientConfig, LinkClient, LinkEvent, SendError};
pub use endpoint::Endpoint;
pub use endpoint_error::EndpointError;
pub use server::{CommandHandler, LinkServer, ServerError};
