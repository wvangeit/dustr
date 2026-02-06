use clap::Parser;
use jwalk::WalkDir as JWalkDir;
use parking_lot::Mutex;
use pyo3::prelude::*;
use rayon::prelude::*;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Calculate directory sizes for all items in a directory (parallel version)
#[pyfunction]
fn calculate_directory_sizes(
    py: Python,
    path: &str,
    use_inodes: bool,
) -> PyResult<HashMap<String, u64>> {
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

    // Collect entries first to get count
    let entries_vec: Vec<_> = entries.flatten().collect();
    let total_entries = entries_vec.len();

    // Shared state for progress and cancellation
    let progress = Arc::new(AtomicUsize::new(0));
    let cancelled = Arc::new(AtomicBool::new(false));
    let results: Arc<Mutex<HashMap<String, u64>>> = Arc::new(Mutex::new(HashMap::new()));

    // Clone for the signal checking thread
    let cancelled_for_thread = cancelled.clone();

    // Spawn a thread to check for Ctrl+C while holding the GIL
    let signal_checker = std::thread::spawn(move || {
        // This thread will periodically reacquire the GIL to check signals
        loop {
            std::thread::sleep(std::time::Duration::from_millis(100));
            if cancelled_for_thread.load(Ordering::Relaxed) {
                break;
            }
            // Try to check Python signals
            Python::attach(|py| {
                if py.check_signals().is_err() {
                    cancelled_for_thread.store(true, Ordering::Relaxed);
                }
            });
            if cancelled_for_thread.load(Ordering::Relaxed) {
                break;
            }
        }
    });

    // Process entries in parallel, releasing the GIL
    py.detach(|| {
        entries_vec.par_iter().for_each(|entry| {
            // Check for cancellation
            if cancelled.load(Ordering::Relaxed) {
                return;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();

            let size = if use_inodes {
                count_inodes_parallel(&file_path, &cancelled)
            } else {
                calculate_size_kb_parallel(&file_path, &cancelled)
            };

            if !cancelled.load(Ordering::Relaxed) {
                results.lock().insert(file_name, size);
            }

            // Update progress
            let current = progress.fetch_add(1, Ordering::Relaxed) + 1;
            // Only update progress bar every 10 items or at completion
            if current % 10 == 0 || current == total_entries {
                print_progress(current, total_entries);
            }
        });
    });

    // Signal the checker thread to stop and wait for it
    cancelled.store(true, Ordering::Relaxed);
    let _ = signal_checker.join();

    // Check for Python signals after computation
    if let Err(e) = py.check_signals() {
        print!("\r{}\r", " ".repeat(50));
        io::stdout().flush().ok();
        return Err(e);
    }

    // Clear progress bar
    print!("\r{}\r", " ".repeat(50));
    io::stdout().flush().ok();

    let final_results = match Arc::try_unwrap(results) {
        Ok(mutex) => mutex.into_inner(),
        Err(arc) => arc.lock().clone(),
    };

    Ok(final_results)
}

/// Calculate total size in kilobytes using parallel jwalk
fn calculate_size_kb_parallel(path: &Path, cancelled: &AtomicBool) -> u64 {
    if path.is_file() {
        return fs::metadata(path)
            .map(|m| (m.len() + 1023) / 1024)
            .unwrap_or(0);
    }

    if !path.is_dir() {
        return 0;
    }

    // Use jwalk for parallel directory traversal
    let mut total: u64 = 0;
    let mut count = 0;
    for entry in JWalkDir::new(path)
        .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus()))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        // Check cancellation periodically
        count += 1;
        if count % 1000 == 0 && cancelled.load(Ordering::Relaxed) {
            break;
        }
        if entry.file_type().is_file() {
            total += entry
                .metadata()
                .map(|m| (m.len() + 1023) / 1024)
                .unwrap_or(0);
        }
    }
    total
}

