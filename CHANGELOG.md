# Changelog

## 0.12.0

### Added
- Cargo features `wire`, `driver`, `server`, `full`. Default is `driver` for
  backward compatibility.
- `protocol` module (wire): `Command<T>`, `Reply<T>`, `ReplyResult<T>`,
  `ErrorEnvelope`, `ErrorCode`, `new_correlation_id()` (uuid v7).
- `presence` module (wire): `DeviceStatus`, `RunnerStatus`, `PresenceState`.
- `frame` module (wire): `FrameHeader` for WebSocket binary frame transport.
- `topics` module (wire): topic builders and `parse_device_topic` parser.
- `DeviceType` variants: `Dome`, `Weather`, `AuxBox`.

### Changed
- Runner publishes retained per-device `DeviceStatus` on connect and graceful
  shutdown.
- Runner publishes retained `RunnerStatus` on connect, sets LWT for crash
  detection, publishes graceful Offline on shutdown.
- MQTT keepalive default lowered from 30 s to 15 s. Detection latency ~22 s.
- Runner topic parsing rewritten on top of `topics::parse_device_topic`;
  byte-offset slicing removed.
- `RunnerConfig` adds `driver_version` and `keepalive_secs` fields.
- `uuid` dependency gains `v7` and `serde` features.

### Notes
- MQTT v3.1.1 is unchanged.
- Existing drivers (qhy-rs, pegasus-rs, eqmount-simulator) need to pass
  `driver_version: env!("CARGO_PKG_VERSION").to_string()` in `RunnerConfig`.
  This is the only required source change.
