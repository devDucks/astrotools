use serialport::{available_ports, SerialPortInfo, SerialPortType, UsbPortInfo};

/// Simple entrypoint to find on the system serial devices
/// connected via USB that will match `device_name`.
/// Since someone may potetnially have multiple devices from
/// the same manufacturer, this function will return a vec of
/// tuples containing the serial address and information read
/// fromt the port.
pub fn find_serial_devices(device_name: &str) -> Vec<(String, UsbPortInfo)> {
    find_serial_devices_with(device_name, || available_ports().unwrap())
}

fn find_serial_devices_with<F>(device_name: &str, list_ports: F) -> Vec<(String, UsbPortInfo)>
where
    F: Fn() -> Vec<SerialPortInfo>,
{
    let mut devices = Vec::new();

    for port in list_ports() {
        if let SerialPortType::UsbPort(info) = port.port_type {
            if let Some(ref serial) = info.serial_number {
                if serial.starts_with(device_name) {
                    devices.push((port.port_name, info));
                }
            }
        }
    }
    devices
}

#[cfg(test)]
mod tests {
    use super::*;
    use serialport::SerialPortType;

    fn make_usb_port(port_name: &str, serial: Option<&str>) -> SerialPortInfo {
        SerialPortInfo {
            port_name: port_name.to_string(),
            port_type: SerialPortType::UsbPort(UsbPortInfo {
                vid: 0x0403,
                pid: 0x6001,
                serial_number: serial.map(str::to_string),
                manufacturer: None,
                product: None,
            }),
        }
    }

    #[test]
    fn matches_device_by_serial_prefix() {
        let ports = vec![
            make_usb_port("/dev/ttyUSB0", Some("ABC123")),
            make_usb_port("/dev/ttyUSB1", Some("XYZ999")),
        ];
        let result = find_serial_devices_with("ABC", || ports.clone());
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, "/dev/ttyUSB0");
    }

    #[test]
    fn matches_exact_serial() {
        let ports = vec![make_usb_port("/dev/ttyUSB0", Some("EXACT"))];
        let result = find_serial_devices_with("EXACT", || ports.clone());
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn returns_multiple_matches() {
        let ports = vec![
            make_usb_port("/dev/ttyUSB0", Some("ABC001")),
            make_usb_port("/dev/ttyUSB1", Some("ABC002")),
            make_usb_port("/dev/ttyUSB2", Some("XYZ000")),
        ];
        let result = find_serial_devices_with("ABC", || ports.clone());
        assert_eq!(result.len(), 2);
    }

    #[test]
    fn skips_ports_without_serial_number() {
        let ports = vec![make_usb_port("/dev/ttyUSB0", None)];
        let result = find_serial_devices_with("ABC", || ports.clone());
        assert!(result.is_empty());
    }

    #[test]
    fn skips_non_usb_ports() {
        let ports = vec![SerialPortInfo {
            port_name: "/dev/ttyS0".to_string(),
            port_type: SerialPortType::Unknown,
        }];
        let result = find_serial_devices_with("ABC", || ports.clone());
        assert!(result.is_empty());
    }

    #[test]
    fn returns_empty_when_no_ports() {
        let result = find_serial_devices_with("ABC", || vec![]);
        assert!(result.is_empty());
    }
}
