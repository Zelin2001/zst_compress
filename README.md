# Scope

- Do batch compression with standard tools.

- Simplify the work for team data maintainers.

# Installation

## For Windows 10/11 systems

1. Put the `.bin` folder alongside the `zst_compress` and `zst_extract` binary.

   - Put into the `.bin` folder `zstd.exe`, that can be downloaded from [here](https://github.com/facebook/zstd/releases).

   - Put into the `.bin` folder `eza.exe`, that can be downloaded from [here](https://github.com/eza-community/eza/releases).

2. Change directory to the archive folder you are working with, open windows terminal and run zst_compress.exe or zst_extract.exe with absolute path.

## For Linux systems

1. [Install](https://crates.io/crates/eza) `eza` with package manager of your choice.

2. Put the binary files `zst_compress` and `zst_extract` in path and run them in desired working directories.

# Usage

```
Usage: zst_extract.exe [OPTIONS]

Options:
  -p, --preserve         Preseve original files, do not delete
  -t, --target <TARGET>  Target location for oprated files
  -h, --help             Print help
  -V, --version          Print version
```