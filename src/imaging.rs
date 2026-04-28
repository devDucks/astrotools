use std::sync::mpsc::Receiver;
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};

/// Exposure state machine for imaging devices (CCDs, CMOSes).
///
/// Embed this in device structs. In `tick()`, skip `sync_state()` while
/// `ReadingOut` to avoid USB contention errors during frame transfer.
pub enum ExposureState {
    Idle,
    /// Exposure in progress. `done` is set to `true` (Release ordering) by
    /// the monitor thread when the hardware signals completion.
    Exposing {
        done: Arc<AtomicBool>,
    },
    /// USB readout in progress. `rx` receives `(frame_bytes, pixel_count)`.
    ReadingOut {
        rx: Receiver<(Vec<u8>, usize)>,
    },
}

impl Default for ExposureState {
    fn default() -> Self {
        ExposureState::Idle
    }
}

impl ExposureState {
    pub fn is_idle(&self) -> bool {
        matches!(self, ExposureState::Idle)
    }

    pub fn is_exposing(&self) -> bool {
        matches!(self, ExposureState::Exposing { .. })
    }

    pub fn is_reading_out(&self) -> bool {
        matches!(self, ExposureState::ReadingOut { .. })
    }

    /// Returns `true` if the exposure monitor thread has signalled completion.
    /// Only meaningful when in the `Exposing` state.
    pub fn exposure_done(&self) -> bool {
        match self {
            ExposureState::Exposing { done } => done.load(Ordering::Acquire),
            _ => false,
        }
    }
}
