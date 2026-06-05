use jwalk::WalkDir as JWalkDir;
use parking_lot::Mutex;
use rayon::prelude::*;
use signal_hook::consts::SIGINT;
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::os::unix::fs::MetadataExt;
use std::path::Path;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::Arc;

/// Error type for core dustr operations
#[derive(Debug)]
pub enum DustrError {
    NotFound(String),
    PermissionDenied(String),
    OsError(String),
    Cancelled,
}

impl std::fmt::Display for DustrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DustrError::NotFound(msg) => write!(f, "{}", msg),
            DustrError::PermissionDenied(msg) => write!(f, "{}", msg),
            DustrError::OsError(msg) => write!(f, "{}", msg),
            DustrError::Cancelled => write!(f, "Cancelled"),
        }
    }
}

impl std::error::Error for DustrError {}

/// Shared CLI arguments (used by both the binary and the Python entry point)
#[derive(clap::Parser, Debug)]
#[command(about = "Show disk usage statistics", long_about = None)]
pub struct Cli {
    /// Directory to analyze
    #[arg(default_value = ".")]
    pub dirname: String,

    /// Count inodes instead of disk usage
    #[arg(short, long)]
    pub inodes: bool,

    /// Don't use thousand separators
    #[arg(short = 'g', long)]
    pub nogrouping: bool,

    /// Don't append file type indicators
    #[arg(short = 'f', long = "noF")]
    pub no_f: bool,

    /// Output results as JSON
    #[arg(short, long)]
    pub json: bool,

    /// Cross mount boundaries (by default stays on the same filesystem)
    #[arg(short = 'x', long)]
    pub cross_mounts: bool,

    /// Show directories being traversed
    #[arg(short, long)]
    pub verbose: bool,

    /// Live-update statistics table during traversal
    #[arg(short, long)]
    pub live: bool,
}

