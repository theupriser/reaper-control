//! Fake.

mod clock;
mod config_repository;
mod driver;
mod extension;
mod fault_check;
mod midi_source;
mod mirror_repository;
mod stats_source;

pub use clock::FakeClock;
pub use config_repository::FakeConfigRepository;
pub use driver::FakeDriver;
pub use extension::FakeExtension;
pub use fault_check::FakeFaultCheck;
pub use midi_source::FakeMidiSource;
pub use mirror_repository::FakeMirrorRepository;
pub use stats_source::FakeStatsSource;
