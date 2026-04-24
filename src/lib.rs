pub mod core;

#[cfg(feature = "extension-module")]
mod python {
    use clap::Parser;
    use pyo3::prelude::*;
    use std::collections::HashMap;

    use crate::core::DustrError;

    /// Convert a DustrError to a PyErr
    fn to_pyerr(_py: Python, e: DustrError) -> PyErr {
        match e {
            DustrError::NotFound(msg) => {
                PyErr::new::<pyo3::exceptions::PyFileNotFoundError, _>(msg)
            }
            DustrError::PermissionDenied(msg) => {
                PyErr::new::<pyo3::exceptions::PyPermissionError, _>(msg)
            }
            DustrError::OsError(msg) => PyErr::new::<pyo3::exceptions::PyOSError, _>(msg),
            DustrError::Cancelled => PyErr::new::<pyo3::exceptions::PyKeyboardInterrupt, _>(""),
        }
    }

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
        let result = py.detach(|| {
            crate::core::calculate_directory_sizes(path, use_inodes, cross_mounts, verbose, live)
        });

        py.check_signals()?;

        result.map_err(|e| to_pyerr(py, e))
    }

    /// Get file type indicator (@ for symlinks, / for directories, empty for files)
    #[pyfunction]
    fn get_file_type_indicator(path: &str) -> PyResult<String> {
        Ok(crate::core::get_file_type_indicator(path))
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
        let result = py.detach(|| {
            crate::core::print_disk_usage(
                dirname,
                inodes,
                no_grouping,
                no_f,
                json,
                cross_mounts,
                verbose,
                live,
            )
        });

        py.check_signals()?;

        result.map_err(|e| to_pyerr(py, e))
    }

    /// Command-line arguments structure
    #[derive(Parser)]
    #[command(name = "dustr")]
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

    /// Main entry point for the dustr command (called from Python)
    #[pyfunction]
    #[pyo3(signature = (args=vec![]))]
    fn main(py: Python, args: Vec<String>) -> PyResult<()> {
        let cli = match Cli::try_parse_from(std::iter::once("dustr".to_string()).chain(args)) {
            Ok(cli) => cli,
            Err(e) => {
                eprintln!("{}", e);
                return Ok(());
            }
        };

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
    pub fn _dustr(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add_function(wrap_pyfunction!(calculate_directory_sizes, m)?)?;
        m.add_function(wrap_pyfunction!(get_file_type_indicator, m)?)?;
        m.add_function(wrap_pyfunction!(print_disk_usage, m)?)?;
        m.add_function(wrap_pyfunction!(main, m)?)?;
        Ok(())
    }
}

#[cfg(feature = "extension-module")]
pub use python::_dustr;

#[cfg(test)]
mod tests {
    use crate::core::{format_progress_bar, BAR_WIDTH};

    #[test]
    fn progress_bar_zero_total() {
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
        let bar = format_progress_bar(3, 7);
        assert!(!bar.contains('\n'));
    }
}
