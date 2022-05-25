pub mod devices {
    pub use lightspeed::devices::actions::DeviceActions;
    use lightspeed::props::Property;
    #[cfg(test)]
    use mockall::{automock, predicate::*};
    #[cfg(windows)]
    use serialport::COMPort;
    #[cfg(unix)]
    use serialport::TTYPort;
    use uuid::Uuid;

    pub struct BaseDevice {
        pub id: Uuid,
        pub name: String,
        pub properties: Vec<Property>,
        pub address: String,
        pub baud: u32,
        #[cfg(unix)]
        pub port: TTYPort,
        #[cfg(windows)]
        pub port: COMPort,
    }

    #[cfg_attr(test, automock)]
    pub trait AstronomicalDevice {
        fn new(name: &str, address: &str, baud: u32, timeout_ms: u64) -> Option<Self>
        where
            Self: Sized;
        fn send_command<C: 'static>(
            &mut self,
            comm: C,
            val: Option<String>,
        ) -> Result<String, DeviceActions>;
        fn fetch_props(&mut self);
        fn get_id(&self) -> Uuid;
        fn get_address(&self) -> &String;
        fn get_name(&self) -> &String;
        fn get_properties(&self) -> &Vec<Property>;
        fn update_property(&mut self, prop_name: &str, val: &str) -> Result<(), DeviceActions>;
        fn update_property_remote(
            &mut self,
            prop_name: &str,
            val: &str,
        ) -> Result<(), DeviceActions>;
        fn find_property_index(&self, prop_name: &str) -> Option<usize>;
    }

    pub type PowerBoxDevice = BaseDevice;
    pub type CcdDevice = BaseDevice;
    pub type MountDevice = BaseDevice;
    pub type FocuserDevice = BaseDevice;
}

pub mod utils {
    use crate::devices::AstronomicalDevice;

    pub fn print_device_table<T: AstronomicalDevice>(devices: &Vec<T>) {
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
