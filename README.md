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

- `--help`: Show help message
- `--version`: Show version information
- `--inodes`: Count inodes instead of measuring disk usage
- `--sort`: Sort by size
- `--reverse`: Reverse sort order

### Man Page

After installation, you can view the man page with:

```bash
dustr-man
```

Or if the man page is in your system's MANPATH:

```bash
man dustr
```

To install the man page to your local man directory for system-wide access:

```bash
mkdir -p ~/.local/share/man/man1
cp $(python3 -c "import sysconfig, os; print(os.path.join(sysconfig.get_path('data'), 'share', 'man', 'man1', 'dustr.1'))") ~/.local/share/man/man1/
mandb  # Update man database (may require sudo on some systems)
```

## Differences from duk

- **Performance**: Rust backend provides faster directory traversal
- **Implementation**: Uses native Rust file system operations instead of calling `du` command
- **Progress**: Since the Rust implementation calculates all sizes in parallel, progress indication is less granular

## Requirements

- Python >= 3.9
- Rust toolchain (for building from source)

## License

LGPL-3.0-or-later
