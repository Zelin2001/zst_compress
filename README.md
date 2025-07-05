# Scope

- Do zst batch compression within rust.

- Simplify the work for team data maintainers.

# Installation

You could choose to install from pre-built binary or Build from source.

## Install from pre-built binary

### For Windows 10/11 systems

1. Extract `zst_compress-<version>.zip`

   > Put it into searching path if you like to.

2. Change directory to the archive folder you are working with,
   open windows terminal and run `zst_compress.exe -h` and follow the instructions.

   > If zst_compress.exe was not in Path, just run with absolute path.

### For Linux systems

1. Extract `zst_compress-<version>.tar.zst` with:
   `tar -xvf zst_compress-<version>.tar.zst`

   > Put it into searching path if you like to.

2. In terminal emulator, change directory to the archive folder you are working with,
   run `zst_compress -h` and follow the instructions.

## Build from source

The program was brought with 🦀Rust and therefore can work with most Rust hosts. 

This will work for **Linux**, **Windows**, **macOS** with different chips.

1. Install Rust: follow the tutorial to
   [install Rust](https://www.rust-lang.org/tools/install).

2. Install a Rust [toolchain](https://rust-lang.github.io/rustup/concepts/toolchains.html).

3. Install zst_compress with `cargo install --locked zst_compress`.

4. Now you are ready to run `zst_compress -h`.

# Usage

```
Usage: zst_extract.exe [OPTIONS]

Options:
  -p, --preserve            Preseve original files after compression
  -f, --flag                Leave flag text pointing to compression target
  -i, --input <INPUT>       Select a single input file instead of ./*
  -x                        Extract file from batch archived
  -l, --leveldir <LEVELDIR> Select recursive level for listing directory, default to 4
  -t, --target <TARGET>     Target location for oprated files, default to current
  -h, --help                Print help
  -V, --version             Print version
```
