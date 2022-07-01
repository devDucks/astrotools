//! The basic building blocks you need to build your next astrophotography suite.
//!
//! Astrotools provides traits and utils that can be used to implement
//! multiplatform drivers to drive astronomical equipment.
use lightspeed_astro::devices::actions::DeviceActions;
use lightspeed_astro::props::Property;
use uuid::Uuid;

pub trait AstroSerialDevice {
    /// Main and only entrypoint to create a new serial device.
    ///
    /// A device that doesn't work/cannot communicate with is not really useful
    /// so this may return `None` if there is something wrong with the just
    /// discovered device.
    fn new(name: &str, address: &str, baud: u32, timeout_ms: u64) -> Option<Self>
    where
        Self: Sized;

    /// Implement this method with the actual logic to issue serial commands
    /// to a  device.
    fn send_command<C: std::fmt::UpperHex>(
        &mut self,
        comm: C,
        val: Option<String>,
    ) -> Result<String, DeviceActions>;

    /// Use this method to fetch the real properties from the device,
    /// this should not be called directly from clients ideally,
    /// for that goal `get_properties` should be used.
    fn fetch_props(&mut self);

    /// Use this method to return the id of the device as a uuid.
    fn get_id(&self) -> Uuid;

    /// Return the OS address of the device (/dev/ttyUSB0 or /COM6 for example).
    fn get_address(&self) -> &String;

    /// Use this method to return the name of the device (e.g. ZWO533MC).
    fn get_name(&self) -> &String;

    /// Use this method to return the actual cached state stored into `self.properties`.
    fn get_properties(&self) -> &Vec<Property>;

    /// Method to be used when receving requests from clients to update properties.
    ///
    /// Ideally this should call internally `update_property_remote` which will be
    /// responsible to trigger the action against the device to update the property
    /// on the device itself, if the action is successful the last thing this method
    /// does would be to update the property inside `self.properties`.
    fn update_property(&mut self, prop_name: &str, val: &str) -> Result<(), DeviceActions>;

    /// Use this method to send a command to the device to change the requested property.
    ///
    /// Ideally this method will be a big `match` clause where the matching will execute
    /// `self.send_command` to issue a serial command to the device.
    fn update_property_remote(&mut self, prop_name: &str, val: &str) -> Result<(), DeviceActions>;

    /// Properties are packed into a vector so to find them we need to
    /// lookup the index, use this method to do so.
    fn find_property_index(&self, prop_name: &str) -> Option<usize>;
}

/// Module that gathers all functions that may help debugging or working woth devices.
pub mod utils {
    use std::net::{SocketAddr, TcpListener};

    use crate::AstroSerialDevice;

    /// Private method that checks if a given port is free or used.
    fn port_is_available(host: &str, port: u16) -> bool {
        match TcpListener::bind((host, port)) {
            Ok(_) => true,
            Err(_) => false,
        }
    }

    /// Build a socket address from a given host, this method first looks for a free
    /// port in range 50051 to 50551 (where gRPC astroservice will live) and then returns
    /// a SocketAddre of the given pair host:free_port.
    pub fn build_server_address(host: &str) -> SocketAddr {
        let port = {
            (50051..50551)
                .find(|port| port_is_available(host, *port))
                .unwrap()
        };
        format!("{host}:{port}").parse().unwrap()
    }

    pub fn print_device_table<T: AstroSerialDevice>(devices: &Vec<T>) {
        for d in devices {
            println!("");
            println!("=======================================");
            println!("Device id: {}", d.get_id());
            println!("Device address: {}", d.get_address());
            println!("Device name: {}", d.get_name());
            println!("=======================================");
            println!("");
            println!(
            "-----------------------------------------------------------------------------------"
        );
            println!(
            "|          name           |    value        |    kind     |    permission         |"
        );
            println!(
            "-----------------------------------------------------------------------------------"
        );

            for prop in d.get_properties() {
                let name_padding = 25 - prop.name.len();
                let val_padding = 17 - prop.value.len();
                let kind_padding = 13 - prop.kind.len();
                let mut perm_padding = 15;

                match prop.permission {
                    0 => (),
                    _ => {
                        perm_padding = 14;
                    }
                }
                let mut name = String::new();
                let mut val = String::new();
                let mut kind = String::new();
                let mut perm = String::new();

                for _ in 0..name_padding as usize {
                    name += " ";
                }

                for _ in 0..val_padding as usize {
                    val += " ";
                }

                for _ in 0..kind_padding as usize {
                    kind += " ";
                }

                for _ in 0..perm_padding as usize {
                    perm += " ";
                }

                println!(
                    "|{}{}|{}{}|{}{}|{:?}{}|",
                    prop.name, name, prop.value, val, prop.kind, kind, prop.permission, perm
                );
            }
            println!(
            "-----------------------------------------------------------------------------------"
        );
        }
    }
}

#[cfg(test)]
mod test_utils {

    mod factory {
        pub use crate::devices::{AstronomicalDevice, BaseDevice, CcdDevice, DeviceActions};
        use lightspeed::props::Property;
        use std::time::Duration;
        use uuid::Uuid;

        impl AstronomicalDevice for CcdDevice {
            fn new(name: &str, address: &str, baud: u32, timeout_ms: u64) -> Option<Self> {
                let port =
                    serialport::new(address, baud).timeout(Duration::from_millis(timeout_ms));
                if let Ok(p) = port.open_native() {
                    let d = Self {
                        id: Uuid::new_v4(),
                        name: name.to_owned(),
                        properties: Vec::new(),
                        address: address.to_owned(),
                        baud: baud,
                        port: p,
                    };
                    Some(d)
                } else {
                    None
                }
            }
            fn send_command<C>(
                &mut self,
                _comm: C,
                _val: Option<String>,
            ) -> Result<String, DeviceActions> {
                Ok(String::from("OK"))
            }
            fn fetch_props(&mut self) {}
            fn get_id(&self) -> Uuid {
                self.id
            }
            fn get_address(&self) -> &String {
                &self.address
            }
            fn get_name(&self) -> &String {
                &self.name
            }
            fn get_properties(&self) -> &Vec<Property> {
                &self.properties
            }
            fn update_property(
                &mut self,
                _prop_name: &str,
                _val: &str,
            ) -> Result<(), DeviceActions> {
                Ok(())
            }
            fn update_property_remote(
                &mut self,
                _prop_name: &str,
                _val: &str,
            ) -> Result<(), DeviceActions> {
                Ok(())
            }
            fn find_property_index(&self, _prop_name: &str) -> Option<usize> {
                None
            }
        }
    }

    #[test]
    fn test_device_cannot_be_constructed_fake_port() {
        use crate::test_utils::factory::{AstronomicalDevice, CcdDevice};
        let device: Option<CcdDevice> = CcdDevice::new("/lol", "foo", 9600, 500);
        assert_eq!(device.is_some(), false);
    }
}
