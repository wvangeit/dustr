use clap::Parser;
use jwalk::WalkDir as JWalkDir;
use parking_lot::Mutex;
use pyo3::prelude::*;
use rayon::prelude::*;
use signal_hook::consts::SIGINT;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Calculate directory sizes for all items in a directory (parallel version)
#[pyfunction]
#[pyo3(signature = (path, use_inodes, cross_mounts=false, verbose=false, live=false))]
fn calculate_directory_sizes(
    py: Python,
    path: &str,
    use_inodes: bool,
    cross_mounts: bool,
    verbose: bool,
    live: bool,
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

    // Compute the base directory device id once for mount boundary checks
    let base_dev = if !cross_mounts {
        match fs::metadata(base_path) {
            Ok(m) => Some(m.dev()),
            Err(e) => {
                return Err(PyErr::new::<pyo3::exceptions::PyOSError, _>(format!(
                    "Cannot read metadata for '{}': {}",
                    path, e
                )));
            }
        }
    } else {
        None
    };

    // Shared state for progress and cancellation
    let progress = Arc::new(AtomicUsize::new(0));
    let cancelled = Arc::new(AtomicBool::new(false));
    let results: Arc<Mutex<HashMap<String, u64>>> = Arc::new(Mutex::new(HashMap::new()));
    let current_entry: Arc<Mutex<String>> = Arc::new(Mutex::new(String::new()));

    // Register OS signal handler to set cancelled flag directly on Ctrl+C.
    // This is instant (no GIL needed, no thread polling) and additive
    // (Python's own SIGINT handler still fires for KeyboardInterrupt).
    let signal_id = signal_hook::flag::register(SIGINT, cancelled.clone()).ok();

    // Spawn live display thread if requested
    let live_last_lines = Arc::new(AtomicUsize::new(0));
    let live_display = if live {
        let results_for_display = results.clone();
        let cancelled_for_display = cancelled.clone();
        let progress_for_display = progress.clone();
        let last_lines_for_display = live_last_lines.clone();
        let dirname = path.to_string();
        Some(std::thread::spawn(move || {
            let mut last_lines = 0usize;
            loop {
                std::thread::sleep(std::time::Duration::from_millis(500));
                if cancelled_for_display.load(Ordering::Relaxed) {
                    break;
                }
                let snapshot: Vec<(String, u64)> = {
                    let r = results_for_display.lock();
                    r.iter().map(|(k, v)| (k.clone(), *v)).collect()
                };
                let current = progress_for_display.load(Ordering::Relaxed);
                let table = render_stats_table(
                    &dirname,
                    &snapshot,
                    use_inodes,
                    false,
                    current,
                    total_entries,
                );
                let bar = format_progress_bar(current, total_entries);
                // Move cursor up to overwrite previous output, then print
                if last_lines > 0 {
                    eprint!("\x1b[{}A\x1b[J", last_lines);
                }
                eprintln!("{}{}", table, bar);
                io::stderr().flush().ok();
                last_lines = table.lines().count() + 1;
                last_lines_for_display.store(last_lines, Ordering::Relaxed);
            }
        }))
    } else {
        None
    };

    // Process entries in parallel, releasing the GIL
    py.detach(|| {
        entries_vec.par_iter().for_each(|entry| {
            // Check for cancellation
            if cancelled.load(Ordering::Relaxed) {
                return;
            }

            let file_name = entry.file_name().to_string_lossy().to_string();
            let file_path = entry.path();

            if verbose {
                *current_entry.lock() = file_name.clone();
            }

            let size = if use_inodes {
                count_inodes_parallel(&file_path, &cancelled, base_dev, &current_entry)
            } else {
                calculate_size_kb_parallel(&file_path, &cancelled, base_dev, &current_entry)
            };

            if !cancelled.load(Ordering::Relaxed) {
                results.lock().insert(file_name, size);
            }

            // Update progress periodically (skip equality check to avoid race condition)
            let current = progress.fetch_add(1, Ordering::Relaxed) + 1;
            if !live && current.is_multiple_of(10) {
                let entry_name = if verbose {
                    Some(current_entry.lock().clone())
                } else {
                    None
                };
                print_progress(current, total_entries, entry_name.as_deref());
            }
        });
    });

    // Ensure final progress state is shown after parallel iteration completes
    if !live {
        print_progress(total_entries, total_entries, None);
    }

    // Unregister our signal handler now that computation is done
    if let Some(id) = signal_id {
        signal_hook::low_level::unregister(id);
    }
    // Stop the live display thread
    cancelled.store(true, Ordering::Relaxed);
    if let Some(handle) = live_display {
        let _ = handle.join();
    }

    // Check for Python signals after computation
    if let Err(e) = py.check_signals() {
        eprint!("\r{}\r", " ".repeat(80));
        io::stderr().flush().ok();
        return Err(e);
    }

    // Clear progress bar / live display
    if live {
        // Use the actual line count last printed by the live thread
        // so we don't overshoot and erase previous stdout output.
        let lines = live_last_lines.load(Ordering::Relaxed);
        if lines > 0 {
            eprint!("\x1b[{}A\x1b[J", lines);
        }
    } else {
        eprint!("\r{}\r", " ".repeat(80));
    }
    io::stderr().flush().ok();

    let final_results = match Arc::try_unwrap(results) {
        Ok(mutex) => mutex.into_inner(),
        Err(arc) => arc.lock().clone(),
    };

    Ok(final_results)
}

