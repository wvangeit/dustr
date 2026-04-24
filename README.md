# Dustr

[![Build](https://github.com/wvangeit/dustr/actions/workflows/build.yml/badge.svg)](https://github.com/wvangeit/dustr/actions/workflows/build.yml)
[![PyPI version](https://img.shields.io/pypi/v/dustr)](https://pypi.org/project/dustr/)
[![Crates.io](https://img.shields.io/crates/v/dustr-cli)](https://crates.io/crates/dustr-cli)
[![License](https://img.shields.io/pypi/l/dustr)](https://github.com/wvangeit/dustr/blob/main/LICENSE.txt)
[![PyPI - Python Version](https://img.shields.io/pypi/pyversions/dustr)](https://pypi.org/project/dustr/)
[![Benchmarks](https://img.shields.io/badge/benchmarks-live-blue)](https://wvangeit.github.io/dustr/dev/bench/)

Dustr is a Rust-based implementation of [duk](https://github.com/wvangeit/duk), a commandline utility that shows disk usage statistics in a directory with a histogram visualization.

## Introduction

Dustr provides the same functionality as duk but with a Rust backend for improved performance. It will show you a histogram of the disk usage in a directory:

```
Statistics of directory "." :

Size           In %   Histogram            Name
4.0 KB         1.39  #                    .gitignore
4.0 KB         1.39  #                    setup.py
4.0 KB         1.39  #                    README.md
12.0 KB        4.17  ##                   dist/
12.0 KB        4.17  ##                   dustr/
12.0 KB        4.17  ##                   build/
16.0 KB        5.56  ##                   duk.egg-info/
220.0 KB       76.39 ####################  .git/

Total directory size: 284.0 KB
```

## Installation

### From PyPI (Python package)

```bash
pip install dustr
```

### From crates.io (standalone Rust binary, no Python needed)

```bash
cargo install dustr-cli
```

This installs the `dustr-cli` binary to `~/.cargo/bin/`.

### From source

First, ensure you have Rust installed (see [rustup.rs](https://rustup.rs/)).

**Python package** (requires maturin):

```bash
pip install maturin
maturin develop  # For development
# OR
maturin build --release && pip install target/wheels/dustr-*.whl
```

**Standalone binary**:

```bash
cargo install --path .
# OR
make install
```

## Usage

### Python (`dustr`)

```bash
dustr [OPTIONS] [DIRECTORY]
```

### Standalone binary (`dustr-cli`)

```bash
dustr-cli [OPTIONS] [DIRECTORY]
```

Both accept the same options:

- `-i, --inodes`: Show inode count instead of size
- `-g, --nogrouping`: Don't use thousand separators (for inode mode)
- `-f, --noF`: Don't add file type indicators (`/` for directories, `@` for symlinks)
- `-j, --json`: Output results as JSON
- `-x, --cross-mounts`: Cross filesystem mount boundaries
- `-v, --verbose`: Show directories being traversed
- `-l, --live`: Live-update statistics table during traversal

### JSON output

```bash
dustr --json .
```

```json
{
  "directory": ".",
  "mode": "size",
  "entries": [
    { "name": ".gitignore", "value": 4, "percentage": 1.41 },
    { "name": ".git/", "value": 220, "percentage": 77.46 }
  ],
  "total": 284
}
```

## Differences from duk

- **Performance**: Rust backend with parallel directory traversal (jwalk + rayon)
- **Implementation**: Uses native Rust file system operations instead of calling `du` command

## Requirements

- **Python package**: Python >= 3.9
- **Standalone binary**: Rust toolchain (for building from source), or install from crates.io via `cargo install dustr-cli`

## License

LGPL-3.0-or-later
