//! Fake.

mod extension;
mod fault_check;
mod mirror_repository;
mod stats_source;

pub use extension::FakeExtension;
pub use fault_check::FakeFaultCheck;
pub use mirror_repository::FakeMirrorRepository;
pub use stats_source::FakeStatsSource;
