#!/usr/bin/env python3
"""Tests for dustr"""

import os
import tempfile
import shutil
from pathlib import Path
from dustr._dustr import calculate_directory_sizes, get_file_type_indicator


def test_calculate_directory_sizes():
    """Test basic directory size calculation"""
    # Create a temporary directory structure
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create some test files
        (Path(tmpdir) / "file1.txt").write_text("Hello" * 100)
        (Path(tmpdir) / "file2.txt").write_text("World" * 200)

        # Create a subdirectory with files
        subdir = Path(tmpdir) / "subdir"
        subdir.mkdir()
        (subdir / "file3.txt").write_text("Test" * 300)

        # Calculate sizes
        sizes = calculate_directory_sizes(tmpdir, use_inodes=False)

        # Verify we got results for all items
        assert "file1.txt" in sizes
        assert "file2.txt" in sizes
        assert "subdir" in sizes

        # Verify sizes are reasonable (in kilobytes)
        assert sizes["file1.txt"] >= 0
        assert sizes["file2.txt"] >= sizes["file1.txt"]  # file2 has more content
        assert sizes["subdir"] > 0


def test_calculate_directory_sizes_inodes():
    """Test inode counting"""
    with tempfile.TemporaryDirectory() as tmpdir:
        # Create test structure
        (Path(tmpdir) / "file1.txt").write_text("x")
        subdir = Path(tmpdir) / "subdir"
        subdir.mkdir()
        (subdir / "file2.txt").write_text("y")
        (subdir / "file3.txt").write_text("z")

        # Calculate inodes
        inodes = calculate_directory_sizes(tmpdir, use_inodes=True)

        # Verify inode counts
        assert inodes["file1.txt"] == 1  # Single file
        assert inodes["subdir"] == 3  # Directory + 2 files


def test_get_file_type_indicator():
    """Test file type indicators"""
    with tempfile.TemporaryDirectory() as tmpdir:
        tmppath = Path(tmpdir)

        # Create a file
        file_path = tmppath / "testfile.txt"
        file_path.write_text("test")
        assert get_file_type_indicator(str(file_path)) == ""

        # Create a directory
        dir_path = tmppath / "testdir"
        dir_path.mkdir()
        assert get_file_type_indicator(str(dir_path)) == "/"

        # Create a symlink
        link_path = tmppath / "testlink"
        link_path.symlink_to(file_path)
        assert get_file_type_indicator(str(link_path)) == "@"


def test_nonexistent_directory():
    """Test that nonexistent directory raises error"""
    try:
        calculate_directory_sizes("/nonexistent/path/that/does/not/exist", False)
        assert False, "Should have raised an error"
    except Exception as e:
        assert "not found" in str(e).lower()


def test_permission_denied():
    """Test handling of permission denied errors"""
    # This test is platform-dependent and might not work everywhere
    # On Unix systems, we can create a directory and remove read permissions
    if os.name == "posix":
        with tempfile.TemporaryDirectory() as tmpdir:
            protected = Path(tmpdir) / "protected"
            protected.mkdir()
            os.chmod(protected, 0o000)

            try:
                calculate_directory_sizes(str(protected), False)
                assert False, "Should have raised permission error"
            except PermissionError:
                pass
            finally:
                # Restore permissions for cleanup
                os.chmod(protected, 0o755)


if __name__ == "__main__":
    test_calculate_directory_sizes()
    test_calculate_directory_sizes_inodes()
    test_get_file_type_indicator()
    test_nonexistent_directory()
    test_permission_denied()
    print("All tests passed!")
