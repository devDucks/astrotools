//! The basic building blocks you need to build your next astrophotography suite.
//!
//! Astrotools provides traits and utils that can be used to implement
//! multiplatform drivers to drive astronomical equipment.
//use uuid::Uuid;

pub mod filter_wheel;
pub mod properties;
pub mod types;

pub trait Lightspeed {
    /// This method is used to synchronize the device state with the internal state of the driver.
    fn sync_state(&mut self);

    /// Method to be used when receving requests from clients to update properties.
    ///
    /// The internal logic would be a match on the prop_name that will then call prop.update_int(...),
    /// a method to update the value on the device itself, or both of them depending on the type
    /// of device.
    fn update_property<T>(
        &mut self,
        prop_name: &str,
        val: T,
    ) -> Result<(), properties::PropertyError>;
}
