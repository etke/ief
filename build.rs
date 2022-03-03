#[cfg(feature = "binja")]
use std::env;
#[cfg(feature = "binja")]
use std::fs::File;
#[cfg(feature = "binja")]
use std::io::BufReader;
#[cfg(feature = "binja")]
use std::path::PathBuf;

#[cfg(all(target_os = "macos", feature = "binja"))]
static LASTRUN_PATH: (&str, &str) =
    ("HOME", "Library/Application Support/Binary Ninja/lastrun");

#[cfg(all(target_os = "linux", feature = "binja"))]
static LASTRUN_PATH: (&str, &str) = ("HOME", ".binaryninja/lastrun");

#[cfg(all(windows, feature = "binja"))]
static LASTRUN_PATH: (&str, &str) = ("APPDATA", "Binary Ninja\\lastrun");

#[cfg(feature = "binja")]
// Check last run location for path to BinaryNinja; Otherwise check the default install locations
fn link_path() -> PathBuf {
    use std::io::prelude::*;

    let home = PathBuf::from(env::var(LASTRUN_PATH.0).unwrap());
    let lastrun = PathBuf::from(&home).join(LASTRUN_PATH.1);

    File::open(lastrun)
        .and_then(|f| {
            let mut binja_path = String::new();
            let mut reader = BufReader::new(f);

            reader.read_line(&mut binja_path)?;
            Ok(PathBuf::from(binja_path.trim()))
        })
        .unwrap_or_else(|_| {
            #[cfg(all(target_os = "macos", feature = "binja"))]
            return PathBuf::from(
                "/Applications/Binary Ninja.app/Contents/MacOS",
            );

            #[cfg(all(target_os = "linux", feature = "binja"))]
            return home.join("binaryninja");

            #[cfg(all(windows, feature = "binja"))]
            return PathBuf::from(env::var("PROGRAMFILES").unwrap())
                .join("Vector35\\BinaryNinja\\");
        })
}

fn main() {
    #[cfg(feature = "binja")]
    // do nothing if not building with "binja" feature
    // Use BINARYNINJADIR first for custom BN builds/configurations (BN devs/build server), fallback on defaults
    let install_path =
        env::var("BINARYNINJADIR").map_or_else(|_| link_path(), PathBuf::from);

    #[cfg(all(target_os = "linux", feature = "binja"))]
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{},-L{},-l:libbinaryninjacore.so.1",
        install_path.to_str().unwrap(),
        install_path.to_str().unwrap(),
    );

    #[cfg(all(target_os = "macos", feature = "binja"))]
    println!(
        "cargo:rustc-link-arg=-Wl,-rpath,{},-L{},-lbinaryninjacore",
        install_path.to_str().unwrap(),
        install_path.to_str().unwrap(),
    );

    #[cfg(all(target_os = "windows", feature = "binja"))]
    {
        println!("cargo:rustc-link-lib=binaryninjacore");
        println!("cargo:rustc-link-search={}", install_path.to_str().unwrap());
    }
}
