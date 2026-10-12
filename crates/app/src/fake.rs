//! Fake.

mod extension;
mod fault_check;
mod midi_source;
mod mirror_repository;
mod stats_source;

pub use extension::FakeExtension;
pub use fault_check::FakeFaultCheck;
pub use midi_source::FakeMidiSource;
pub use mirror_repository::FakeMirrorRepository;
pub use stats_source::FakeStatsSource;
