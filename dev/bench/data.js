window.BENCHMARK_DATA = {
  "lastUpdate": 1780671555652,
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
          "id": "e8c3b1cd2a2ea4a12acdea4e58ab843ee1a6959c",
          "message": "Updte action versions (#24)",
          "timestamp": "2026-04-24T11:20:32+02:00",
          "tree_id": "30f0310891fc89aa07751939981ce48cc0c0feaa",
          "url": "https://github.com/wvangeit/dustr/commit/e8c3b1cd2a2ea4a12acdea4e58ab843ee1a6959c"
        },
        "date": 1777022481824,
        "tool": "pytest",
        "benches": [
          {
            "name": "test_dustr.py::test_bench_sizes_small",
            "value": 738.107810901832,
            "unit": "iter/sec",
            "range": "stddev: 0.00018422822610297622",
            "extra": "mean: 1.3548156315785145 msec\nrounds: 361"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_medium",
            "value": 104.05218712692202,
            "unit": "iter/sec",
            "range": "stddev: 0.000813017495535833",
            "extra": "mean: 9.610562042104968 msec\nrounds: 95"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_large",
            "value": 36.773418010786635,
            "unit": "iter/sec",
            "range": "stddev: 0.0016921967195820957",
            "extra": "mean: 27.193555945946418 msec\nrounds: 37"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_small",
            "value": 783.9398298631512,
            "unit": "iter/sec",
            "range": "stddev: 0.0001881862766947725",
            "extra": "mean: 1.2756081039721703 msec\nrounds: 856"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_medium",
            "value": 130.00807776495756,
            "unit": "iter/sec",
            "range": "stddev: 0.0004837445961125031",
            "extra": "mean: 7.6918297477477235 msec\nrounds: 111"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_large",
            "value": 48.877974684421964,
            "unit": "iter/sec",
            "range": "stddev: 0.0009068650832793036",
            "extra": "mean: 20.459112851063217 msec\nrounds: 47"
          },
          {
            "name": "test_dustr.py::test_bench_deep_tree",
            "value": 16.103695187816097,
            "unit": "iter/sec",
            "range": "stddev: 0.0016236914897844756",
            "extra": "mean: 62.09754893750041 msec\nrounds: 16"
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
          "id": "f65ea607c677fa4cdd01912e8230a04660ac08a3",
          "message": "Rust bin (#25)\n\n* Make ctrl-c more snappy\n\n* Add benchmark for tests\n\n* Add benchmark data\n\n* Add benchmark to github actions\n\n* Address comments\n\n* Fix ci\n\n* Create separate rust bin\n\n* Update readme\n\n* Formatting issues etc\n\n* address comments",
          "timestamp": "2026-04-24T15:09:58+02:00",
          "tree_id": "7aa2b9c3b4fd2fabdba5738bc60bd659481873e0",
          "url": "https://github.com/wvangeit/dustr/commit/f65ea607c677fa4cdd01912e8230a04660ac08a3"
        },
        "date": 1777036262439,
        "tool": "pytest",
        "benches": [
          {
            "name": "test_dustr.py::test_bench_sizes_small",
            "value": 746.0417201252944,
            "unit": "iter/sec",
            "range": "stddev: 0.00018664154455648498",
            "extra": "mean: 1.3404076112955914 msec\nrounds: 301"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_medium",
            "value": 107.60576464307154,
            "unit": "iter/sec",
            "range": "stddev: 0.0005603237179058707",
            "extra": "mean: 9.293182417475506 msec\nrounds: 103"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_large",
            "value": 36.31153773880274,
            "unit": "iter/sec",
            "range": "stddev: 0.0009971694717757736",
            "extra": "mean: 27.539456114286054 msec\nrounds: 35"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_small",
            "value": 802.3874991273401,
            "unit": "iter/sec",
            "range": "stddev: 0.00018676394446887995",
            "extra": "mean: 1.2462806325965687 msec\nrounds: 724"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_medium",
            "value": 131.5280004116763,
            "unit": "iter/sec",
            "range": "stddev: 0.0004772926669565622",
            "extra": "mean: 7.602943836065692 msec\nrounds: 122"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_large",
            "value": 47.61276339556456,
            "unit": "iter/sec",
            "range": "stddev: 0.0010912282401556287",
            "extra": "mean: 21.002771708334755 msec\nrounds: 48"
          },
          {
            "name": "test_dustr.py::test_bench_deep_tree",
            "value": 15.742215255738888,
            "unit": "iter/sec",
            "range": "stddev: 0.0019675969595544587",
            "extra": "mean: 63.52346120000145 msec\nrounds: 15"
          },
          {
            "name": "test_dustr.py::test_bench_cli_sizes_small",
            "value": 213.55137486789636,
            "unit": "iter/sec",
            "range": "stddev: 0.0002259361543526411",
            "extra": "mean: 4.682713939999701 msec\nrounds: 50"
          },
          {
            "name": "test_dustr.py::test_bench_cli_sizes_medium",
            "value": 54.55753368261041,
            "unit": "iter/sec",
            "range": "stddev: 0.0009350971100239588",
            "extra": "mean: 18.329274299998985 msec\nrounds: 50"
          },
          {
            "name": "test_dustr.py::test_bench_cli_sizes_large",
            "value": 22.67892380450967,
            "unit": "iter/sec",
            "range": "stddev: 0.0021327901206600777",
            "extra": "mean: 44.09380306666719 msec\nrounds: 30"
          },
          {
            "name": "test_dustr.py::test_bench_cli_deep_tree",
            "value": 14.47543012072357,
            "unit": "iter/sec",
            "range": "stddev: 0.002048858634995444",
            "extra": "mean: 69.08257589999778 msec\nrounds: 10"
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
          "id": "6d0077f9ec911a5f4b62e9335c82f416df1801ab",
          "message": "Small improvements (#26)\n\n* Small improvements",
          "timestamp": "2026-06-05T16:58:13+02:00",
          "tree_id": "91b67c05fc3e17e21f630a0db6da95109efd818b",
          "url": "https://github.com/wvangeit/dustr/commit/6d0077f9ec911a5f4b62e9335c82f416df1801ab"
        },
        "date": 1780671554874,
        "tool": "pytest",
        "benches": [
          {
            "name": "test_dustr.py::test_bench_sizes_small",
            "value": 3866.861523675784,
            "unit": "iter/sec",
            "range": "stddev: 0.000013713529728153463",
            "extra": "mean: 258.60765736690104 usec\nrounds: 1147"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_medium",
            "value": 359.3392053811233,
            "unit": "iter/sec",
            "range": "stddev: 0.00011750025090602568",
            "extra": "mean: 2.7828858778139094 msec\nrounds: 311"
          },
          {
            "name": "test_dustr.py::test_bench_sizes_large",
            "value": 97.18305247766338,
            "unit": "iter/sec",
            "range": "stddev: 0.00025843482732225317",
            "extra": "mean: 10.289859955055855 msec\nrounds: 89"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_small",
            "value": 3952.3340395398072,
            "unit": "iter/sec",
            "range": "stddev: 0.00001650414574990021",
            "extra": "mean: 253.01505135847165 usec\nrounds: 3018"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_medium",
            "value": 357.03907829161,
            "unit": "iter/sec",
            "range": "stddev: 0.00013791895294163438",
            "extra": "mean: 2.800813862686635 msec\nrounds: 335"
          },
          {
            "name": "test_dustr.py::test_bench_inodes_large",
            "value": 92.98197949268959,
            "unit": "iter/sec",
            "range": "stddev: 0.0002998741804535517",
            "extra": "mean: 10.754772112359921 msec\nrounds: 89"
          },
          {
            "name": "test_dustr.py::test_bench_deep_tree",
            "value": 19.07021986139137,
            "unit": "iter/sec",
            "range": "stddev: 0.00023885273899628353",
            "extra": "mean: 52.43778033333276 msec\nrounds: 18"
          },
          {
            "name": "test_dustr.py::test_bench_cli_sizes_small",
            "value": 613.5486889231876,
            "unit": "iter/sec",
            "range": "stddev: 0.00005586414578864442",
            "extra": "mean: 1.6298624999998879 msec\nrounds: 50"
          },
          {
            "name": "test_dustr.py::test_bench_cli_sizes_medium",
            "value": 222.39420905410358,
            "unit": "iter/sec",
            "range": "stddev: 0.0002599542943535538",
            "extra": "mean: 4.496519959999148 msec\nrounds: 50"
          },
          {
            "name": "test_dustr.py::test_bench_cli_sizes_large",
            "value": 76.50246772862721,
            "unit": "iter/sec",
            "range": "stddev: 0.0009253663959909121",
            "extra": "mean: 13.071473766666486 msec\nrounds: 30"
          },
          {
            "name": "test_dustr.py::test_bench_cli_deep_tree",
            "value": 18.830970986140443,
            "unit": "iter/sec",
            "range": "stddev: 0.0006613640257646602",
            "extra": "mean: 53.10400619999882 msec\nrounds: 10"
          }
        ]
      }
    ]
  }
}