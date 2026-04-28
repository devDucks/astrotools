pub trait FilterWheel {
    fn actual_slot(&self) -> i32;
    fn set_slot(&self, slot: i32);
    fn set_unidirection(&self, flag: bool);
    fn is_unidirectional(&self) -> bool;
}
