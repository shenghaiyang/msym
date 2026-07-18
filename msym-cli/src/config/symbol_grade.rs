use num_enum::{IntoPrimitive, TryFromPrimitive};
use std::fmt;
use std::fmt::Display;

#[derive(Debug, Clone, Copy, IntoPrimitive, TryFromPrimitive)]
#[repr(i32)]
pub enum SymbolGrade {
    Low = -25,
    Normal = 0,
    High = 200,
}

impl Display for SymbolGrade {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", i32::from(*self))
    }
}
