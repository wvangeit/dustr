window.BENCHMARK_DATA = {
  "lastUpdate": 1777021815990,
  "repoUrl": "https://github.com/wvangeit/dustr",
  "entries": {
    "Benchmark": [
      {
        "commit": {
          "author": {
            "email": "werner.vangeit@gmail.com",
            "name": "Werner Van Geit",
            "username": "wvangeit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "dff1dd7adadbd124e37d0c89a977d7fd44ba9293",
          "message": "Make ctrl-c more snappy (#22)\n\n* Make ctrl-c more snappy\n\n* Add benchmark for tests\n\n* Add benchmark data\n\n* Add benchmark to github actions\n\n* Address comments\n\n* Fix ci",
          "timestamp": "2026-04-24T10:51:07+02:00",
          "tree_id": "eb38d5d88e83a0a712b2cfbb1ecb9ac72514db7d",
          "url": "https://github.com/wvangeit/dustr/commit/dff1dd7adadbd124e37d0c89a977d7fd44ba9293"
        },
        "date": 1777020720293,
        "tool": "pytest",
        "benches": [
          {
            "name": "test_dustr.py::test_bench_sizes_small",
            "value": 759.0113037523398,
            "unit": "iter/sec",
            "range": "stddev: 0.0001924411601404288",
            "extra": "mean: 1.3175034351349966 msec\nrounds: 370"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_medium",
            "value": 106.26159734858646,
            "unit": "iter/sec",
            "range": "stddev: 0.000715100593274761",
            "extra": "mean: 9.410737509615483 msec\nrounds: 104"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_large",
            "value": 36.442105102338,
            "unit": "iter/sec",
            "range": "stddev: 0.0010969534591957605",
            "extra": "mean: 27.44078579411823 msec\nrounds: 34"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_small",
            "value": 816.1206157545199,
            "unit": "iter/sec",
            "range": "stddev: 0.00017258760135136888",
            "extra": "mean: 1.2253090789472092 msec\nrounds: 760"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_medium",
            "value": 132.71906295619834,
            "unit": "iter/sec",
            "range": "stddev: 0.0004860883188292617",
            "extra": "mean: 7.534712630769801 msec\nrounds: 130"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_large",
            "value": 49.92738582713118,
            "unit": "iter/sec",
            "range": "stddev: 0.0008402046294527861",
            "extra": "mean: 20.02908791304245 msec\nrounds: 46"
          },
          {
            "name": "test_dustr.py::test_bench_deep_tree",
            "value": 14.784326400005622,
            "unit": "iter/sec",
            "range": "stddev: 0.0023377620987256376",
            "extra": "mean: 67.63919930769518 msec\nrounds: 13"
          }
        ]
      },
      {
        "commit": {
          "author": {
            "email": "werner.vangeit@gmail.com",
            "name": "Werner Van Geit",
            "username": "wvangeit"
          },
          "committer": {
            "email": "noreply@github.com",
            "name": "GitHub",
            "username": "web-flow"
          },
          "distinct": true,
          "id": "5dbe7281bffe341a5b4ff8863f925272c1299799",
          "message": "Add benchmark sticker (#23)",
          "timestamp": "2026-04-24T11:09:22+02:00",
          "tree_id": "6ae32256230de745014a118396fde1447edb5e87",
          "url": "https://github.com/wvangeit/dustr/commit/5dbe7281bffe341a5b4ff8863f925272c1299799"
        },
        "date": 1777021815125,
        "tool": "pytest",
        "benches": [
          {
            "name": "test_dustr.py::test_bench_sizes_small",
            "value": 721.249954589107,
            "unit": "iter/sec",
            "range": "stddev: 0.00021654749215165373",
            "extra": "mean: 1.3864818897211517 msec\nrounds: 399"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_medium",
            "value": 108.71642055532706,
            "unit": "iter/sec",
            "range": "stddev: 0.0006524866911159584",
            "extra": "mean: 9.198242500001076 msec\nrounds: 104"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_large",
            "value": 37.52016490443284,
            "unit": "iter/sec",
            "range": "stddev: 0.0013272354498469138",
            "extra": "mean: 26.652334885709802 msec\nrounds: 35"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_small",
            "value": 777.2121816020284,
            "unit": "iter/sec",
            "range": "stddev: 0.00024042468303687124",
            "extra": "mean: 1.2866499312179465 msec\nrounds: 945"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_medium",
            "value": 130.98220307896838,
            "unit": "iter/sec",
            "range": "stddev: 0.00043778575237250685",
            "extra": "mean: 7.634624983343012 msec\nrounds: 120"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_large",
            "value": 48.81825910247878,
            "unit": "iter/sec",
            "range": "stddev: 0.0010662075902568447",
            "extra": "mean: 20.484138893621964 msec\nrounds: 47"
          },
          {
            "name": "test_dustr.py::test_bench_deep_tree",
            "value": 16.171101988480807,
            "unit": "iter/sec",
            "range": "stddev: 0.001765014166970202",
            "extra": "mean: 61.83870466665363 msec\nrounds: 15"
          }
        ]
      }
    ]
  }
}