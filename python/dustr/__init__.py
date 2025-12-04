#!/usr/bin/env python3
"""Dustr - Rust-based disk usage analyzer"""

from __future__ import print_function

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


if __name__ == "__main__":
    main()
