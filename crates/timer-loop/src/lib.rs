//! The timer loop: one step of REAPER's timer, generic over the port, so the extension (real
//! REAPER) and the app's simulator (`FakeReaper`) run the same code.

mod timer_loop;

pub use timer_loop::TimerLoop;
