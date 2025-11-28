# Development Guide for Dustr

## Prerequisites

1. **Rust**: Install from [rustup.rs](https://rustup.rs/)

   ```bash
   curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
   ```

2. **Maturin**: Python build tool for Rust extensions
   ```bash
   pip install maturin
   ```

## Building

### Development Build

For rapid iteration during development:

```bash
cd dustr
maturin develop
```

This builds the Rust extension and installs it in your current Python environment.

### Release Build

To create an optimized wheel:

```bash
maturin build --release
```

The wheel will be in `target/wheels/`.

### Build for Multiple Python Versions

```bash
maturin build --release --interpreter python3.9 python3.10 python3.11 python3.12
```

## Testing

After building with `maturin develop`, you can test the command:

```bash
dustr .
dustr --inodes /tmp
dustr --nogrouping ~
```

## Project Structure

```
dustr/
├── Cargo.toml              # Rust package configuration
├── pyproject.toml          # Python package configuration
├── README.md
├── .gitignore
├── src/
│   └── lib.rs             # Rust implementation (PyO3 bindings)
└── python/
    └── dustr/
        └── __init__.py    # Python CLI wrapper
```

## How It Works

1. **Rust Core** (`src/lib.rs`): Implements fast directory traversal using `walkdir` crate
2. **PyO3 Bindings**: Exposes Rust functions to Python
3. **Python Wrapper** (`python/dustr/__init__.py`): CLI interface and formatting
4. **Maturin**: Builds the Rust extension as a Python wheel

## Publishing

### To PyPI

```bash
# Build wheels for multiple platforms using cibuildwheel or maturin
maturin build --release

# Upload to PyPI
maturin publish
```

### To TestPyPI

```bash
maturin publish --repository testpypi
```

## Performance Notes

The Rust implementation calculates all directory sizes in parallel, which is significantly faster than calling `du` sequentially for each subdirectory (as the original duk does). This makes the progress bar less meaningful since calculation happens very quickly.

## Troubleshooting

### Import Error

If you get `ImportError: cannot import name '_dustr'`:

- Run `maturin develop` again
- Check that you're in the correct virtual environment
- Verify Rust toolchain is installed: `rustc --version`

### Build Errors

If maturin build fails:

- Update Rust: `rustup update`
- Update maturin: `pip install -U maturin`
- Check `Cargo.toml` dependencies

## Adding Features

To add new Rust functions exposed to Python:

1. Add function in `src/lib.rs` with `#[pyfunction]` decorator
2. Register in `#[pymodule]` using `m.add_function(wrap_pyfunction!(your_function, m)?)?`
3. Import and use in `python/dustr/__init__.py`
4. Rebuild with `maturin develop`
