pub enum DeviceType {
    Ccd,
    Mount,
    Focuser,
    FilterWheel,
    PowerBox,
}

pub trait DevType {
    fn dev_type(&self) -> DeviceType;
}
