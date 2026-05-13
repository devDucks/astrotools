//! Binary frame header used on the WebSocket channel between camera drivers
//! and the lightspeed server.
//!
//! Wire format on the WebSocket:
//!
//! 1. 4-byte big-endian `u32` header length `N`
//! 2. `N` bytes of JSON-encoded [`FrameHeader`]
//! 3. Raw image bytes (length implied by width * height * bit_depth/8 and the
//!    sender's framing — astrotools does not prescribe transport-level framing)
//!
//! The framing scheme itself is implemented in lightspeed-server and in
//! camera drivers; this type only defines the header payload.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrameHeader {
    /// The `Command.id` of the originating expose command. The server uses
    /// this to pair the frame with the device-state snapshot it took at
    /// exposure start.
    pub correlation_id: Uuid,
    pub driver_id: Uuid,
    pub width: u32,
    pub height: u32,
    pub bit_depth: u8,
    /// Bayer pattern, e.g. `"RGGB"`. None for mono sensors.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bayer: Option<String>,
    /// Shutter-open timestamp, nanoseconds since unix epoch. Driver-supplied.
    pub timestamp_ns: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn frame_header_roundtrip() {
        let h = FrameHeader {
            correlation_id: Uuid::now_v7(),
            driver_id: Uuid::now_v7(),
            width: 6248,
            height: 4176,
            bit_depth: 16,
            bayer: Some("RGGB".into()),
            timestamp_ns: 1_700_000_000_000_000_000,
        };
        let json = serde_json::to_string(&h).unwrap();
        let back: FrameHeader = serde_json::from_str(&json).unwrap();
        assert_eq!(back.width, 6248);
        assert_eq!(back.bayer.as_deref(), Some("RGGB"));
    }

    #[test]
    fn mono_frame_omits_bayer() {
        let h = FrameHeader {
            correlation_id: Uuid::now_v7(),
            driver_id: Uuid::now_v7(),
            width: 1280,
            height: 960,
            bit_depth: 16,
            bayer: None,
            timestamp_ns: 0,
        };
        let json = serde_json::to_string(&h).unwrap();
        assert!(!json.contains("bayer"));
    }
}
