use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum SymbolWeight {
    W100 = 100,
    W200 = 200,
    W300 = 300,
    W400 = 400,
    W500 = 500,
    W600 = 600,
    W700 = 700,
}

impl Display for SymbolWeight {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", u32::from(*self))
    }
}
