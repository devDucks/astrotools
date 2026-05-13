//! Topic builders and parsers for the lightspeed MQTT topic tree.
//!
//! Topic layout:
//!
//! ```text
//! devices/{device_uuid}                       device state, retained
//! devices/{device_uuid}/{action}              commands to a device
//! devices/{device_uuid}/status                per-device presence, retained
//! devices/{device_uuid}/frame                 raw science frame, NOT retained
//! devices/{device_uuid}/preview               framing/focus shot, NOT retained
//! runners/{runner_id}/status                  runner presence + LWT, retained
//! server/{area}/{action}                      server API endpoints
//! clients/{client_id}/replies/{correlation_id}  request/response reply topic
//! ```

use uuid::Uuid;

pub const DEVICES_PREFIX: &str = "devices";
pub const RUNNERS_PREFIX: &str = "runners";
pub const SERVER_PREFIX:  &str = "server";
pub const CLIENTS_PREFIX: &str = "clients";

pub const STATUS_SUFFIX:  &str = "status";
pub const FRAME_SUFFIX:   &str = "frame";
pub const PREVIEW_SUFFIX: &str = "preview";

pub fn device_state(uuid: Uuid) -> String {
    format!("{DEVICES_PREFIX}/{uuid}")
}

pub fn device_cmd(uuid: Uuid, action: &str) -> String {
    format!("{DEVICES_PREFIX}/{uuid}/{action}")
}

pub fn device_status(uuid: Uuid) -> String {
    format!("{DEVICES_PREFIX}/{uuid}/{STATUS_SUFFIX}")
}

pub fn device_frame(uuid: Uuid) -> String {
    format!("{DEVICES_PREFIX}/{uuid}/{FRAME_SUFFIX}")
}

pub fn device_preview(uuid: Uuid) -> String {
    format!("{DEVICES_PREFIX}/{uuid}/{PREVIEW_SUFFIX}")
}

pub fn runner_status(runner_id: Uuid) -> String {
    format!("{RUNNERS_PREFIX}/{runner_id}/{STATUS_SUFFIX}")
}

pub fn server_endpoint(area: &str, action: &str) -> String {
    format!("{SERVER_PREFIX}/{area}/{action}")
}

pub fn client_reply(client_id: &str, correlation_id: Uuid) -> String {
    format!("{CLIENTS_PREFIX}/{client_id}/replies/{correlation_id}")
}

/// Parsed device topic: returns `(device_uuid, action)`. `action` is empty
/// for plain `devices/{uuid}` state topics.
///
/// Returns `None` if the topic is not a device topic or the UUID is malformed.
pub fn parse_device_topic(topic: &str) -> Option<(Uuid, &str)> {
    let mut parts = topic.splitn(3, '/');
    if parts.next()? != DEVICES_PREFIX {
        return None;
    }
    let uuid = parts.next()?.parse::<Uuid>().ok()?;
    let action = parts.next().unwrap_or("");
    Some((uuid, action))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_device_topics() {
        let id = Uuid::nil();
        assert_eq!(device_state(id), "devices/00000000-0000-0000-0000-000000000000");
        assert_eq!(device_cmd(id, "expose"), "devices/00000000-0000-0000-0000-000000000000/expose");
        assert_eq!(device_status(id), "devices/00000000-0000-0000-0000-000000000000/status");
        assert_eq!(device_frame(id), "devices/00000000-0000-0000-0000-000000000000/frame");
        assert_eq!(device_preview(id), "devices/00000000-0000-0000-0000-000000000000/preview");
    }

    #[test]
    fn builds_runner_status() {
        let id = Uuid::nil();
        assert_eq!(runner_status(id), "runners/00000000-0000-0000-0000-000000000000/status");
    }

    #[test]
    fn parses_state_topic() {
        let id = Uuid::now_v7();
        let topic = device_state(id);
        let (parsed_id, action) = parse_device_topic(&topic).unwrap();
        assert_eq!(parsed_id, id);
        assert_eq!(action, "");
    }

    #[test]
    fn parses_cmd_topic() {
        let id = Uuid::now_v7();
        let topic = device_cmd(id, "expose");
        let (parsed_id, action) = parse_device_topic(&topic).unwrap();
        assert_eq!(parsed_id, id);
        assert_eq!(action, "expose");
    }

    #[test]
    fn parses_nested_action() {
        // Currently splitn(3) means action retains anything after the second '/'
        let id = Uuid::now_v7();
        let topic = format!("devices/{id}/sub/action");
        let (_, action) = parse_device_topic(&topic).unwrap();
        assert_eq!(action, "sub/action");
    }

    #[test]
    fn rejects_non_device_topic() {
        assert!(parse_device_topic("runners/abc/status").is_none());
        assert!(parse_device_topic("server/profiles/list").is_none());
    }

    #[test]
    fn rejects_bad_uuid() {
        assert!(parse_device_topic("devices/not-a-uuid/expose").is_none());
    }

    #[test]
    fn builds_client_reply() {
        let cid = Uuid::nil();
        let s = client_reply("client-1", cid);
        assert_eq!(s, "clients/client-1/replies/00000000-0000-0000-0000-000000000000");
    }
}
