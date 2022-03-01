use cpp_demangle::{DemangleOptions, Symbol};

use goblin::elf::sym::Symtab;
use goblin::error::Error;
use goblin::mach::exports::Export;
use goblin::mach::imports::Import;
use goblin::mach::Mach;
use goblin::strtab::Strtab;
use goblin::Object;

use ignore::Walk;

use memmap::Mmap;

use std::fs;
use std::path::Path;

use crate::SymbolType;

#[macro_export]
macro_rules! demangle_compare {
    ($left:expr; $right:expr) => {
        demangle($left).eq(&demangle($right))
    };
}

fn demangle(symname: &str) -> String {
    if let Ok(symbol) = Symbol::new(symname) {
        let dopts: DemangleOptions = DemangleOptions::new();
        if let Ok(demangled) = symbol.demangle(&dopts) {
            return demangled;
        }
    }
    symname.to_string()
}

fn find_in_elf(
    dynsyms: &Symtab,
    dynstrtab: &Strtab,
    stype: &SymbolType,
    name: &str,
) -> bool {
    for sym in dynsyms.iter() {
        match sym.st_shndx {
            // SHN_UNDEF - import
            0 => {
                if stype.eq(&SymbolType::Import) {
                    if let Some(iname) = dynstrtab.get_at(sym.st_name) {
                        if demangle_compare!(iname; name) {
                            return true;
                        }
                    }
                }
            }
            // everything else is exported
            _ => {
                if stype.eq(&SymbolType::Export) {
                    if let Some(ename) = dynstrtab.get_at(sym.st_name) {
                        if demangle_compare!(ename; name) {
                            return true;
                        }
                    }
                }
            }
        }
    }
    false
}

fn find_in_macho(
    imports: Vec<Import<'_>>,
    exports: Vec<Export<'_>>,
    stype: &SymbolType,
    name: &str,
) -> bool {
    match stype {
        SymbolType::Import => {
            for import in imports {
                if demangle_compare!(import.name; name) {
                    return true;
                }
            }
        }
        SymbolType::Export => {
            for export in exports {
                if demangle_compare!(&export.name; name) {
                    return true;
                }
            }
        }
        SymbolType::Library => {
            for import in imports {
                if import.dylib.contains(name) {
                    return true;
                }
            }
        }
    }
    false
}

fn parse(file: &Path, stype: &SymbolType, name: &str) -> Result<(), Error> {
    let fp = fs::File::open(file);
    if let Err(err) = fp {
        return Err(Error::IO(err));
    }
    let buffer = unsafe { Mmap::map(&fp.unwrap()) };
    if let Ok(buffer) = buffer {
        match Object::parse(&buffer)? {
            Object::Elf(elf) => match stype {
                SymbolType::Library => {
                    for library in &elf.libraries {
                        if library.contains(name) {
                            return Ok(());
                        }
                    }
                }
                _ => {
                    if find_in_elf(&elf.dynsyms, &elf.dynstrtab, stype, name) {
                        return Ok(());
                    }
                }
            },
            Object::PE(pe) => match stype {
                SymbolType::Import => {
                    for import in &pe.imports {
                        if demangle_compare!(
                            &import.name; name)
                        {
                            return Ok(());
                        }
                    }
                }
                SymbolType::Export => {
                    for export in &pe.exports {
                        if let Some(export_name) = export.name {
                            if demangle_compare!(
                                export_name;
                                name)
                            {
                                return Ok(());
                            }
                        }
                    }
                }
                SymbolType::Library => {
                    for import in &pe.imports {
                        if import.dll.contains(name) {
                            return Ok(());
                        }
                    }
                }
            },
            Object::Mach(mach) => match mach {
                Mach::Binary(macho) => {
                    if let (Ok(imports), Ok(exports)) =
                        (macho.imports(), macho.exports())
                    {
                        if find_in_macho(imports, exports, stype, name) {
                            return Ok(());
                        }
                    }
                }
                Mach::Fat(fatmach) => match fatmach.arches() {
                    Ok(arches) => {
                        for (idx, _) in arches.iter().enumerate() {
                            if let Ok(container) = fatmach.get(idx) {
                                if let (Ok(imports), Ok(exports)) =
                                    (container.imports(), container.exports())
                                {
                                    if find_in_macho(
                                        imports, exports, stype, name,
                                    ) {
                                        return Ok(());
                                    }
                                }
                            }
                        }
                    }
                    _ => {
                        return Err(Error::Malformed(
                            "Malformed Fat MachO".to_string(),
                        ))
                    }
                },
            },
            _ => return Err(Error::BadMagic(0)),
        }
    }
    return Err(Error::Malformed(format!(
        "Unable to parse {}",
        &file.display()
    )));
}

#[must_use]
pub fn walk(basepath: &Path, stype: &SymbolType, name: &str) -> Vec<String> {
    let mut retvec: Vec<String> = Vec::new();
    for entry in Walk::new(basepath).flatten() {
        if parse(entry.path(), stype, name).is_ok() {
            retvec.extend(vec![entry.path().display().to_string()]);
        }
    }
    retvec
}
