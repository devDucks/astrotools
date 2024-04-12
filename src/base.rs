/// Implement properties read/write functionalities for properties
pub trait PropertyManager {
    /// This method should ask the device for the actual state and update
    /// the internal state of the device representation
    fn fetch_props(&mut self);

    /// This method is meant to be called when a request to update a device
    /// property is sent by a client
    pub fn update_property<V>(&mut self, prop_name: &str, val: V);
}
