# `I`mport `E`xport `F`ind

Cross-platform ELF/PE/MachO import/export search using [goblin](https://docs.rs/goblin).

Uses [ignore](https://docs.rs/ignore) crate for fast recursive directory iteration that respects various filters such as globs, file types and `.gitignore` files.

## Build/Install

### git *(HEAD)*

```sh
git clone https://github.com/etke/ief && cd ief
cargo build --release
cargo install --path .
```

### cargo

```sh
cargo install ief
```

## Usage

```sh
Usage: ief <path> <-e|-i|-l> <name>
```

### Performance

On a Surface Pro running Win10, `ief` is able to recursively search `C:\Windows\System32` for binaries that import `ntdll.dll` in roughly `74.1` seconds.

```powershell
PS C:\Users\etke> Measure-Command { ief 'C:\Windows\System32\' -l ntdll.dll | Out-Host }

searching for library import "ntdll.dll" in C:\Windows\System32\
C:\Windows\System32\aadauthhelper.dll
(...) // edited for brevity
C:\Windows\System32\zipfldr.dll


Days              : 0
Hours             : 0
Minutes           : 1
Seconds           : 14
Milliseconds      : 169
Ticks             : 741690457
TotalDays         : 0.000858438028935185
TotalHours        : 0.0206025126944444
TotalMinutes      : 1.23615076166667
TotalSeconds      : 74.1690457
TotalMilliseconds : 74169.0457
```
