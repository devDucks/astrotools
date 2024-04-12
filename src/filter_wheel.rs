use crate::types::{DevType, DeviceType};

pub trait FilterWheel {
    fn actual_slot(&self) -> i32;
    fn set_slot(&self, slot: i32);
    fn set_unidirection(&self, flag: bool);
    fn is_unidirectional(&self) -> bool;
}

impl DevType for dyn FilterWheel {
    fn dev_type(&self) -> DeviceType {
        DeviceType::FilterWheel
    }
}
