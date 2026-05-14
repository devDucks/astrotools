//! Presence contract for lightspeed drivers and runners.
//!
//! Two retained topics carry presence state:
//!
//! - `devices/{uuid}/status` — per-device, payload [`DeviceStatus`]. Published
//!   by the driver runner on connect (Online) and on graceful shutdown (Offline).
//! - `runners/{runner_id}/status` — per-runner-connection, payload
//!   [`RunnerStatus`]. Set as the MQTT LWT so it flips to Offline if the
//!   runner crashes or loses network. Also published on graceful shutdown.
//!
//! Server-side reconciliation: a device is effectively Online iff its own
//! `DeviceStatus` is Online AND its owning runner's `RunnerStatus` is Online.
//! Stale per-device retained Online statuses are overridden by a runner
//! Offline LWT.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum PresenceState {
    Online,
    Offline,
}

/// Per-device presence status. Published retained to `devices/{uuid}/status`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    pub state: PresenceState,
    /// The runner process that owns this device. Server uses this to look up
    /// `RunnerStatus` for reconciliation.
    pub runner_id: Uuid,
    /// Unix epoch seconds of when the runner came online.
    pub started_at: u64,
    /// Driver crate version, e.g. `"0.4.1"`.
    pub driver_version: String,
    pub pid: u32,
}

/// Per-runner presence status. Published retained to `runners/{runner_id}/status`.
/// Used as the LWT payload so a crashed runner is detected within ~1.5x keepalive.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerStatus {
    pub state: PresenceState,
    /// Every device UUID hosted by this runner. The server uses this list to
    /// mark all hosted devices Offline when a runner LWT fires.
    pub device_uuids: Vec<Uuid>,
    pub started_at: u64,
    pub runner_version: String,
    pub pid: u32,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_status_roundtrip() {
        let s = DeviceStatus {
            state: PresenceState::Online,
            runner_id: Uuid::now_v7(),
            started_at: 1_700_000_000,
            driver_version: "0.4.1".into(),
            pid: 1234,
        };
        let json = serde_json::to_string(&s).unwrap();
        let back: DeviceStatus = serde_json::from_str(&json).unwrap();
        assert_eq!(back.state, PresenceState::Online);
        assert_eq!(back.driver_version, "0.4.1");
    }

    #[test]
    fn runner_status_shape() {
        let s = RunnerStatus {
            state: PresenceState::Offline,
            device_uuids: vec![Uuid::now_v7(), Uuid::now_v7()],
            started_at: 1_700_000_000,
            runner_version: "0.12.0".into(),
            pid: 1234,
        };
        let json = serde_json::to_string(&s).unwrap();
        assert!(json.contains(r#""state":"offline""#));
        assert!(json.contains(r#""device_uuids":["#));
    }
}
