//! On-the-wire protocol envelopes used by lightspeed-compliant clients and servers.
//!
//! These types are transport-agnostic but in practice are serialized as JSON
//! and exchanged over MQTT. The server uses them to issue commands and parse
//! replies; drivers use them when responding to commands and emitting frames.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Generate a new correlation id (uuid v7, time-sortable).
pub fn new_correlation_id() -> Uuid {
    Uuid::now_v7()
}

/// A command issued by a client (typically the server) to a driver.
///
/// `id` is a fresh correlation id. `parent_id` is set when this command is a
/// retry or a follow-up to a previous command (e.g. re-exposure after a
/// failed filter change inside a sequence step).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Command<T> {
    pub id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_id: Option<Uuid>,
    pub payload: T,
}

impl<T> Command<T> {
    pub fn new(payload: T) -> Self {
        Self {
            id: new_correlation_id(),
            parent_id: None,
            payload,
        }
    }

    pub fn with_parent(payload: T, parent_id: Uuid) -> Self {
        Self {
            id: new_correlation_id(),
            parent_id: Some(parent_id),
            payload,
        }
    }
}

/// A reply to a `Command`. `correlation_id` always equals the original
/// `Command.id`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Reply<T> {
    pub correlation_id: Uuid,
    #[serde(flatten)]
    pub result: ReplyResult<T>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "snake_case")]
pub enum ReplyResult<T> {
    Ok { data: T },
    Error { error: ErrorEnvelope },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorEnvelope {
    pub code: ErrorCode,
    pub message: String,
    /// For validation errors, the field name that failed validation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub field: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    NotFound,
    Validation,
    Conflict,
    DriverUnavailable,
    ActivationFailed,
    Internal,
}

impl<T> Reply<T> {
    pub fn ok(correlation_id: Uuid, data: T) -> Self {
        Self {
            correlation_id,
            result: ReplyResult::Ok { data },
        }
    }

    pub fn error(correlation_id: Uuid, code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            correlation_id,
            result: ReplyResult::Error {
                error: ErrorEnvelope {
                    code,
                    message: message.into(),
                    field: None,
                },
            },
        }
    }

    pub fn validation_error(
        correlation_id: Uuid,
        field: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self {
            correlation_id,
            result: ReplyResult::Error {
                error: ErrorEnvelope {
                    code: ErrorCode::Validation,
                    message: message.into(),
                    field: Some(field.into()),
                },
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Debug, Serialize, Deserialize, PartialEq)]
    struct ExposePayload {
        duration_ms: u32,
    }

    #[test]
    fn command_roundtrip() {
        let cmd = Command::new(ExposePayload { duration_ms: 30_000 });
        let json = serde_json::to_string(&cmd).unwrap();
        let decoded: Command<ExposePayload> = serde_json::from_str(&json).unwrap();
        assert_eq!(decoded.id, cmd.id);
        assert!(decoded.parent_id.is_none());
        assert_eq!(decoded.payload, cmd.payload);
    }

    #[test]
    fn reply_ok_shape() {
        let id = new_correlation_id();
        let reply: Reply<u32> = Reply::ok(id, 42);
        let json = serde_json::to_string(&reply).unwrap();
        assert!(json.contains(r#""status":"ok""#));
        assert!(json.contains(r#""data":42"#));
    }

    #[test]
    fn reply_error_shape() {
        let id = new_correlation_id();
        let reply: Reply<()> = Reply::error(id, ErrorCode::NotFound, "device missing");
        let json = serde_json::to_string(&reply).unwrap();
        assert!(json.contains(r#""status":"error""#));
        assert!(json.contains(r#""code":"not_found""#));
    }

    #[test]
    fn validation_error_has_field() {
        let id = new_correlation_id();
        let reply: Reply<()> =
            Reply::validation_error(id, "latitude", "must be between -90 and 90");
        let json = serde_json::to_string(&reply).unwrap();
        assert!(json.contains(r#""field":"latitude""#));
    }

    #[test]
    fn correlation_ids_are_v7_and_unique() {
        let a = new_correlation_id();
        let b = new_correlation_id();
        assert_ne!(a, b);
        assert_eq!(a.get_version_num(), 7);
    }
}
