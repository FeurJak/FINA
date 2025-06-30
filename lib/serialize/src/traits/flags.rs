pub trait Flags: Default + Clone + Copy + Sized {
    const BIT_SIZE: usize;
    fn u8_bitmask(&self) -> u8;
    fn from_u8(value: u8) -> Option<Self>;
    fn from_u8_remove_flags(value: &mut u8) -> Option<Self> {
        Self::from_u8(*value).map(|f| {
            *value &= !f.u8_bitmask();
            f
        })
    }
}
