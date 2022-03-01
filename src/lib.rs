#[derive(PartialEq)]
pub enum SymbolType {
    Import,
    Export,
    Library,
}

pub mod walk;
#[cfg(feature = "binja")]
pub mod binja;