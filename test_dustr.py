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
    with tempfile.TemporaryDirectory() as tmpdir:
        # Build a deterministic directory tree large enough that dustr is
        # still running when we interrupt it.
        root = Path(tmpdir)
        for i in range(200):
            subdir = root / f"dir_{i}"
            subdir.mkdir()
            for j in range(200):
                (subdir / f"file_{j}.txt").write_text("x" * 4096)

        proc = subprocess.Popen(
            [sys.executable, "-m", "dustr", tmpdir],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.DEVNULL,
        )
        # Let it start working
        time.sleep(0.5)
        if proc.poll() is not None:
            print("Skipped: dustr finished before SIGINT could be sent")
            return

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
        assert (
            elapsed < 5
        ), f"dustr took {elapsed:.1f}s to exit after SIGINT (expected < 5s)"


# ---------------------------------------------------------------------------
# Benchmarks (run with: pytest test_dustr.py -k bench --benchmark-only)
# ---------------------------------------------------------------------------


def _make_tree(root, dirs, files_per_dir, file_size=1024):
    """Create a directory tree with the given shape."""
    for i in range(dirs):
        d = Path(root) / f"dir_{i:04d}"
        d.mkdir()
        for j in range(files_per_dir):
            (d / f"file_{j:04d}.bin").write_bytes(b"x" * file_size)


def test_bench_sizes_small(benchmark, tmp_path):
    """Benchmark size calculation: 10 dirs x 10 files = 100 files"""
    _make_tree(tmp_path, dirs=10, files_per_dir=10)
    benchmark(calculate_directory_sizes, str(tmp_path), False)


def test_bench_sizes_medium(benchmark, tmp_path):
    """Benchmark size calculation: 50 dirs x 50 files = 2500 files"""
    _make_tree(tmp_path, dirs=50, files_per_dir=50)
    benchmark(calculate_directory_sizes, str(tmp_path), False)


def test_bench_sizes_large(benchmark, tmp_path):
    """Benchmark size calculation: 100 dirs x 100 files = 10000 files"""
    _make_tree(tmp_path, dirs=100, files_per_dir=100)
    benchmark(calculate_directory_sizes, str(tmp_path), False)


def test_bench_inodes_small(benchmark, tmp_path):
    """Benchmark inode counting: 10 dirs x 10 files = 100 files"""
    _make_tree(tmp_path, dirs=10, files_per_dir=10)
    benchmark(calculate_directory_sizes, str(tmp_path), True)


def test_bench_inodes_medium(benchmark, tmp_path):
    """Benchmark inode counting: 50 dirs x 50 files = 2500 files"""
    _make_tree(tmp_path, dirs=50, files_per_dir=50)
    benchmark(calculate_directory_sizes, str(tmp_path), True)


def test_bench_inodes_large(benchmark, tmp_path):
    """Benchmark inode counting: 100 dirs x 100 files = 10000 files"""
    _make_tree(tmp_path, dirs=100, files_per_dir=100)
    benchmark(calculate_directory_sizes, str(tmp_path), True)


def test_bench_deep_tree(benchmark, tmp_path):
    """Benchmark on a deep directory tree: 5 levels, 5 dirs each, 5 files each"""

    def _make_deep(parent, depth):
        if depth == 0:
            return
        for i in range(5):
            d = Path(parent) / f"d{depth}_{i}"
            d.mkdir()
            for j in range(5):
                (d / f"f_{j}.bin").write_bytes(b"x" * 512)
            _make_deep(d, depth - 1)

    _make_deep(tmp_path, depth=5)
    benchmark(calculate_directory_sizes, str(tmp_path), False)


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
