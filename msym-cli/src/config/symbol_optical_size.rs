use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum SymbolOpticalSize {
    Dp20 = 20,
    Dp24 = 24,
    Dp40 = 40,
    Dp48 = 48,
}

impl Display for SymbolOpticalSize {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", u32::from(*self))
    }
}
