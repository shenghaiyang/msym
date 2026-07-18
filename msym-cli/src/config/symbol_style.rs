use strum::{Display, EnumString};

#[derive(Clone, Debug, Display, EnumString)]
#[strum(ascii_case_insensitive)]
pub enum SymbolStyle {
    Outlined,
    Rounded,
    Sharp,
}
