//! The basic building blocks you need to build your next astrophotography suite.
//!
//! Astrotools provides traits and utils that can be used to implement
//! multiplatform drivers to drive astronomical equipment, plus the wire-format
//! types that the server side of the lightspeed protocol consumes.

// Always available (wire feature is enabled by both `driver` and `server`)
pub mod properties;

#[cfg(feature = "wire")]
pub mod protocol;
#[cfg(feature = "wire")]
pub mod presence;
#[cfg(feature = "wire")]
pub mod frame;
#[cfg(feature = "wire")]
pub mod topics;

// Driver-only modules
#[cfg(feature = "driver")]
pub mod device;
#[cfg(feature = "driver")]
pub mod filter_wheel;
#[cfg(feature = "driver")]
pub mod imaging;
#[cfg(feature = "driver")]
pub mod runner;
#[cfg(feature = "driver")]
mod serial;

#[cfg(feature = "driver")]
pub use crate::serial::find_serial_devices;

use serde::{Serialize, Serializer};

fn io_serialize<S>(err: &std::io::Error, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let kind = err.kind();
    serializer.serialize_str(&kind.to_string())
}

#[derive(Debug, Serialize)]
pub enum LightspeedError {
    PropertyError(properties::PropertyErrorType),
    #[serde(serialize_with = "io_serialize")]
    IoError(std::io::Error),
    DeviceConnectionError,
    UnknownCommand,
    QueueFull,
    ParseError,
}

impl From<properties::PropertyErrorType> for LightspeedError {
    fn from(error: properties::PropertyErrorType) -> Self {
        LightspeedError::PropertyError(error)
    }
}

impl From<std::io::Error> for LightspeedError {
    fn from(error: std::io::Error) -> Self {
        LightspeedError::IoError(error)
    }
}

#[cfg(feature = "driver")]
pub trait Lightspeed {
    fn sync_state(&mut self);
    fn update_property(
        &mut self,
        prop_name: &str,
        val: properties::PropValue,
    ) -> Result<(), LightspeedError>;
}

#[cfg(test)]
mod tests {
    use crate::LightspeedError;
    use std::io::{Error, ErrorKind};

    #[test]
    fn test_serialize_lightspeed_error() {
        let custom_error_1 = Error::new(ErrorKind::Other, "oh no!");
        let custom_error_2 = Error::from(ErrorKind::NotConnected);
        let e1 = LightspeedError::IoError(custom_error_1);
        let e2 = LightspeedError::IoError(custom_error_2);
        assert_eq!(
            "{\"IoError\":\"other error\"}",
            serde_json::to_string(&e1).unwrap()
        );
        assert_eq!(
            "{\"IoError\":\"not connected\"}",
            serde_json::to_string(&e2).unwrap()
        );
    }
}
