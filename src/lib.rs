#[derive(PartialEq)]
pub enum SymbolType {
    Import,
    Export,
    Library,
}

#[cfg(feature = "binja")]
pub mod binja;
pub mod walk;
