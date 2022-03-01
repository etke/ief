#![warn(clippy::pedantic)]
use std::path::Path;
use std::{env, process};

use ief::{walk, SymbolType};

fn usage() {
    eprintln!("Usage: ief <path> <-e|-i|-l> <name>");
    process::exit(1);
}

fn main() {
    let argv: Vec<String> = env::args().collect();
    match argv.len() {
        4 => {
            let basepath: &Path = Path::new(&argv[1]);
            let stype = {
                match argv[2].as_str() {
                    "-e" => SymbolType::Export,
                    "-i" => SymbolType::Import,
                    "-l" => SymbolType::Library,
                    _ => return usage(),
                }
            };
            let name: &String = &argv[3];
            match stype {
                SymbolType::Export => {
                    println!(
                        "searching for export \"{}\" in {}",
                        name,
                        basepath.display()
                    );
                }
                SymbolType::Import => {
                    println!(
                        "searching for import \"{}\" in {}",
                        name,
                        basepath.display()
                    );
                }
                SymbolType::Library => {
                    println!(
                        "searching for library import \"{}\" in {}",
                        name,
                        basepath.display()
                    );
                }
            };
            for entry in walk::walk(basepath, &stype, name) {
                println!("{}", entry);
            }
        }
        _ => {
            usage();
        }
    }
}
