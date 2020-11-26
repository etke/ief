extern crate cpp_demangle;
extern crate goblin;
extern crate ignore;

use cpp_demangle::{DemangleOptions, Symbol};

use goblin::elf::sym::Symtab;
use goblin::error::Error;
use goblin::mach::exports::Export;
use goblin::mach::imports::Import;
use goblin::mach::{Mach, MachO};
use goblin::strtab::Strtab;
use goblin::{error, Object};

use ignore::Walk;

use memmap::Mmap;

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::{env, fs, process};

#[macro_export]
macro_rules! demangle_compare {
    ($left:expr; $right:expr) => {
        demangle($left).eq(&demangle($right))
    };
}

fn demangle(symname: &str) -> String {
    if let Ok(symbol) = Symbol::new(symname) {
        let dopts: DemangleOptions = DemangleOptions { no_params: true };
        if let Ok(demangled) = symbol.demangle(&dopts) {
            return demangled;
        }
    }
    symname.to_string()
}

fn find_in_elf(
    dynsyms: &Symtab,
    dynstrtab: &Strtab,
    ie: u8,
    name: &OsStr,
) -> bool {
    for sym in dynsyms.iter() {
        match sym.st_shndx {
            // SHN_UNDEF - import
            0 => {
                if ie.eq(&b'i') {
                    if let Some(import) = dynstrtab.get(sym.st_name) {
                        if let Ok(iname) = import {
                            if demangle_compare!(
                                &iname.to_string(); &name.to_string_lossy())
                            {
                                return true;
                            }
                        }
                    }
                }
            }
            // everything else is exported
            _ => {
                if ie.eq(&b'e') {
                    if let Some(export) = dynstrtab.get(sym.st_name) {
                        if let Ok(ename) = export {
                            if demangle_compare!(
                                &ename.to_string(); &name.to_string_lossy())
                            {
                                return true;
                            }
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
    ie: u8,
    name: &OsStr,
) -> bool {
    match ie {
        b'i' => {
            for import in imports {
                if demangle_compare!(
                    &import.name.to_string(); &name.to_string_lossy())
                {
                    return true;
                }
            }
        }
        b'e' => {
            for export in exports {
                if demangle_compare!(
                    &export.name.to_string(); &name.to_string_lossy())
                {
                    return true;
                }
            }
        }
        b'l' => {
            for import in imports {
                if let Some(name) = name.to_str() {
                    if import.dylib.contains(name) {
                        return true;
                    }
                }
            }
        }
        _ => (),
    }
    false
}

fn parse(file: &Path, ie: u8, name: &OsStr) -> error::Result<()> {
    let fp = fs::File::open(file);
    if let Err(err) = fp {
        return Err(Error::IO(err));
    }
    let buffer = unsafe { Mmap::map(&fp.unwrap()) };
    if let Ok(buffer) = buffer {
        match Object::parse(&buffer)? {
            Object::Elf(elf) => match ie {
                b'l' => {
                    for library in &elf.libraries {
                        if let Some(name) = name.to_str() {
                            if library.contains(name) {
                                return Ok(());
                            }
                        }
                    }
                }
                _ => {
                    if find_in_elf(&elf.dynsyms, &elf.dynstrtab, ie, &name) {
                        return Ok(());
                    }
                }
            },
            Object::PE(pe) => match ie {
                b'i' => {
                    for import in &pe.imports {
                        if demangle_compare!(
                            &import.name.to_string(); &name.to_string_lossy())
                        {
                            return Ok(());
                        }
                    }
                }
                b'e' => {
                    for export in &pe.exports {
                        if let Some(export_name) = export.name {
                            if demangle_compare!(
                                &export_name.to_string();
                                &name.to_string_lossy())
                            {
                                return Ok(());
                            }
                        }
                    }
                }
                b'l' => {
                    for import in &pe.imports {
                        if let Some(name) = name.to_str() {
                            if import.dll.contains(name) {
                                return Ok(());
                            }
                        }
                    }
                }
                _ => (),
            },
            Object::Mach(mach) => match mach {
                Mach::Binary(macho) => {
                    let imports: Vec<Import<'_>> = macho.imports().unwrap();
                    let exports: Vec<Export<'_>> = macho.exports().unwrap();
                    if find_in_macho(imports, exports, ie, name) {
                        return Ok(());
                    }
                }
                Mach::Fat(fatmach) => {
                    for (idx, _) in fatmach.iter_arches().enumerate() {
                        let container: MachO = fatmach.get(idx).unwrap();
                        let imports: Vec<Import<'_>> =
                            container.imports().unwrap();
                        let exports: Vec<Export<'_>> =
                            container.exports().unwrap();
                        if find_in_macho(imports, exports, ie, name) {
                            return Ok(());
                        }
                    }
                }
            },
            _ => return Err(Error::BadMagic(0)),
        }
    }
    Err(Error::Malformed(format!("Unable to parse {}", &file.display())))
}

fn walk(basepath: &Path, ie: u8, name: &OsStr) {
    for result in Walk::new(basepath) {
        if let Ok(entry) = result {
            if parse(entry.path(), ie, name).is_ok() {
                println!("{}", entry.path().display())
            }
        }
    }
}

fn usage() {
    println!("Usage: ief <path> <-e|-i|-l> <name>");
    process::exit(0);
}

fn main() {
    let argv: Vec<OsString> = env::args_os().collect();
    match argv.len() {
        4 => {
            let basepath: &Path = Path::new(&argv[1]);
            let ie: u8 = argv[2].to_str().unwrap().bytes().nth(1).unwrap();
            let name: &OsString = &argv[3];
            match ie {
                b'e' => {
                    println!(
                        "searching for export \"{}\" in {}",
                        name.to_string_lossy(),
                        basepath.display()
                    );
                }
                b'i' => {
                    println!(
                        "searching for import \"{}\" in {}",
                        name.to_string_lossy(),
                        basepath.display()
                    );
                }
                b'l' => {
                    println!(
                        "searching for library import \"{}\" in {}",
                        name.to_string_lossy(),
                        basepath.display()
                    );
                }
                _ => {
                    usage();
                }
            };
            walk(basepath, ie, name);
        }
        _ => {
            usage();
        }
    }
}
