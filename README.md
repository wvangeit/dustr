# Dustr

Dustr is a Rust-based implementation of [duk](https://github.com/wvangeit/duk), a commandline utility that shows disk usage statistics in a directory with a histogram visualization.

## Introduction

Dustr provides the same functionality as duk but with a Rust backend for improved performance. It will show you a histogram of the disk usage in a directory:

```bash
>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>>

Statistics of directory "." :

in kByte       in %   histogram            name
4              1.39   #                    .gitignore
4              1.39   #                    setup.py
4              1.39   #                    .travis.yml
4              1.39   #                    README.md
12             4.17   ##                   dist/
12             4.17   ##                   dustr/
12             4.17   ##                   build/
16             5.56   ##                   duk.egg-info/
220            76.39  #################### .git/

Total directory size: 288 kByte
```

## Installation

### From PyPI (once published)

```bash
pip install dustr
```

### From source

First, ensure you have Rust installed (see [rustup.rs](https://rustup.rs/)).

Then install maturin:

```bash
pip install maturin
```

Build and install the package:

```bash
cd dustr
maturin develop  # For development
# OR
maturin build --release  # To build a wheel
pip install target/wheels/dustr-*.whl
```

## Usage

```bash
dustr [directory]
```

Options:

- `--nogrouping`: Don't group thousands with commas
- `--noprogress`: Don't show progress bar (note: Rust version calculates all at once, so this has less effect)
- `--inodes`: Show inode count instead of size
- `--noF`: Don't add file type indicators (/ for directories, @ for symlinks)

## Differences from duk

- **Performance**: Rust backend provides faster directory traversal
- **Implementation**: Uses native Rust file system operations instead of calling `du` command
- **Progress**: Since the Rust implementation calculates all sizes in parallel, progress indication is less granular

## Requirements

- Python >= 3.9
- Rust toolchain (for building from source)

## License

LGPL-3.0-or-later