/// Calculate total size in kilobytes using parallel jwalk
fn calculate_size_kb_parallel(
    path: &Path,
    cancelled: &AtomicBool,
    base_dev: Option<u64>,
    current_entry: &Mutex<String>,
) -> u64 {
    if path.is_file() {
        return fs::metadata(path)
            .map(|m| (m.blocks() * 512).div_ceil(1024))
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
        // Check cancellation every iteration (atomic relaxed load is ~1ns, negligible vs I/O)
        if cancelled.load(Ordering::Relaxed) {
            break;
        }
        count += 1;
        // Skip entries on different filesystems
        if let Some(dev) = base_dev {
            match entry.metadata() {
                Ok(m) => {
                    if m.dev() != dev {
                        continue;
                    }
                }
                Err(_) => continue,
            }
        }
        if entry.file_type().is_dir() {
            if count % 100 == 0 {
                *current_entry.lock() = entry.path().to_string_lossy().to_string();
            }
        } else if entry.file_type().is_file() {
            total += entry
                .metadata()
                .map(|m| (m.blocks() * 512).div_ceil(1024))
                .unwrap_or(0);
        }
    }
    total
}

/// Count inodes using parallel jwalk
fn count_inodes_parallel(
    path: &Path,
    cancelled: &AtomicBool,
    base_dev: Option<u64>,
    current_entry: &Mutex<String>,
) -> u64 {
    if !path.is_dir() {
        return 1;
    }

    let mut count: u64 = 0;
    let mut iter_count = 0;
    for entry in JWalkDir::new(path)
        .parallelism(jwalk::Parallelism::RayonNewPool(num_cpus()))
        .into_iter()
        .filter_map(|e| e.ok())
    {
        // Check cancellation every iteration (atomic relaxed load is ~1ns, negligible vs I/O)
        if cancelled.load(Ordering::Relaxed) {
            break;
        }
        iter_count += 1;
        // Skip entries on different filesystems
        if let Some(dev) = base_dev {
            match entry.metadata() {
                Ok(m) => {
                    if m.dev() != dev {
                        continue;
                    }
                }
                Err(_) => continue,
            }
        }
        if entry.file_type().is_dir() && iter_count % 100 == 0 {
            *current_entry.lock() = entry.path().to_string_lossy().to_string();
        }
        count += 1;
    }
    count
}

/// Get the number of CPUs for parallelism
fn num_cpus() -> usize {
    std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(4)
}

/// Render the statistics table as a string (used for live display)
fn render_stats_table(
    dirname: &str,
    entries: &[(String, u64)],
    inodes: bool,
    no_grouping: bool,
    current: usize,
    total: usize,
) -> String {
    let max_marks = 20;
    let mut sorted: Vec<(String, u64)> = entries.to_vec();
    sorted.sort_by_key(|k| k.1);

    let total_size: u64 = sorted.iter().map(|(_, s)| s).sum();
    let max_size = sorted.iter().map(|(_, s)| s).max().copied().unwrap_or(0);

    let col0_name = if inodes { "inodes" } else { "Size" };
    let mut out = format!(
        "Statistics of directory \"{}\" ({}/{}):\n\n{:<14} {:<6} {:<20} {:<10}\n",
        dirname, current, total, col0_name, "In %", "Histogram", "Name"
    );

    for (filename, file_size) in &sorted {
        let nmarks = if max_size != 0 {
            ((max_marks - 1) as f64 * (*file_size as f64) / (max_size as f64)) as usize + 1
        } else {
            max_marks
        };
        let percentage = if total_size != 0 {
            100.0 * (*file_size as f64) / (total_size as f64)
        } else {
            0.0
        };
        let size_str = if inodes {
            if no_grouping {
                file_size.to_string()
            } else {
                format_with_grouping(*file_size)
            }
        } else {
            format_size(*file_size)
        };
        let histogram = "#".repeat(nmarks);
        out.push_str(&format!(
            "{:<14} {:<6.2} {:<20} {:<10}\n",
            size_str, percentage, histogram, filename
        ));
    }

    let total_str = if inodes {
        if no_grouping {
            total_size.to_string()
        } else {
            format_with_grouping(total_size)
        }
    } else {
        format_size(total_size)
    };
    out.push_str(&format!("\nTotal directory size: {}\n", total_str));
    out
}

/// Width of the rendered progress bar (number of characters between the brackets).
const BAR_WIDTH: usize = 40;

/// Format a progress bar as a string (no trailing newline)
fn format_progress_bar(current: usize, total: usize) -> String {
    let progress = if total > 0 {
        current as f64 / total as f64
    } else {
        0.0
    };
    let filled = (BAR_WIDTH as f64 * progress) as usize;
    let empty = BAR_WIDTH - filled;
    format!(
        "[{}{}] {}/{}",
        ">".repeat(filled),
        "-".repeat(empty),
        current,
        total
    )
}

