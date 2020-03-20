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

use std::ffi::{OsStr, OsString};
use std::path::Path;
use std::{env, fs, process};

fn demangle(symname: &str) -> String {
    match Symbol::new(symname) {
        Ok(symbol) => {
            let dopts: DemangleOptions = DemangleOptions { no_params: true };
            symbol.demangle(&dopts).unwrap()
        }
        _ => symname.to_string(),
    }
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
                    let import: &str =
                        dynstrtab.get(sym.st_name).unwrap().unwrap();
                    if demangle(&import.to_string())
                        .eq(&name.to_string_lossy())
                    {
                        return true;
                    }
                }
            }
            // everything else is exported
            _ => {
                if ie.eq(&b'e') {
                    let export: &str =
                        dynstrtab.get(sym.st_name).unwrap().unwrap();
                    if demangle(&export.to_string())
                        .eq(&name.to_string_lossy())
                    {
                        return true;
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
                if demangle(&import.name.to_string())
                    .eq(&name.to_string_lossy())
                {
                    return true;
                }
            }
        }
        b'e' => {
            for export in exports {
                if demangle(&export.name).eq(&name.to_string_lossy()) {
                    return true;
                }
            }
        }
        b'l' => {
            for import in imports {
                if import.dylib.contains(name.to_str().unwrap()) {
                    return true;
                }
            }
        }
        _ => (),
    }
    false
}

fn parse(file: &Path, ie: u8, name: &OsStr) -> error::Result<()> {
    let buffer: Vec<u8> = fs::read(file)?;
    match Object::parse(&buffer)? {
        Object::Elf(elf) => match ie {
            b'l' => {
                for library in &elf.libraries {
                    if library.contains(name.to_str().unwrap()) {
                        return Ok(());
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
                    if demangle(&import.name.to_string())
                        .eq(&name.to_string_lossy())
                    {
                        return Ok(());
                    }
                }
            }
            b'e' => {
                for export in &pe.exports {
                    let estr: String = export.name.unwrap().to_string();
                    if demangle(&estr).eq(&name.to_string_lossy()) {
                        return Ok(());
                    }
                }
            }
            b'l' => {
                for import in &pe.imports {
                    if import.dll.contains(name.to_str().unwrap()) {
                        return Ok(());
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
    Err(Error::Malformed(format!(
        "Unable to parse {}",
        &file.display()
    )))
}

fn walk(basepath: &Path, ie: u8, name: &OsStr) {
    for result in Walk::new(basepath) {
        match result {
            Ok(entry) => match parse(entry.path(), ie, name) {
                Ok(_) => println!("{}", entry.path().display()),
                _ => (),
            },
            _ => (),
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
