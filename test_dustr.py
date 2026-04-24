#!/usr/bin/env python3
"""Tests for dustr"""

import os
import tempfile
import shutil
import signal
import subprocess
import sys
import time
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


def test_cross_mounts():
    """Test that cross_mounts parameter is accepted and results match on same fs"""
    with tempfile.TemporaryDirectory() as tmpdir:
        (Path(tmpdir) / "file1.txt").write_text("Hello" * 100)
        subdir = Path(tmpdir) / "subdir"
        subdir.mkdir()
        (subdir / "file2.txt").write_text("World" * 200)

        # Both should give the same results on a single filesystem
        sizes_default = calculate_directory_sizes(tmpdir, use_inodes=False)
        sizes_no_cross = calculate_directory_sizes(
            tmpdir, use_inodes=False, cross_mounts=False
        )
        sizes_cross = calculate_directory_sizes(
            tmpdir, use_inodes=False, cross_mounts=True
        )

        assert sizes_default == sizes_no_cross
        assert sizes_default == sizes_cross

        # Same for inodes
        inodes_default = calculate_directory_sizes(tmpdir, use_inodes=True)
        inodes_no_cross = calculate_directory_sizes(
            tmpdir, use_inodes=True, cross_mounts=False
        )
        inodes_cross = calculate_directory_sizes(
            tmpdir, use_inodes=True, cross_mounts=True
        )

        assert inodes_default == inodes_no_cross
        assert inodes_default == inodes_cross


def test_verbose():
    """Test that verbose parameter is accepted and results are unchanged"""
    with tempfile.TemporaryDirectory() as tmpdir:
        (Path(tmpdir) / "file1.txt").write_text("Hello" * 100)
        subdir = Path(tmpdir) / "subdir"
        subdir.mkdir()
        (subdir / "file2.txt").write_text("World" * 200)

        sizes_normal = calculate_directory_sizes(tmpdir, use_inodes=False)
        sizes_verbose = calculate_directory_sizes(
            tmpdir, use_inodes=False, verbose=True
        )

        assert sizes_normal == sizes_verbose


def test_disk_usage_vs_apparent_size():
    """Test that sizes reflect actual disk usage (st_blocks), not apparent size"""
    with tempfile.TemporaryDirectory() as tmpdir:
        testfile = Path(tmpdir) / "testfile.bin"
        testfile.write_bytes(b"x" * 4096)

        sizes = calculate_directory_sizes(tmpdir, use_inodes=False)

        # Compare with what os.stat reports for actual blocks
        stat = os.stat(testfile)
        expected_kb = (stat.st_blocks * 512 + 1023) // 1024  # div_ceil

        assert sizes["testfile.bin"] == expected_kb, (
            f"Reported {sizes['testfile.bin']} KB, "
            f"expected {expected_kb} KB from st_blocks"
        )


def test_live():
    """Test that live parameter is accepted and results are unchanged"""
    with tempfile.TemporaryDirectory() as tmpdir:
        (Path(tmpdir) / "file1.txt").write_text("Hello" * 100)
        subdir = Path(tmpdir) / "subdir"
        subdir.mkdir()
        (subdir / "file2.txt").write_text("World" * 200)

        sizes_normal = calculate_directory_sizes(tmpdir, use_inodes=False)
        sizes_live = calculate_directory_sizes(tmpdir, use_inodes=False, live=True)

        assert sizes_normal == sizes_live


def test_ctrlc_exits_quickly():
    """Test that Ctrl+C (SIGINT) causes dustr to exit promptly"""
    # Run dustr on a large directory (root filesystem) so it takes a while
    proc = subprocess.Popen(
        [sys.executable, "-m", "dustr", "/"],
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
    )
    # Let it start working
    time.sleep(0.5)
    assert proc.poll() is None, "Process exited too quickly, nothing to interrupt"

    # Send SIGINT (same as Ctrl+C)
    proc.send_signal(signal.SIGINT)
    t0 = time.monotonic()

    try:
        proc.wait(timeout=10)
    except subprocess.TimeoutExpired:
        proc.kill()
        proc.wait()
        raise AssertionError("dustr did not exit within 10 seconds after SIGINT")

    elapsed = time.monotonic() - t0
    assert elapsed < 5, f"dustr took {elapsed:.1f}s to exit after SIGINT (expected < 5s)"


if __name__ == "__main__":
    test_calculate_directory_sizes()
    test_calculate_directory_sizes_inodes()
    test_get_file_type_indicator()
    test_nonexistent_directory()
    test_permission_denied()
    test_cross_mounts()
    test_verbose()
    test_disk_usage_vs_apparent_size()
    test_live()
    test_ctrlc_exits_quickly()
    print("All tests passed!")
