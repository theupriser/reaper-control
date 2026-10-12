//! Test doubles for the core's traits.

mod clock;
mod config_repository;
mod driver;

pub use clock::FakeClock;
pub use config_repository::FakeConfigRepository;
pub use driver::FakeDriver;
