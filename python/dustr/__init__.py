#!/usr/bin/env python3
"""Dustr - Rust-based disk usage analyzer"""

from __future__ import print_function

import argparse
import os
import sys
import warnings
from collections import OrderedDict

try:
    from dustr._dustr import calculate_directory_sizes, get_file_type_indicator
except ImportError as e:
    print(f"Error: Failed to import Rust extension: {e}", file=sys.stderr)
    print("Please ensure the package is properly installed.", file=sys.stderr)
    sys.exit(1)


def _print_error_files(errors, max_marks):
    """Print the lines with files that generated errors"""

    fmt_error = "{0:<%d} {1:<10}" % (22 + max_marks)

    permission_error = False

    for filename in errors:
        error = errors[filename]

        if "Permission denied" in error:
            error_string = "Permission denied"
            permission_error = True
        else:
            error_string = error

        print(fmt_error.format(error_string, filename))

    return permission_error


def print_normal_files(file_sizes, max_marks, total_size, fmt, args):
    """Print the lines with files that generated no errors"""

    maxsize = max(file_sizes.values()) if file_sizes.values() else 0

    for filename in file_sizes:
        file_size = file_sizes[filename]
        if maxsize != 0:
            nmarks = int((max_marks - 1) * float(file_size) / maxsize) + 1
            percentage = 100 * float(file_size) / total_size
        else:
            nmarks = max_marks
            percentage = 100.0
        if not args.nogrouping:
            file_size = f"{file_size:,}"
        print(
            fmt.format(
                file_size,
                f"{percentage:02.2f}",
                "".join(["#"] * nmarks),
                filename,
            )
        )


def calculate_file_sizes(dirname, args):
    """Calculate the file sizes using Rust backend"""

    file_sizes = {}
    errors = {}

    try:
        # Use the Rust function to calculate all sizes at once
        raw_sizes = calculate_directory_sizes(dirname, args.inodes)

        # Add file type indicators if requested
        for filename, size in raw_sizes.items():
            display_name = filename
            if not args.noF:
                full_path = os.path.join(dirname, filename)
                try:
                    indicator = get_file_type_indicator(full_path)
                    display_name = filename + indicator
                except Exception as e:
                    # If we can't get the indicator, just use the filename
                    pass

            file_sizes[display_name] = size

    except PermissionError as e:
        errors["<root>"] = f"Permission denied: {e}"
    except FileNotFoundError as e:
        errors["<root>"] = f"Directory not found: {e}"
    except Exception as e:
        errors["<root>"] = str(e)

    file_sizes = OrderedDict(sorted(file_sizes.items(), key=lambda x: x[1]))

    return file_sizes, errors


def print_header(dirname, fmt, args):
    """Print header string"""

    print(f'\n\nStatistics of directory "{dirname}" :\n')

    if args.inodes:
        col0_name = "inodes"
    else:
        col0_name = "in kByte"

    print(fmt.format(col0_name, "in %", "histogram", "name"))


def print_tail(total_size, permission_error, args):
    """Print tail string"""

    if not args.nogrouping:
        total_size = f"{total_size:,}"
    print(f"\nTotal directory size: {total_size} kByte\n")

    if permission_error:
        print(
            "The Ducky has no permission to access certain subdirectories !\n",
            file=sys.stderr,
        )


def print_progress(progress, total_bar_size):
    """Print a progress bar"""

    bar_size = int(round(total_bar_size * progress))
    pbar = f"\r{'>' * bar_size}{'-' * (total_bar_size - bar_size)}"
    sys.stdout.write(pbar)
    sys.stdout.flush()


def parse_arguments():
    """Parse command line arguments"""

    parser = argparse.ArgumentParser(
        description="Show disk usage statistics (Rust implementation)"
    )
    parser.add_argument(
        "dirname",
        metavar="dirname",
        type=str,
        nargs="?",
        default=".",
        help="Directory name",
    )
    parser.add_argument("--nogrouping", action="store_true")
    parser.add_argument("--noprogress", action="store_true")
    parser.add_argument("--inodes", action="store_true", default=False)
    parser.add_argument("--noF", action="store_true")
    return parser.parse_args()


def main():
    """Main"""

    args = parse_arguments()
    max_marks = 20
    fmt = f"{{0:<14}} {{1:<6}} {{2:<{max_marks}}} {{3:<10}}"

    try:
        files_list = os.listdir(args.dirname)
    except PermissionError:
        print(
            f"Permission denied: Unable to access directory '{args.dirname}'",
            file=sys.stderr,
        )
        sys.exit(1)
    except Exception as e:
        print(f"Error accessing directory '{args.dirname}': {e}", file=sys.stderr)
        sys.exit(1)

    file_sizes, errors = calculate_file_sizes(args.dirname, args)

    total_size = sum(file_sizes.values())

    print_header(args.dirname, fmt, args)
    permission_error = _print_error_files(errors, max_marks)
    print_normal_files(file_sizes, max_marks, total_size, fmt, args)
    print_tail(total_size, permission_error, args)


if __name__ == "__main__":
    try:
        main()
    except KeyboardInterrupt:
        print("\nThe Duck was shot by the user !")
        warnings.filterwarnings("ignore")
        sys.exit(1)
    except Exception as e:
        print("Sorry, the Duck was eaten by the Python !\nReason:", sys.exc_info()[0])
        if isinstance(e, SystemExit):
            raise
