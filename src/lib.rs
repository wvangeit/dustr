use pyo3::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use walkdir::WalkDir;

/// Calculate directory sizes for all items in a directory
#[pyfunction]
fn calculate_directory_sizes(path: &str, use_inodes: bool) -> PyResult<HashMap<String, u64>> {
    let mut sizes: HashMap<String, u64> = HashMap::new();
    let base_path = Path::new(path);

    if !base_path.exists() {
        return Err(PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(
            format!("Directory not found: {}", path),
        ));
    }

    let entries = match fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(e) => {
            return Err(PyErr::new::<pyo3::exceptions::PyPermissionError, _>(
                format!("Permission denied: {}", e),
            ));
        }
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name().to_string_lossy().to_string();
        let file_path = entry.path();

        let size = if use_inodes {
            // Count inodes (files + directories)
            if file_path.is_dir() {
                WalkDir::new(&file_path)
                    .into_iter()
                    .filter_map(|e| e.ok())
                    .count() as u64
            } else {
                1
            }
        } else {
            // Calculate size in kilobytes
            calculate_size_kb(&file_path)
        };

        sizes.insert(file_name, size);
    }

    Ok(sizes)
}

/// Calculate total size in kilobytes for a file or directory
fn calculate_size_kb(path: &Path) -> u64 {
    if path.is_file() {
        fs::metadata(path)
            .map(|m| (m.len() + 1023) / 1024) // Round up to KB
            .unwrap_or(0)
    } else if path.is_dir() {
        WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .filter_map(|e| fs::metadata(e.path()).ok())
            .map(|m| (m.len() + 1023) / 1024)
            .sum()
    } else {
        0
    }
}

/// Get file type indicator (@ for symlinks, / for directories, empty for files)
#[pyfunction]
fn get_file_type_indicator(path: &str) -> PyResult<String> {
    let p = Path::new(path);
    
    if p.is_symlink() {
        Ok("@".to_string())
    } else if p.is_dir() {
        Ok("/".to_string())
    } else {
        Ok("".to_string())
    }
}

/// Python module definition
#[pymodule]
fn _dustr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate_directory_sizes, m)?)?;
    m.add_function(wrap_pyfunction!(get_file_type_indicator, m)?)?;
    Ok(())
}
