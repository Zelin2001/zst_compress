# zst_compress

[![Crates.io Downloads (latest version)](https://img.shields.io/crates/dv/zst_compress)](https://crates.io/crates/zst_compress)
![Crates.io License](https://img.shields.io/crates/l/zst_compress)
[![Crates.io Version](https://img.shields.io/crates/v/zst_compress)](https://crates.io/crates/zst_compress)
[![GitHub Release](https://img.shields.io/github/v/release/Zelin2001/zst_compress)](https://github.com/Zelin2001/zst_compress)
[![GitHub Release Date](https://img.shields.io/github/release-date/Zelin2001/zst_compress)](https://github.com/Zelin2001/zst_compress)

Batch compress to or decompress dir/\*.tar.zst

## Scope

- Do zst batch compression within rust.

- Simplify the work for team data maintainers.

## Installation

You could choose to install from pre-built binary or Build from source.

### Install from pre-built binary

#### For Windows 10/11 systems

1. Extract `zst_compress-<version>.zip`

   > Put it into searching path if you like to.

2. Change directory to the archive folder you are working with,
   open windows terminal and run `zst_compress.exe -h` and follow the instructions.

   > If zst_compress.exe was not in Path, just run with absolute path.

#### For Linux systems

1. Extract `zst_compress-<version>.tar.zst` with:
   `tar -xvf zst_compress-<version>.tar.zst`

   > Put it into searching path if you like to.

2. In terminal emulator, change directory to the archive folder you are working with,
   run `zst_compress -h` and follow the instructions.

### Build from source

The program was brought with 🦀Rust and therefore can work with most Rust hosts.

This will work for **Linux**, **Windows**, **macOS** with different chips.

1. Install Rust: follow the tutorial to
   [install Rust](https://www.rust-lang.org/tools/install).

2. Install a Rust [toolchain](https://rust-lang.github.io/rustup/concepts/toolchains.html).

3. Install zst_compress with `cargo install --locked zst_compress`.

4. Now you are ready to run `zst_compress -h`.

## Usage

```
Usage: zst_compress [OPTIONS] <DIRECTORY>

Arguments:
  <DIRECTORY>  Directory to start processing

Options:
  -n, --dryrun              Preview what would be done without executing
  -e, --exclude <PATTERN>   Exclude files matching glob pattern(s)
  -x, --extract             Extract files (decompress mode)
  -f, --flag                Create compression message file on the compression
  -i, --include <PATTERN>   Include files matching glob pattern(s) [default: *]
  -l, --leveldir <LEVEL>    Directory listing depth for logs in *_archive_filelist.txt [default: 4]
  -p, --preserve            Keep original (do not delete) files after compression
  -q, --quiet               Suppress output except errors (NO FUNCTION)
  -t, --target <DIRECTORY>  Output directory [default: DIRECTORY to start]
  -v, --verbose             Show detailed progress information (NO FUNCTION)
  -z, --zstdlevel <LEVEL>   Zstandard compress level, 1(fastest) to 22(smallest); [default: 3]
  -h, --help                Print help
  -V, --version             Print version
```