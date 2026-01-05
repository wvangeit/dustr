#!/usr/bin/env python3
"""Dustr - Rust-based disk usage analyzer"""

from __future__ import print_function

import os
import subprocess
import sys
import warnings

try:
    from dustr._dustr import main as rust_main
except ImportError as e:
    print(f"Error: Failed to import Rust extension: {e}", file=sys.stderr)
    print("Please ensure the package is properly installed.", file=sys.stderr)
    sys.exit(1)


def main():
    """Main entry point - delegates to Rust implementation"""
    try:
        rust_main(sys.argv[1:])
    except KeyboardInterrupt:
        print("\nThe Dustr was shot by the user !")
        warnings.filterwarnings("ignore")
        sys.exit(1)
    except Exception as e:  # pylint: disable=broad-except
        print("Sorry, the Dustr was eaten by the Python !\nReason:", sys.exc_info()[0])
        if isinstance(e, SystemExit):
            raise


def show_man():
    """Show the dustr man page"""
    import sysconfig

    # Try to find the man page in possible locations
    possible_paths = [
        # Installed via pip in data directory
        os.path.join(sysconfig.get_path("data"), "share", "man", "man1", "dustr.1"),
        # User local installation
        os.path.expanduser("~/.local/share/man/man1/dustr.1"),
        # Development mode - relative to package
        os.path.join(os.path.dirname(__file__), "..", "..", "man", "dustr.1"),
    ]

    man_page = None
    for path in possible_paths:
        if os.path.exists(path):
            man_page = path
            break

    if man_page:
        try:
            subprocess.run(["man", man_page], check=True)
        except (subprocess.CalledProcessError, FileNotFoundError):
            # If man command fails, try to display the file directly
            with open(man_page, "r") as f:
                print(f.read())
    else:
        print("Man page not found. Install locations tried:", file=sys.stderr)
        for path in possible_paths:
            print(f"  - {path}", file=sys.stderr)


if __name__ == "__main__":
    main()