/// Count inodes using parallel jwalk
fn count_inodes_parallel(path: &Path, cancelled: &AtomicBool) -> u64 {
    if !path.is_dir() {
        return 1;
    }

    let mut count: u64 = 0;
    let mut iter_count = 0;
    for _ in JWalkDir::new(path)
        .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus()))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        count += 1;
        iter_count += 1;
        // Check cancellation periodically
        if iter_count % 1000 == 0 && cancelled.load(Ordering::Relaxed) {
            break;
        }
    }
    count
}

/// Get the number of CPUs for parallelism
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Print a progress bar
fn print_progress(current: usize, total: usize) {
    let bar_width = 40;
    let progress = if total > 0 {
        current as f64 / total as f64
    } else {
        0.0
    };
    let filled = (bar_width as f64 * progress) as usize;
    let empty = bar_width - filled;

    print!(
        "\r[{}{}] {}/{}",
        ">".repeat(filled),
        "-".repeat(empty),
        current,
        total
    );
    io::stdout().flush().ok();
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

/// Format a number with thousand separators
/// Format size with units (K, M, G, T)
fn format_size(size_kb: u64) -> String {
    if size_kb >= 1_000_000_000 {
        // Terabytes
        let tb = size_kb as f64 / 1_000_000_000.0;
        format!("{:.1} Tb", tb)
    } else if size_kb >= 1_000_000 {
        // Gigabytes
        let gb = size_kb as f64 / 1_000_000.0;
        format!("{:.1} Gb", gb)
    } else if size_kb >= 1_000 {
        // Megabytes
        let mb = size_kb as f64 / 1_000.0;
        format!("{:.1} Mb", mb)
    } else {
        let kb = size_kb as f64;
        // Kilobytes
        format!("{:.1} Kb", kb)
    }
}

/// Format a number with thousand separators (for legacy support)
fn format_with_grouping(num: u64) -> String {
    let s = num.to_string();
    let mut result = String::new();
    let len = s.len();

    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i) % 3 == 0 {
            result.push('\'');
        }
        result.push(c);
    }

    result
}

