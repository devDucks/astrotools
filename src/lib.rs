//! The basic building blocks you need to build your next astrophotography suite.
//!
//! Astrotools provides traits and utils that can be used to implement
//! multiplatform drivers to drive astronomical equipment.
pub mod filter_wheel;
pub mod properties;
pub mod types;

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

pub trait Lightspeed {
    /// This method is used to synchronize the device state with the internal state of the driver.
    fn sync_state(&mut self);

    /// Method to be used when receving requests from clients to update properties.
    ///
    /// The internal logic would be a match on the prop_name that will then call prop.update_int(...),
    /// a method to update the value on the device itself, or both of them depending on the type
    /// of device.
    fn update_property<T>(&mut self, prop_name: &str, val: T) -> Result<(), LightspeedError>;
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