/// Print a progress bar to stderr
fn print_progress(current: usize, total: usize, current_entry: Option<&str>) {
    let bar = format_progress_bar(current, total);

    match current_entry {
        Some(name) => {
            // Truncate long names to fit in terminal
            let max_name_len = 30;
            let display_name = if name.len() > max_name_len {
                format!("{}...", &name[..max_name_len - 3])
            } else {
                name.to_string()
            };
            eprint!(
                "\r{blank}\r{bar} {name}",
                blank = " ".repeat(80),
                bar = bar,
                name = display_name,
            );
        }
        None => {
            eprint!("\r{}", bar);
        }
    }
    io::stderr().flush().ok();
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
/// Format size with units (KB, MB, GB, TB)
fn format_size(size_kb: u64) -> String {
    if size_kb >= 1_000_000_000 {
        let tb = size_kb as f64 / 1_000_000_000.0;
        format!("{:.1} TB", tb)
    } else if size_kb >= 1_000_000 {
        let gb = size_kb as f64 / 1_000_000.0;
        format!("{:.1} GB", gb)
    } else if size_kb >= 1_000 {
        let mb = size_kb as f64 / 1_000.0;
        format!("{:.1} MB", mb)
    } else {
        let kb = size_kb as f64;
        format!("{:.1} KB", kb)
    }
}

/// Format a number with thousand separators (for legacy support)
fn format_with_grouping(num: u64) -> String {
    let s = num.to_string();
    let mut result = String::new();
    let len = s.len();

    for (i, c) in s.chars().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            result.push('\'');
        }
        result.push(c);
    }

    result
}

/// Escape a string for JSON output
fn json_escape(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}

/// Print the complete disk usage analysis
#[pyfunction]
#[pyo3(signature = (dirname, inodes=false, no_grouping=false, no_f=false, json=false, cross_mounts=false, verbose=false, live=false))]
#[allow(clippy::too_many_arguments)]
fn print_disk_usage(
    py: Python,
    dirname: &str,
    inodes: bool,
    no_grouping: bool,
    no_f: bool,
    json: bool,
    cross_mounts: bool,
    verbose: bool,
    live: bool,
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

    match calculate_directory_sizes(py, dirname, inodes, cross_mounts, verbose, live) {
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

    if json {
        let mode = if inodes { "inodes" } else { "size" };
        println!("{{");
        println!("  \"directory\": \"{}\",", json_escape(dirname));
        println!("  \"mode\": \"{}\",", mode);
        println!("  \"entries\": [");
        for (i, (name, size)) in file_sizes.iter().enumerate() {
            let percentage = if total_size != 0 {
                100.0 * (*size as f64) / (total_size as f64)
            } else {
                0.0
            };
            let comma = if i + 1 < file_sizes.len() { "," } else { "" };
            println!(
                "    {{\"name\": \"{}\", \"value\": {}, \"percentage\": {:.2}}}{}",
                json_escape(name),
                size,
                percentage,
                comma
            );
        }
        println!("  ],");
        println!("  \"total\": {}", total_size);
        println!("}}");

        return Ok(());
    }

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
        println!("{:<width$} {:<10}", error, filename, width = 22 + max_marks);
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
            0.0
        };
        let size_str = if inodes {
            if no_grouping {
                file_size.to_string()
            } else {
                format_with_grouping(*file_size)
            }
        } else {
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

    /// Output results as JSON
    #[arg(short, long)]
    json: bool,

    /// Cross mount boundaries (by default stays on the same filesystem)
    #[arg(short = 'x', long)]
    cross_mounts: bool,

    /// Show directories being traversed
    #[arg(short, long)]
    verbose: bool,

    /// Live-update statistics table during traversal
    #[arg(short, long)]
    live: bool,
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
    print_disk_usage(
        py,
        &cli.dirname,
        cli.inodes,
        cli.nogrouping,
        cli.no_f,
        cli.json,
        cli.cross_mounts,
        cli.verbose,
        cli.live,
    )
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn progress_bar_zero_total() {
        // Avoid divide-by-zero; should render an empty bar.
        let bar = format_progress_bar(0, 0);
        assert_eq!(bar, format!("[{}] 0/0", "-".repeat(BAR_WIDTH)));
    }

    #[test]
    fn progress_bar_half() {
        let bar = format_progress_bar(5, 10);
        let filled = BAR_WIDTH / 2;
        assert_eq!(
            bar,
            format!(
                "[{}{}] 5/10",
                ">".repeat(filled),
                "-".repeat(BAR_WIDTH - filled)
            )
        );
    }

    #[test]
    fn progress_bar_full() {
        let bar = format_progress_bar(10, 10);
        assert_eq!(bar, format!("[{}] 10/10", ">".repeat(BAR_WIDTH)));
    }

    #[test]
    fn progress_bar_no_newline() {
        // The live view appends its own newline; the bar itself must not.
        let bar = format_progress_bar(3, 7);
        assert!(!bar.contains('\n'));
    }
}