/// Print the complete disk usage analysis
#[pyfunction]
#[pyo3(signature = (dirname, inodes=false, no_grouping=false, no_f=false))]
fn print_disk_usage(
    py: Python,
    dirname: &str,
    inodes: bool,
    no_grouping: bool,
    no_f: bool,
) -> PyResult<()> {
    let max_marks = 20;

    // Verify directory exists and is accessible
    match fs::read_dir(dirname) {
        Err(e) if e.kind() == io::ErrorKind::PermissionDenied => {
            eprintln!(
                "Permission denied: Unable to access directory '{}'",
                dirname
            );
            return Err(PyErr::new::<pyo3::exceptions::PyPermissionError, _>(
                "Permission denied",
            ));
        }
        Err(e) => {
            eprintln!("Error accessing directory '{}': {}", dirname, e);
            return Err(PyErr::new::<pyo3::exceptions::PyOSError, _>(format!(
                "Error accessing directory: {}",
                e
            )));
        }
        Ok(_) => {}
    }

    // Calculate file sizes
    let mut file_sizes: Vec<(String, u64)> = Vec::new();
    let mut errors: Vec<(String, String)> = Vec::new();
    let mut permission_error = false;

    match calculate_directory_sizes(py, dirname, inodes) {
        Ok(raw_sizes) => {
            for (filename, size) in raw_sizes {
                let mut display_name = filename.clone();

                if !no_f {
                    let full_path = Path::new(dirname).join(&filename);
                    if let Ok(indicator) = get_file_type_indicator(&full_path.to_string_lossy()) {
                        display_name.push_str(&indicator);
                    }
                }

                file_sizes.push((display_name, size));
            }
        }
        Err(e) => {
            // Check if it's a keyboard interrupt
            if e.is_instance_of::<pyo3::exceptions::PyKeyboardInterrupt>(py) {
                // Just return the error without printing anything
                return Err(e);
            }

            let error_msg = e.to_string();
            if error_msg.contains("Permission denied") {
                errors.push(("Permission denied".to_string(), "<root>".to_string()));
                permission_error = true;
            } else {
                errors.push((error_msg, "<root>".to_string()));
            }
        }
    }

    // Sort by size
    file_sizes.sort_by_key(|k| k.1);

    let total_size: u64 = file_sizes.iter().map(|(_, s)| s).sum();
    let max_size = file_sizes
        .iter()
        .map(|(_, s)| s)
        .max()
        .copied()
        .unwrap_or(0);

    // Print header
    println!("Statistics of directory \"{}\" :\n", dirname);
    let col0_name = if inodes { "inodes" } else { "Size" };
    println!(
        "{:<14} {:<6} {:<20} {:<10}",
        col0_name, "In %", "Histogram", "Name"
    );

    // Print errors
    for (error, filename) in &errors {
        println!(
            "{} {:<10}",
            format!("{:<width$}", error, width = 22 + max_marks),
            filename
        );
    }

    // Print files
    for (filename, file_size) in &file_sizes {
        let nmarks = if max_size != 0 {
            ((max_marks - 1) as f64 * (*file_size as f64) / (max_size as f64)) as usize + 1
        } else {
            max_marks
        };

        let percentage = if total_size != 0 {
            100.0 * (*file_size as f64) / (total_size as f64)
        } else {
            100.0
        };
        let size_str = if inodes {
            // For inodes, use the old format with grouping
            if no_grouping {
                file_size.to_string()
            } else {
                format_with_grouping(*file_size)
            }
        } else {
            // For sizes, use K/M/G format
            format_size(*file_size)
        };

        let histogram = "#".repeat(nmarks);

        println!(
            "{:<14} {:<6.2} {:<20} {:<10}",
            size_str, percentage, histogram, filename
        );
    }

    // Print footer
    let total_str = if inodes {
        if no_grouping {
            total_size.to_string()
        } else {
            format_with_grouping(total_size)
        }
    } else {
        format_size(total_size)
    };
    println!("\nTotal directory size: {}", total_str);

    if permission_error {
        eprintln!("The Dustr has no permission to access certain subdirectories !\n");
    }

    Ok(())
}

/// Command-line arguments structure
#[derive(Parser)]
#[command(name = "dustr")]
#[command(about = "Show disk usage statistics", long_about = None)]
struct Cli {
    /// Directory to analyze
    #[arg(default_value = ".")]
    dirname: String,

    /// Count inodes instead of disk usage
    #[arg(short, long)]
    inodes: bool,

    /// Don't use thousand separators
    #[arg(short = 'g', long)]
    nogrouping: bool,

    /// Don't append file type indicators
    #[arg(short = 'f', long = "noF")]
    no_f: bool,
}

/// Main entry point for the dustr command
#[pyfunction]
#[pyo3(signature = (args=vec![]))]
fn main(py: Python, args: Vec<String>) -> PyResult<()> {
    // Parse arguments using clap
    let cli = match Cli::try_parse_from(std::iter::once("dustr".to_string()).chain(args)) {
        Ok(cli) => cli,
        Err(e) => {
            // Print clap's error message (includes help text for --help)
            eprintln!("{}", e);
            return Ok(());
        }
    };

    // Allow Ctrl+C by releasing GIL
    print_disk_usage(py, &cli.dirname, cli.inodes, cli.nogrouping, cli.no_f)
}

/// Python module definition
#[pymodule]
fn _dustr(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(calculate_directory_sizes, m)?)?;
    m.add_function(wrap_pyfunction!(get_file_type_indicator, m)?)?;
    m.add_function(wrap_pyfunction!(print_disk_usage, m)?)?;
    m.add_function(wrap_pyfunction!(main, m)?)?;
    Ok(())
}
