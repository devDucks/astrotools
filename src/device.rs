use crate::LightspeedError;
use std::sync::mpsc::SyncSender;
use uuid::Uuid;

pub enum DeviceType {
    Ccd,
    Mount,
    Focuser,
    FilterWheel,
    PowerBox,
}

/// Unified lifecycle trait for all Lightspeed-compliant devices.
///
/// Implementors own an internal `mpsc::SyncSender<Command>` for routing MQTT
/// messages without exposing the concrete command type. The runner extracts a
/// dispatch closure via [`AstroDevice::dispatcher`] before moving the device
/// into its thread, then routes incoming MQTT payloads through that closure.
pub trait LightspeedDevice: Send + 'static {
    fn id(&self) -> Uuid;
    fn name(&self) -> &str;
    fn dev_type(&self) -> DeviceType;

    /// Use this namespace to build reproducible UUIDs based on some device
    /// attributes e.g. the device internal id.
    fn uuid_namespace(&self) -> Uuid {
        Uuid::parse_str("9e0c8dda-d62f-4cd0-a747-38b890496dbf").unwrap()
    }

    /// MQTT topic suffixes this device subscribes to, e.g. `&["expose", "set"]`.
    /// The runner prefixes each with `devices/{uuid}/`.
    fn command_topics(&self) -> &[&str];

    /// Serialize current state to JSON for MQTT publishing.
    fn state_json(&self) -> String;

    /// Return a closure that routes an MQTT message to this device's command channel.
    ///
    /// Call this **before** moving the device into its thread. The closure captures
    /// the internal `SyncSender` and is safe to call from any thread.
    fn dispatcher(&self) -> Box<dyn Fn(&str, &[u8]) -> Result<(), LightspeedError> + Send + Sync>;

    /// Called by the device thread on each tick interval.
    ///
    /// Implementations should:
    /// 1. Drain the internal command queue via `try_recv()`
    /// 2. Sync hardware state (skip during `ReadingOut` to avoid USB contention)
    /// 3. Advance exposure/operation state machine if applicable
    /// 4. Push current state: `state_tx.try_send((self.id(), self.state_json()))`
    fn tick(&mut self, state_tx: &SyncSender<(Uuid, String)>);

    /// Clean shutdown. Called by the device thread before it exits.
    fn close(&mut self);
}
