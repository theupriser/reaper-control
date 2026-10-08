use std::io::{Read, Write};
use std::net::TcpStream;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use protocol::frame::{FrameDecoder, encode};

use crate::chaos_settings::ChaosSettings;

/// Bytes that no frame decoder accepts: a length far above the largest frame.
const GARBAGE: [u8; 8] = [0xFF; 8];

/// Passes the server's frames on to the app, one at a time, with the current faults applied.
pub(super) struct FrameRelay {
    settings: Arc<Mutex<ChaosSettings>>,
    frames: u32,
    garbled: bool,
    held: Option<Vec<u8>>,
}

impl FrameRelay {
    pub(super) fn new(settings: Arc<Mutex<ChaosSettings>>) -> Self {
        Self {
            settings,
            frames: 0,
            garbled: false,
            held: None,
        }
    }

    /// Runs until either side closes, a fault cuts the connection or `stop` is set.
    pub(super) fn run(mut self, mut from: &TcpStream, mut to: &TcpStream, stop: &AtomicBool) {
        let mut decoder = FrameDecoder::new();
        let mut chunk = [0u8; 4096];
        while !stop.load(Ordering::SeqCst) {
            match from.read(&mut chunk) {
                Ok(0) => return,
                Ok(count) => decoder.push(chunk.get(..count).unwrap_or_default()),
                Err(error)
                    if matches!(
                        error.kind(),
                        std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                    ) => {}
                Err(_) => return,
            }
            while let Ok(Some(payload)) = decoder.next_frame() {
                if !self.pass_on(&payload, &mut to) {
                    return;
                }
            }
        }
    }

    /// False when the connection ends here.
    fn pass_on(&mut self, payload: &[u8], to: &mut &TcpStream) -> bool {
        let settings = self
            .settings
            .lock()
            .map(|settings| settings.clone())
            .unwrap_or_default();
        self.frames += 1;
        if settings
            .cut_after_frames
            .is_some_and(|cut| self.frames > cut)
        {
            return false;
        }
        if !self.garbled
            && settings
                .garbage_after_frames
                .is_some_and(|after| self.frames >= after)
        {
            self.garbled = true;
            if to.write_all(&GARBAGE).is_err() {
                return false;
            }
        }
        thread::sleep(settings.delay);
        let Ok(frame) = encode(payload) else {
            return false;
        };
        let sent = if settings.swap_neighbours {
            match self.held.take() {
                Some(earlier) => to.write_all(&frame).and_then(|()| to.write_all(&earlier)),
                None => {
                    self.held = Some(frame);
                    Ok(())
                }
            }
        } else {
            let earlier = self.held.take().unwrap_or_default();
            to.write_all(&earlier).and_then(|()| to.write_all(&frame))
        };
        sent.is_ok()
    }
}