/// Calculate directory sizes for all items in a directory (parallel version)
pub fn calculate_directory_sizes(
    path: &str,
    use_inodes: bool,
    cross_mounts: bool,
    verbose: bool,
    live: bool,
) -> Result<HashMap<String, u64>, DustrError> {
    let base_path = Path::new(path);

    if !base_path.exists() {
        return Err(DustrError::NotFound(format!(
            "Directory not found: {}",
            path
        )));
    }

    if !base_path.is_dir() {
        return Err(DustrError::OsError(format!("Not a directory: {}", path)));
    }

    let entries = match fs::read_dir(base_path) {
        Ok(entries) => entries,
        Err(e) => match e.kind() {
            io::ErrorKind::NotFound => {
                return Err(DustrError::NotFound(format!(
                    "Directory not found: {}",
                    path
                )));
            }
            io::ErrorKind::PermissionDenied => {
                return Err(DustrError::PermissionDenied(format!(
                    "Permission denied: {}",
                    e
                )));
            }
            _ => {
                return Err(DustrError::OsError(format!(
                    "Cannot read directory '{}': {}",
                    path, e
                )));
            }
        },
    };

    // Collect entries first to get count
    let entries_vec: Vec<_> = entries.flatten().collect();
    let total_entries = entries_vec.len();

    // Compute the base directory device id once for mount boundary checks
    let base_dev = if !cross_mounts {
        match fs::metadata(base_path) {
            Ok(m) => Some(m.dev()),
            Err(e) => {
                return Err(DustrError::OsError(format!(
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
    let signal_id = match signal_hook::flag::register(SIGINT, cancelled.clone()) {
        Ok(id) => Some(id),
        Err(e) => {
            if verbose {
                eprintln!(
                    "Warning: failed to register SIGINT handler: {}. \
                     Ctrl+C responsiveness may be reduced.",
                    e
                );
            }
            None
        }
    };

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

    // Process entries in parallel
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
            count_inodes(&file_path, &cancelled, base_dev, &current_entry)
        } else {
            calculate_size_kb(&file_path, &cancelled, base_dev, &current_entry)
        };

        if !cancelled.load(Ordering::Relaxed) {
            results.lock().insert(file_name, size);
        }

        // Update progress periodically
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

    // Ensure final progress state is shown after parallel iteration completes
    if !live {
        print_progress(total_entries, total_entries, None);
    }

    // Unregister our signal handler now that computation is done
    if let Some(id) = signal_id {
        signal_hook::low_level::unregister(id);
    }
    // Stop the live display thread
    let was_cancelled = cancelled.load(Ordering::Relaxed);
    cancelled.store(true, Ordering::Relaxed);
    if let Some(handle) = live_display {
        let _ = handle.join();
    }

    // Check if we were cancelled by SIGINT
    if was_cancelled {
        // Clear progress bar
        eprint!("\r{}\r", " ".repeat(80));
        io::stderr().flush().ok();
        return Err(DustrError::Cancelled);
    }

    // Clear progress bar / live display
    if live {
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

/// Calculate total size in kilobytes by walking the tree serially.
/// The caller's rayon `par_iter` already provides top-level parallelism;
/// using Serial here avoids nested thread-pool oversubscription.
fn calculate_size_kb(
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

    let mut total: u64 = 0;
    let mut count = 0;
    for entry in JWalkDir::new(path)
        .parallelism(jwalk::Parallelism::Serial)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if cancelled.load(Ordering::Relaxed) {
            break;
        }
        count += 1;
        // Fetch metadata once and reuse for both the device check and block count.
        let meta = match entry.metadata() {
            Ok(m) => m,
            Err(_) => continue,
        };
        if let Some(dev) = base_dev {
            if meta.dev() != dev {
                continue;
            }
        }
        if entry.file_type().is_dir() {
            if count % 100 == 0 {
                *current_entry.lock() = entry.path().to_string_lossy().to_string();
            }
        } else if entry.file_type().is_file() {
            total += (meta.blocks() * 512).div_ceil(1024);
        }
    }
    total
}

/// Count inodes by walking the tree serially.
/// The caller's rayon `par_iter` already provides top-level parallelism;
/// using Serial here avoids nested thread-pool oversubscription.
fn count_inodes(
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
        .parallelism(jwalk::Parallelism::Serial)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        if cancelled.load(Ordering::Relaxed) {
            break;
        }
        iter_count += 1;
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

/// Render the statistics table as a string (used for live display)
pub fn render_stats_table(
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
pub const BAR_WIDTH: usize = 40;

/// Format a progress bar as a string (no trailing newline)
pub fn format_progress_bar(current: usize, total: usize) -> String {
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
pub fn print_progress(current: usize, total: usize, current_entry: Option<&str>) {
    let bar = format_progress_bar(current, total);

    match current_entry {
        Some(name) => {
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
pub fn get_file_type_indicator(path: &str) -> String {
    let p = Path::new(path);

    if p.is_symlink() {
        "@".to_string()
    } else if p.is_dir() {
        "/".to_string()
    } else {
        "".to_string()
    }
}

/// Format size with units (KB, MB, GB, TB)
pub fn format_size(size_kb: u64) -> String {
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

/// Format a number with thousand separators
pub fn format_with_grouping(num: u64) -> String {
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

/// Escape a string for JSON output, including all ASCII control characters.
pub fn json_escape(s: &str) -> String {
    let mut result = String::with_capacity(s.len());
    for c in s.chars() {
        match c {
            '\\' => result.push_str("\\\\"),
            '"' => result.push_str("\\\""),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                use std::fmt::Write as _;
                let _ = write!(result, "\\u{:04x}", c as u32);
            }
            c => result.push(c),
        }
    }
    result
}

/// Print the complete disk usage analysis
#[allow(clippy::too_many_arguments)]
pub fn print_disk_usage(
    dirname: &str,
    inodes: bool,
    no_grouping: bool,
    no_f: bool,
    json: bool,
    cross_mounts: bool,
    verbose: bool,
    live: bool,
) -> Result<(), DustrError> {
    let max_marks = 20;

    // Calculate file sizes
    let mut file_sizes: Vec<(String, u64)> = Vec::new();

    let raw_sizes = calculate_directory_sizes(dirname, inodes, cross_mounts, verbose, live)?;
    for (filename, size) in raw_sizes {
        let mut display_name = filename.clone();

        if !no_f {
            let full_path = Path::new(dirname).join(&filename);
            let indicator = get_file_type_indicator(&full_path.to_string_lossy());
            display_name.push_str(&indicator);
        }

        file_sizes.push((display_name, size));
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

    Ok(())
}
