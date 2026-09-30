# Benchmark results

The baseline of 2026-09-30 on an Intel(R) Core(TM) Ultra X9 388H
(16 cores, 62 GB), release build, from
`bench/baseline.sh`, which writes this file, the CSV files beside it
(`bench/results/2026-09-30/`, one row per run) and a copy of this file as
`bench/RESULTS.md`, the latest baseline's. It started (once more for
every resumption), with the commit it measured:

- 2026-09-30 21:23, commit b53cb17c6831 (Keep every baseline in a directory of its own and run it unattended in the night slot), load average 0.57

Its last part took 9 h 21 min, and the scheduled jobs that ran
meanwhile were: none. The package throttled
473691 times for 11283 s in that part
(platform profile performance, governor powersave, energy preference balance_performance, turbo on).
The journal of `linlog-baseline` says every ten minutes what else used
a CPU. The sequential runs went in four
streams at once, each pinned to a performance core of its own; the
parallel runs had the machine to themselves. A time is the median of up to
three runs; `✓` is proved, `✗` refuted, `?` unknown (`bound`: the copy
bound, `wide`: a context too wide to split, `depth`: the recursion
limit), `>` a time limit reached, `MISMATCH` a verdict against the
problem's known one (for LLTP, the one its header claims). `linlog-bench
summary bench/results/2026-09-30/*.csv` prints the tables again.

## Solved within the time limit

Sequential problems that waited for a CPU for over 1 % of their time (another process slowed them down; `†` in the tables below): 5.

| family | configuration | engines | problems | solved | proved | refuted | timeout | bound | other | mismatch | median solved | total solved |
|---|---|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-yes | engines: classical focus j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 169.0 ms | 12.32 s |
| partition-yes | engines: classical net j1 | net | 3 | 1 | 1 | 0 | 2 | 0 | 0 | 0 | 2.53 s | 2.53 s |
| partition-no | engines: classical focus j1 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 111.1 ms | 115.4 ms |
| partition-no | engines: classical net j1 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 3.45 s | 3.45 s |
| 3-partition-mll-yes | engines: classical focus j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 33 µs | 132 µs |
| 3-partition-mll-yes | engines: classical net j1 | net | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 114.7 ms | 5.67 s |
| 3-partition-mll-no | engines: classical focus j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 17 µs | 66 µs |
| 3-partition-mll-no | engines: classical net j1 | net | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 56.31 s | 58.10 s |
| wide-m1 | engines: classical focus j1 | focus | 6 | 4 | 4 | 0 | 0 | 0 | 2 | 0 | 177.5 ms | 45.71 s |
| wide-m1 | engines: classical net j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 220 µs | 648.3 ms |
| wide-m2 | engines: classical focus j1 | focus | 6 | 4 | 4 | 0 | 0 | 0 | 2 | 0 | 171.1 ms | 43.67 s |
| wide-m2 | engines: classical net j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 217 µs | 639.1 ms |
| wide-m3 | engines: classical focus j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 168.6 ms | 11.02 s |
| wide-m3 | engines: classical net j1 | net | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 182 µs | 470 µs |
| wide-m4 | engines: classical focus j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 171.8 ms | 2.92 s |
| wide-m4 | engines: classical net j1 | net | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 223 µs | 589 µs |
| additive | engines: classical focus j1 | focus | 4 | 3 | 3 | 0 | 0 | 0 | 1 | 0 | 354.7 ms | 6.55 s |
| partition-table | engines: classical focus j1 | focus | 13 | 12 | 6 | 6 | 1 | 0 | 0 | 0 | 5.8 ms | 81.70 s |
| partition-table | engines: classical net j1 | net | 13 | 9 | 5 | 4 | 4 | 0 | 0 | 0 | 51.5 ms | 16.87 s |
| cancellation | engines: classical focus j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 50.68 s | 50.68 s |
| 3-partition-yes | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 2.0 ms | 32.8 ms |
| 3-partition-no | families: classical auto j1 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 49.77 s | 49.77 s |
| 3-partition-mll-yes | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 31 µs | 121 µs |
| 3-partition-mll-no | families: classical auto j1 | focus | 4 | 4 | 0 | 4 | 0 | 0 | 0 | 0 | 16 µs | 62 µs |
| partition-yes | families: classical auto j1 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 186.5 ms | 12.93 s |
| partition-no | families: classical auto j1 | focus | 3 | 2 | 0 | 2 | 1 | 0 | 0 | 0 | 113.8 ms | 118.2 ms |
| qbf | families: classical auto j1 | focus | 20 | 15 | 5 | 10 | 5 | 0 | 0 | 0 | 121.0 ms | 318.35 s |
| wide-m1 | families: classical auto j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 198 µs | 619.5 ms |
| wide-m2 | families: classical auto j1 | net | 6 | 6 | 6 | 0 | 0 | 0 | 0 | 0 | 201 µs | 611.7 ms |
| wide-m3 | families: classical auto j1 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 166.4 ms | 10.74 s |
| wide-m4 | families: classical auto j1 | focus | 5 | 4 | 4 | 0 | 1 | 0 | 0 | 0 | 2.72 s | 46.28 s |
| mix | families: mix auto j1 | focus | 6 | 5 | 0 | 5 | 1 | 0 | 0 | 0 | 1.91 s | 235.74 s |
| counter | families: classical auto j1 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 84 µs | 124.1 ms |
| counter-over | families: classical auto j1 | focus | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| growing | families: classical auto j1 | focus | 4 | 0 | 0 | 0 | 0 | 3 | 1 | 0 | – | – |
| chain | families: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 269.7 ms | 3.11 s |
| additive | families: classical auto j1 | additive | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 69.7 ms | 611.5 ms |
| counter | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 48 µs | 15.5 ms |
| counter-over | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| chain | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 275.9 ms | 3.19 s |
| 3-partition-yes | intuitionistic: intuitionistic auto j1 | two-sided | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 1.1 ms | 18.0 ms |
| 3-partition-no | intuitionistic: intuitionistic auto j1 | two-sided | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 25.38 s | 25.38 s |
| ILL/ILLTP-SYJ-01 | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 0 | 0 | 0 | 3 | 13 | 15 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 0 | 0 | 0 | 1 | 15 | 15 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-all-cores: intuitionistic auto j16 | two-sided | 31 | 1 | 1 | 0 | 30 | 0 | 0 | 0 | 333.3 ms | 333.3 ms |
| ILL/petri-nets/MCC | lltp-all-cores: intuitionistic auto j16 | two-sided | 1009 | 31 | 31 | 0 | 956 | 20 | 2 | 0 | 1.59 s | 62.08 s |
| ILL/petri-nets/MCC | lltp-classical: classical auto j1 | focus | 3137 | 204 | 204 | 0 | 1091 | 68 | 1774 | 0 | 603 µs | 26.76 s |
| ILL/misc | lltp-classical: classical auto j1 | focus | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 17 µs | 77 µs |
| ILL/Non-theorems | lltp-classical: classical auto j1 | focus | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/KLE-cbv | lltp-classical: classical auto j1 | focus | 88 | 67 | 64 | 3 | 0 | 21 | 0 | 0 | 20 µs | 1.5 ms |
| ILL/KLE-cbn | lltp-classical: classical auto j1 | focus | 88 | 68 | 66 | 2 | 0 | 20 | 0 | 0 | 16 µs | 1.4 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-classical: classical auto j1 | focus, net | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 6 µs | 146 µs |
| ILL/KLE-IMP-CONJ | lltp-classical: classical auto j1 | focus, net | 222 | 162 | 162 | 0 | 0 | 60 | 0 | 0 | 25 µs | 4.4 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-classical: classical auto j1 | focus, net | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 22 µs | 582 µs |
| ILL/KLE-01 | lltp-classical: classical auto j1 | focus | 88 | 50 | 48 | 2 | 0 | 38 | 0 | 0 | 36 µs | 1.8 ms |
| ILL/ILLTP-SYN-cbv | lltp-classical: classical auto j1 | focus | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 0 | 19 µs | 362 µs |
| ILL/ILLTP-SYN-cbn | lltp-classical: classical auto j1 | focus, net | 19 | 16 | 6 | 10 | 0 | 3 | 0 | 0 | 15 µs | 346 µs |
| ILL/ILLTP-SYN-01 | lltp-classical: classical auto j1 | focus | 19 | 11 | 4 | 7 | 0 | 8 | 0 | 0 | 16 µs | 199 µs |
| ILL/ILLTP-SYJ-cbv | lltp-classical: classical auto j1 | focus | 252 | 39 | 18 | 21 | 30 | 173 | 10 | 0 | 27 µs | 524.2 ms |
| ILL/ILLTP-SYJ-cbn | lltp-classical: classical auto j1 | focus | 252 | 34 | 13 | 21 | 10 | 186 | 22 | 0 | 21 µs | 757 µs |
| ILL/ILLTP-SYJ-01 | lltp-classical: classical auto j1 | focus | 252 | 11 | 10 | 1 | 9 | 209 | 23 | 0 | 17 µs | 257 µs |
| ILL/ILLTP-LCL-cbv | lltp-classical: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-classical: classical auto j1 | focus | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 27 µs | 27 µs |
| ILL/ILLTP-LCL-01 | lltp-classical: classical auto j1 | focus | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| CLL/misc | lltp-classical: classical auto j1 | focus, net | 14 | 8 | 8 | 0 | 1 | 5 | 0 | 0 | 14 µs | 115 µs |
| CLL/Non-theorems | lltp-classical: classical auto j1 | focus | 3 | 3 | 0 | 3 | 0 | 0 | 0 | 0 | 14 µs | 86 µs |
| ILL/ILLTP-LCL-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 44 µs | 44 µs |
| ILL/ILLTP-LCL-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 30 µs | 58 µs |
| ILL/ILLTP-LCL-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 44 µs | 44 µs |
| ILL/ILLTP-SYJ-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 37 | 35 | 2 | 107 | 65 | 0 | 1 | 2.3 ms | 15.18 s |
| ILL/ILLTP-SYJ-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 60 | 37 | 23 | 100 | 49 | 0 | 1 | 66 µs | 8.65 s |
| ILL/ILLTP-SYJ-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 209 | 83 | 56 | 27 | 100 | 26 | 0 | 4 | 731 µs | 22.52 s |
| ILL/ILLTP-SYN-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 5 | 3 | 2 | 0 | 3 | 0 | 0 | 109 µs | 712 µs |
| ILL/ILLTP-SYN-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 5 | 3 | 2 | 0 | 3 | 0 | 0 | 39 µs | 180 µs |
| ILL/ILLTP-SYN-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 8 | 5 | 3 | 2 | 0 | 3 | 0 | 0 | 28 µs | 161 µs |
| ILL/KLE-01 | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 29 | 28 | 1 | 0 | 9 | 0 | 0 | 123 µs | 31.6 ms |
| ILL/KLE-IMP-CONJ | lltp-copies-10: intuitionistic auto j1 | two-sided | 60 | 60 | 60 | 0 | 0 | 0 | 0 | 0 | 81 µs | 6.9 ms |
| ILL/KLE-cbn | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 32 | 28 | 4 | 0 | 6 | 0 | 3 | 33 µs | 3.2 ms |
| ILL/KLE-cbv | lltp-copies-10: intuitionistic auto j1 | two-sided | 38 | 32 | 28 | 4 | 0 | 6 | 0 | 3 | 42 µs | 6.6 ms |
| ILL/Non-theorems | lltp-copies-10: intuitionistic auto j1 | two-sided | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-copies-10: intuitionistic auto j1 | two-sided | 171 | 59 | 59 | 0 | 108 | 4 | 0 | 0 | 320.6 ms | 76.17 s |
| ILL/ILLTP-LCL-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-LCL-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 1 | 0 | 1 | 0 | 1 | 0 | 0 | 30 µs | 30 µs |
| ILL/ILLTP-LCL-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 2 | 0 | 0 | 0 | 0 | 2 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 11 | 10 | 1 | 9 | 209 | 23 | 0 | 18 µs | 285 µs |
| ILL/ILLTP-SYJ-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 34 | 13 | 21 | 10 | 186 | 22 | 1 | 24 µs | 844 µs |
| ILL/ILLTP-SYJ-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 252 | 39 | 18 | 21 | 30 | 173 | 10 | 0 | 31 µs | 566.2 ms |
| ILL/ILLTP-SYN-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 19 | 11 | 4 | 7 | 0 | 8 | 0 | 2 | 13 µs | 162 µs |
| ILL/ILLTP-SYN-cbn | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 19 | 16 | 6 | 10 | 0 | 3 | 0 | 3 | 12 µs | 278 µs |
| ILL/ILLTP-SYN-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 19 | 16 | 7 | 9 | 0 | 3 | 0 | 2 | 16 µs | 301 µs |
| ILL/KLE-01 | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 50 | 48 | 2 | 0 | 38 | 0 | 2 | 29 µs | 1.5 ms |
| ILL/KLE-IMP-CONJ/ALT | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 27 | 27 | 27 | 0 | 0 | 0 | 0 | 0 | 18 µs | 461 µs |
| ILL/KLE-IMP-CONJ | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 222 | 162 | 162 | 0 | 0 | 60 | 0 | 0 | 22 µs | 3.9 ms |
| ILL/KLE-IMP-CONJ/NON-THEOREMS | lltp-intuitionistic: intuitionistic auto j1 | net, two-sided | 22 | 22 | 0 | 22 | 0 | 0 | 0 | 0 | 6 µs | 166 µs |
| ILL/KLE-cbn | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 68 | 66 | 2 | 0 | 20 | 0 | 2 | 18 µs | 1.6 ms |
| ILL/KLE-cbv | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 88 | 67 | 64 | 3 | 0 | 21 | 0 | 2 | 23 µs | 1.7 ms |
| ILL/Non-theorems | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 1 | 0 | 0 | 0 | 0 | 1 | 0 | 0 | – | – |
| ILL/misc | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 3 | 3 | 3 | 0 | 0 | 0 | 0 | 0 | 18 µs | 70 µs |
| ILL/petri-nets/MCC | lltp-intuitionistic: intuitionistic auto j1 | two-sided | 3137 | 210 | 210 | 0 | 982 | 171 | 1774 | 0 | 543 µs | 26.40 s |
| ILL/ILLTP-SYJ-01 | lltp-portfolio: intuitionistic auto j16 portfolio | two-sided | 31 | 0 | 0 | 0 | 3 | 13 | 15 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-portfolio: intuitionistic auto j16 portfolio | two-sided | 31 | 0 | 0 | 0 | 1 | 15 | 15 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-portfolio: intuitionistic auto j16 portfolio | two-sided | 31 | 1 | 1 | 0 | 30 | 0 | 0 | 0 | 341.8 ms | 341.8 ms |
| ILL/petri-nets/MCC | lltp-portfolio: intuitionistic auto j16 portfolio | two-sided | 1009 | 33 | 33 | 0 | 954 | 20 | 2 | 0 | 1.64 s | 71.04 s |
| ILL/ILLTP-SYJ-01 | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 22 | 0 | 0 | 0 | – | – |
| ILL/ILLTP-SYJ-cbn | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 20 | 0 | 2 | 0 | – | – |
| ILL/ILLTP-SYJ-cbv | lltp-recursion: intuitionistic auto j1 | two-sided | 22 | 0 | 0 | 0 | 22 | 0 | 0 | 0 | – | – |
| ILL/petri-nets/MCC | lltp-recursion: intuitionistic auto j1 | two-sided | 929 | 56 | 56 | 0 | 320 | 0 | 553 | 0 | 8.9 ms | 9.14 s |
| 3-partition-no | long-1: classical auto j1 | focus | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 302.42 s | 302.42 s |
| partition-no | long-1: classical auto j1 | focus | 1 | 1 | 0 | 1 | 0 | 0 | 0 | 0 | 811.48 s | 811.48 s |
| partition-yes | long-1: classical auto j1 | focus | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| counter | long-1: classical auto j1 | focus | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| mix | long-2: mix auto j1 | focus | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| wide-m3 | long-2: classical auto j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 697.42 s | 697.42 s |
| wide-m4 | long-2: classical auto j1 | focus | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 685.07 s | 685.07 s |
| qbf | long-2: classical auto j1 | focus | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| 3-partition-yes | parallel: classical auto j1 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 2.4 ms | 41.0 ms |
| 3-partition-yes | parallel: classical auto j2 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 2.4 ms | 33.4 ms |
| 3-partition-yes | parallel: classical auto j4 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 2.3 ms | 32.9 ms |
| 3-partition-yes | parallel: classical auto j8 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 2.5 ms | 33.5 ms |
| 3-partition-yes | parallel: classical auto j16 | focus | 4 | 4 | 4 | 0 | 0 | 0 | 0 | 0 | 2.8 ms | 35.2 ms |
| 3-partition-no | parallel: classical auto j1 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 45.09 s | 45.09 s |
| 3-partition-no | parallel: classical auto j2 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 23.49 s | 23.49 s |
| 3-partition-no | parallel: classical auto j4 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 77.70 s | 90.39 s |
| 3-partition-no | parallel: classical auto j8 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 46.72 s | 54.16 s |
| 3-partition-no | parallel: classical auto j16 | focus | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 37.07 s | 42.87 s |
| partition-yes | parallel: classical auto j1 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 12.20 s | 12.39 s |
| partition-yes | parallel: classical auto j2 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 11.82 s | 12.02 s |
| partition-yes | parallel: classical auto j4 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 8.70 s | 8.90 s |
| partition-yes | parallel: classical auto j8 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 4.21 s | 4.24 s |
| partition-yes | parallel: classical auto j16 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 1.33 s | 1.35 s |
| partition-no | parallel: classical auto j1 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 127.3 ms | 127.3 ms |
| partition-no | parallel: classical auto j2 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 73.9 ms | 73.9 ms |
| partition-no | parallel: classical auto j4 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 45.9 ms | 45.9 ms |
| partition-no | parallel: classical auto j8 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 29.4 ms | 29.4 ms |
| partition-no | parallel: classical auto j16 | focus | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 26.9 ms | 26.9 ms |
| qbf | parallel: classical auto j1 | focus | 12 | 7 | 2 | 5 | 5 | 0 | 0 | 0 | 8.28 s | 286.63 s |
| qbf | parallel: classical auto j2 | focus | 12 | 7 | 2 | 5 | 5 | 0 | 0 | 0 | 6.37 s | 159.18 s |
| qbf | parallel: classical auto j4 | focus | 12 | 7 | 2 | 5 | 5 | 0 | 0 | 0 | 6.26 s | 159.83 s |
| qbf | parallel: classical auto j8 | focus | 12 | 7 | 2 | 5 | 5 | 0 | 0 | 0 | 6.32 s | 175.47 s |
| qbf | parallel: classical auto j16 | focus | 12 | 7 | 2 | 5 | 5 | 0 | 0 | 0 | 6.33 s | 159.80 s |
| mix | parallel: mix auto j1 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 17.95 s | 19.76 s |
| mix | parallel: mix auto j2 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 17.97 s | 19.83 s |
| mix | parallel: mix auto j4 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 16.72 s | 18.37 s |
| mix | parallel: mix auto j8 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 16.01 s | 17.68 s |
| mix | parallel: mix auto j16 | focus | 4 | 2 | 0 | 2 | 2 | 0 | 0 | 0 | 16.93 s | 18.82 s |
| counter | parallel: classical auto j1 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 110 µs | 129.8 ms |
| counter | parallel: classical auto j2 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 225 µs | 156.6 ms |
| counter | parallel: classical auto j4 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 258 µs | 127.6 ms |
| counter | parallel: classical auto j8 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 321 µs | 105.9 ms |
| counter | parallel: classical auto j16 | focus | 4 | 3 | 3 | 0 | 1 | 0 | 0 | 0 | 645 µs | 110.4 ms |
| counter-over | parallel: classical auto j1 | focus | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| counter-over | parallel: classical auto j2 | focus | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| counter-over | parallel: classical auto j4 | focus | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| counter-over | parallel: classical auto j8 | focus | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| counter-over | parallel: classical auto j16 | focus | 4 | 0 | 0 | 0 | 1 | 3 | 0 | 0 | – | – |
| wide-m3 | parallel: classical auto j1 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 11.12 s | 11.31 s |
| wide-m3 | parallel: classical auto j2 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 10.69 s | 10.88 s |
| wide-m3 | parallel: classical auto j4 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 8.34 s | 8.49 s |
| wide-m3 | parallel: classical auto j8 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 7.87 s | 8.01 s |
| wide-m3 | parallel: classical auto j16 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 8.24 s | 8.37 s |
| wide-m4 | parallel: classical auto j1 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 42.28 s | 45.25 s |
| wide-m4 | parallel: classical auto j2 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 41.95 s | 44.87 s |
| wide-m4 | parallel: classical auto j4 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 32.67 s | 34.87 s |
| wide-m4 | parallel: classical auto j8 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 32.75 s | 34.87 s |
| wide-m4 | parallel: classical auto j16 | focus | 3 | 2 | 2 | 0 | 1 | 0 | 0 | 0 | 35.49 s | 37.47 s |
| partition-table | parallel-net: classical net j2 | net | 13 | 9 | 5 | 4 | 4 | 0 | 0 | 0 | 31.6 ms | 8.83 s |
| partition-table | parallel-net: classical net j4 | net | 13 | 10 | 6 | 4 | 3 | 0 | 0 | 0 | 17.3 ms | 4.54 s |
| partition-table | parallel-net: classical net j8 | net | 13 | 10 | 6 | 4 | 3 | 0 | 0 | 0 | 149.5 ms | 3.38 s |
| partition-table | parallel-net: classical net j16 | net | 13 | 10 | 6 | 4 | 3 | 0 | 0 | 0 | 76.5 ms | 2.31 s |
| wide-m1 | period-16: classical net j1 period 16 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 459.5 ms | 466.1 ms |
| wide-m2 | period-16: classical net j1 period 16 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 442.5 ms | 449.1 ms |
| 3-partition-mll-no | period-16: classical net j1 period 16 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 22.04 s | 22.04 s |
| partition-yes | period-16: classical net j1 period 16 | net | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| partition-no | period-16: classical net j1 period 16 | net | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |
| wide-m1 | period-1: classical net j1 period 1 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 1.35 s | 1.37 s |
| wide-m2 | period-1: classical net j1 period 1 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 1.35 s | 1.37 s |
| 3-partition-mll-no | period-1: classical net j1 period 1 | net | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 56.27 s | 58.07 s |
| partition-yes | period-1: classical net j1 period 1 | net | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 2.76 s | 2.76 s |
| partition-no | period-1: classical net j1 period 1 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 3.49 s | 3.49 s |
| wide-m1 | period-2: classical net j1 period 2 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 856.4 ms | 868.5 ms |
| wide-m2 | period-2: classical net j1 period 2 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 840.4 ms | 852.7 ms |
| 3-partition-mll-no | period-2: classical net j1 period 2 | net | 2 | 2 | 0 | 2 | 0 | 0 | 0 | 0 | 51.40 s | 52.83 s |
| partition-yes | period-2: classical net j1 period 2 | net | 1 | 1 | 1 | 0 | 0 | 0 | 0 | 0 | 6.11 s | 6.11 s |
| partition-no | period-2: classical net j1 period 2 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 8.11 s | 8.11 s |
| wide-m1 | period-8: classical net j1 period 8 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 520.8 ms | 528.1 ms |
| wide-m2 | period-8: classical net j1 period 8 | net | 2 | 2 | 2 | 0 | 0 | 0 | 0 | 0 | 503.9 ms | 511.6 ms |
| 3-partition-mll-no | period-8: classical net j1 period 8 | net | 2 | 1 | 0 | 1 | 1 | 0 | 0 | 0 | 5.01 s | 5.01 s |
| partition-yes | period-8: classical net j1 period 8 | net | 1 | 0 | 0 | 0 | 1 | 0 | 0 | 0 | – | – |
| partition-no | period-8: classical net j1 period 8 | net | 2 | 0 | 0 | 0 | 2 | 0 | 0 | 0 | – | – |

## partition-yes

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-yes/4 | 2.3 ms ✓ | 2.53 s ✓ | 2.3 ms ✓ |  |  |  |  |  |  | > 60 s | 2.76 s ✓ | 6.11 s ✓ | > 60 s |
| partition-yes/5 | 169.0 ms ✓ | > 60 s | 186.5 ms ✓ |  | 191.6 ms ✓ | 192.0 ms ✓ | 195.6 ms ✓ | 30.5 ms ✓ | 17.5 ms ✓ |  |  |  |  |
| partition-yes/6 | 12.15 s ✓ | > 60 s | 12.74 s ✓ |  | 12.20 s ✓ | 11.82 s ✓ | 8.70 s ✓ | 4.21 s ✓ | 1.33 s ✓ |  |  |  |  |
| partition-yes/7 |  |  | > 300 s | > 1200 s | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |  |  |  |  |

## partition-no

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| partition-no/3 | 4.2 ms ✗ | 3.45 s ✗ | 4.4 ms ✗ |  |  |  |  |  |  | > 60 s | 3.49 s ✗ | 8.11 s ✗ | > 60 s |
| partition-no/4 | 111.1 ms ✗ | > 60 s | 113.8 ms ✗ |  | 127.3 ms ✗ | 73.9 ms ✗ | 45.9 ms ✗ | 29.4 ms ✗ | 26.9 ms ✗ | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-no/5 |  |  | > 300 s | 811.48 s ✗ | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |  |  |  |  |

## 3-partition-mll-yes

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 |
|---|--:|--:|--:|
| 3-partition-mll-yes/4 | 31 µs ✓ | 802 µs ✓ | 27 µs ✓ |
| 3-partition-mll-yes/6 | 31 µs ✓ | 114.7 ms ✓ | 28 µs ✓ |
| 3-partition-mll-yes/8 | 33 µs ✓ | 5.55 s ✓ | 31 µs ✓ |
| 3-partition-mll-yes/12 | 37 µs ✓ | > 60 s | 35 µs ✓ |

## 3-partition-mll-no

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-mll-no/4 | 15 µs ✗ | 1.80 s ✗ | 14 µs ✗ | 22.04 s ✗ | 1.80 s ✗ | 1.43 s ✗ | 5.01 s ✗ |
| 3-partition-mll-no/5 | 18 µs ✗ | 56.31 s ✗ | 15 µs ✗ | > 60 s | 56.27 s ✗ | 51.40 s ✗ | > 60 s |
| 3-partition-mll-no/6 | 17 µs ✗ | > 60 s | 17 µs ✗ |  |  |  |  |
| 3-partition-mll-no/8 | 16 µs ✗ | > 60 s | 16 µs ✗ |  |  |  |  |

## wide-m1

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| wide-m1/8 | 39 µs ✓ | 45 µs ✓ | 40 µs ✓ |  |  |  |  |
| wide-m1/16 | 765 µs ✓ | 111 µs ✓ | 110 µs ✓ |  |  |  |  |
| wide-m1/24 | 177.5 ms ✓ | 220 µs ✓ | 198 µs ✓ |  |  |  |  |
| wide-m1/32 | 45.53 s ✓ | 196 µs ✓ | 178 µs ✓ |  |  |  |  |
| wide-m1/256 | 971 µs ? wide | 9.3 ms ✓ | 8.8 ms ✓ | 6.5 ms ✓ | 19.2 ms ✓ | 12.2 ms ✓ | 7.3 ms ✓ |
| wide-m1/2048 | 50.3 ms ? wide | 638.4 ms ✓ | 610.2 ms ✓ | 459.5 ms ✓ | 1.35 s ✓ | 856.4 ms ✓ | 520.8 ms ✓ |

## wide-m2

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | period-16: classical net j1 period 16 | period-1: classical net j1 period 1 | period-2: classical net j1 period 2 | period-8: classical net j1 period 8 |
|---|--:|--:|--:|--:|--:|--:|--:|
| wide-m2/8 | 38 µs ✓ | 44 µs ✓ | 40 µs ✓ |  |  |  |  |
| wide-m2/16 | 714 µs ✓ | 109 µs ✓ | 103 µs ✓ |  |  |  |  |
| wide-m2/24 | 171.1 ms ✓ | 217 µs ✓ | 201 µs ✓ |  |  |  |  |
| wide-m2/32 | 43.50 s ✓ | 199 µs ✓ | 170 µs ✓ |  |  |  |  |
| wide-m2/256 | 566 µs ? wide | 9.3 ms ✓ | 8.8 ms ✓ | 6.6 ms ✓ | 19.3 ms ✓ | 12.4 ms ✓ | 7.6 ms ✓ |
| wide-m2/2048 | 26.9 ms ? wide | 629.2 ms ✓ † | 602.4 ms ✓ | 442.5 ms ✓ | 1.35 s ✓ | 840.4 ms ✓ | 503.9 ms ✓ |

## wide-m3

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| wide-m3/12 | 84 µs ✓ | 71 µs ✓ | 77 µs ✓ |  |  |  |  |  |  |
| wide-m3/24 | 168.6 ms ✓ | 217 µs ✓ | 166.4 ms ✓ |  | 189.7 ms ✓ | 191.7 ms ✓ | 149.5 ms ✓ | 138.8 ms ✓ | 131.0 ms ✓ |
| wide-m3/30 | 10.85 s ✓ | 182 µs ✓ | 10.57 s ✓ |  | 11.12 s ✓ | 10.69 s ✓ | 8.34 s ✓ | 7.87 s ✓ | 8.24 s ✓ |
| wide-m3/36 |  |  | > 300 s | 697.42 s ✓ | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |

## wide-m4

| problem | engines: classical focus j1 | engines: classical net j1 | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|--:|
| wide-m4/12 | 87 µs ✓ | 76 µs ✓ | 79 µs ✓ |  |  |  |  |  |  |
| wide-m4/24 | 171.8 ms ✓ | 223 µs ✓ | 170.2 ms ✓ |  |  |  |  |  |  |
| wide-m4/28 | 2.75 s ✓ | 290 µs ✓ | 2.72 s ✓ |  | 2.97 s ✓ | 2.92 s ✓ | 2.19 s ✓ | 2.12 s ✓ | 1.98 s ✓ |
| wide-m4/32 |  |  | 43.39 s ✓ |  | 42.28 s ✓ | 41.95 s ✓ | 32.67 s ✓ | 32.75 s ✓ | 35.49 s ✓ |
| wide-m4/36 |  |  | > 300 s | 685.07 s ✓ | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |

## additive

| problem | engines: classical focus j1 | families: classical auto j1 |
|---|--:|--:|
| additive/8 | 1.6 ms ✓ | 363 µs ✓ |
| additive/12 | 354.7 ms ✓ | 13.6 ms ✓ |
| additive/14 | 6.19 s ✓ | 69.7 ms ✓ |
| additive/16 | 12.21 s unknown | 527.9 ms ✓ |

## partition-table

| problem | engines: classical focus j1 | engines: classical net j1 | parallel-net: classical net j2 | parallel-net: classical net j4 | parallel-net: classical net j8 | parallel-net: classical net j16 |
|---|--:|--:|--:|--:|--:|--:|
| partition-table/1-1 | 38 µs ✓ | 36 µs ✓ | 203 µs ✓ | 239 µs ✓ | 384 µs ✓ | 482 µs ✓ |
| partition-table/1-3 | 131 µs ✗ | 394 µs ✗ | 557 µs ✗ | 1.3 ms ✗ | 1.4 ms ✗ | 1.6 ms ✗ |
| partition-table/2-1-1 | 165 µs ✓ | 657 µs ✓ | 930 µs ✓ | 4.1 ms ✓ | 4.3 ms ✓ | 4.4 ms ✓ |
| partition-table/1-1-4 | 2.3 ms ✗ | 39.9 ms ✗ | 25.8 ms ✗ | 13.5 ms ✗ | 8.4 ms ✗ | 5.2 ms ✗ |
| partition-table/2-2-1-1 | 2.2 ms ✓ | 51.5 ms ✓ | 31.6 ms ✓ | 17.3 ms ✓ | 9.3 ms ✓ | 7.7 ms ✓ |
| partition-table/1-2-5 | 5.8 ms ✗ | 3.66 s ✗ | 2.04 s ✗ | 1.02 s ✗ | 582.3 ms ✗ | 309.5 ms ✗ |
| partition-table/1-1-2-4 | 526 µs ✓ | 1.03 s ✓ | 554.8 ms ✓ | 281.9 ms ✓ | 149.5 ms ✓ | 76.5 ms ✓ |
| partition-table/1-1-1-5 | 61.2 ms ✗ | 7.50 s ✗ | 3.91 s ✗ | 2.02 s ✗ | 1.11 s ✗ | 611.3 ms ✗ |
| partition-table/2-3-2-1 | 6.1 ms ✓ | 4.58 s ✓ | 2.27 s ✓ | 1.19 s ✓ | 654.3 ms ✓ | 337.5 ms ✓ |
| partition-table/3-3-3-1 | 112.2 ms ✗ | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-table/1-2-3-4-5-5 | 23.15 s ✓ | > 60 s | > 60 s | 3.5 ms ✓ | > 60 s | > 60 s |
| partition-table/1-1-1-1-1-7 | 58.37 s ✗ | > 60 s | > 60 s | > 60 s | > 60 s | > 60 s |
| partition-table/2-2-2-2-2-2-9-1 | > 60 s | > 60 s | > 60 s | > 60 s | 861.2 ms ✓ | 955.4 ms ✓ |

## cancellation

| problem | engines: classical focus j1 |
|---|--:|
| cancellation/3-partition-4 | 50.68 s ✓ |

## 3-partition-yes

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-yes/4 | 180 µs ✓ | 116 µs ✓ | 217 µs ✓ | 489 µs ✓ | 425 µs ✓ | 433 µs ✓ | 706 µs ✓ |
| 3-partition-yes/6 | 555 µs ✓ | 320 µs ✓ | 674 µs ✓ | 920 µs ✓ | 911 µs ✓ | 1.0 ms ✓ | 1.3 ms ✓ |
| 3-partition-yes/8 | 2.0 ms ✓ | 1.1 ms ✓ | 2.4 ms ✓ | 2.4 ms ✓ | 2.3 ms ✓ | 2.5 ms ✓ | 2.8 ms ✓ |
| 3-partition-yes/12 | 30.1 ms ✓ | 16.5 ms ✓ | 37.7 ms ✓ | 29.6 ms ✓ | 29.3 ms ✓ | 29.5 ms ✓ | 30.4 ms ✓ |

## 3-partition-no

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| 3-partition-no/4 | 49.77 s ✗ | 25.38 s ✗ |  | 45.09 s ✗ | 23.49 s ✗ | 12.68 s ✗ | 7.45 s ✗ | 5.80 s ✗ |
| 3-partition-no/5 | > 300 s |  | 302.42 s ✗ | > 120 s | > 120 s | 77.70 s ✗ | 46.72 s ✗ | 37.07 s ✗ |

## qbf

| problem | families: classical auto j1 | long-2: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| qbf/8#0 | 551 µs ✗ |  |  |  |  |  |  |
| qbf/8#1 | 587 µs ✗ |  |  |  |  |  |  |
| qbf/8#2 | 830 µs ✓ |  |  |  |  |  |  |
| qbf/8#3 | 643 µs ✗ |  |  |  |  |  |  |
| qbf/12#0 | 121.0 ms ✓ |  |  |  |  |  |  |
| qbf/12#1 | 28.8 ms ✗ |  |  |  |  |  |  |
| qbf/12#2 | 45.0 ms ✓ |  |  |  |  |  |  |
| qbf/12#3 | 19.2 ms ✗ |  |  |  |  |  |  |
| qbf/16#0 | 7.87 s ✓ |  | 8.28 s ✓ | 6.37 s ✓ | 6.26 s ✓ | 6.32 s ✓ | 6.33 s ✓ |
| qbf/16#1 | 1.45 s ✗ |  | 1.34 s ✗ | 707.1 ms ✗ | 709.6 ms ✗ | 713.0 ms ✗ | 709.2 ms ✗ |
| qbf/16#2 | 2.09 s ✗ |  | 1.89 s ✗ | 1.21 s ✗ | 1.19 s ✗ | 1.21 s ✗ | 1.19 s ✗ |
| qbf/16#3 | 6.56 s ✓ |  | 5.99 s ✓ | 5.53 s ✓ | 5.52 s ✓ | 5.76 s ✓ | 5.50 s ✓ |
| qbf/20#0 | 87.23 s ✗ |  | 78.42 s ✗ | 43.03 s ✗ | 43.23 s ✗ | 43.89 s ✗ | 43.42 s ✗ |
| qbf/20#1 | 123.61 s ✗ |  | 109.45 s ✗ | 61.36 s ✗ | 61.74 s ✗ | 61.97 s ✗ | 61.09 s ✗ |
| qbf/20#2 | > 300 s |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |
| qbf/20#3 | 89.32 s ✗ |  | 81.26 s ✗ | 40.98 s ✗ | 41.17 s ✗ | 55.61 s ✗ | 41.57 s ✗ |
| qbf/24#0 | > 300 s | > 1200 s | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |
| qbf/24#1 | > 300 s |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |
| qbf/24#2 | > 300 s |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |
| qbf/24#3 | > 300 s |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |

## mix

| problem | families: mix auto j1 | long-2: mix auto j1 | parallel: mix auto j1 | parallel: mix auto j2 | parallel: mix auto j4 | parallel: mix auto j8 | parallel: mix auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| mix/4 | 275 µs ✗ |  |  |  |  |  |  |
| mix/6 | 19.9 ms ✗ |  |  |  |  |  |  |
| mix/8 | 1.91 s ✗ |  | 1.80 s ✗ | 1.86 s ✗ | 1.64 s ✗ | 1.67 s ✗ | 1.88 s ✗ |
| mix/9 | 19.82 s ✗ |  | 17.95 s ✗ | 17.97 s ✗ | 16.72 s ✗ | 16.01 s ✗ | 16.93 s ✗ |
| mix/10 | 213.99 s ✗ |  | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |
| mix/11 | > 300 s | > 1200 s | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |

## counter

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | long-1: classical auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|--:|
| counter/2 | 11 µs ✓ | 12 µs ✓ |  | 13 µs ✓ | 99 µs ✓ | 126 µs ✓ | 253 µs ✓ | 479 µs ✓ |
| counter/4 | 84 µs ✓ | 48 µs ✓ |  | 110 µs ✓ | 225 µs ✓ | 258 µs ✓ | 321 µs ✓ | 645 µs ✓ |
| counter/8 | 124.0 ms ✓ | 15.4 ms ✓ |  | 129.6 ms ✓ | 156.3 ms ✓ | 127.2 ms ✓ | 105.3 ms ✓ | 109.3 ms ✓ |
| counter/16 | > 300 s | > 300 s | > 1200 s | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |

## counter-over

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 | parallel: classical auto j1 | parallel: classical auto j2 | parallel: classical auto j4 | parallel: classical auto j8 | parallel: classical auto j16 |
|---|--:|--:|--:|--:|--:|--:|--:|
| counter-over/2 | 24 µs ? bound | 21 µs ? bound | 27 µs ? bound | 102 µs ? bound | 153 µs ? bound | 216 µs ? bound | 446 µs ? bound |
| counter-over/4 | 368 µs ? bound | 60 µs ? bound | 500 µs ? bound | 590 µs ? bound | 504 µs ? bound | 730 µs ? bound | 981 µs ? bound |
| counter-over/8 | 492.7 ms ? bound | 18.4 ms ? bound | 549.9 ms ? bound | 502.6 ms ? bound | 389.9 ms ? bound | 355.2 ms ? bound | 323.2 ms ? bound |
| counter-over/16 | > 300 s | > 300 s | > 120 s | > 120 s | > 120 s | > 120 s | > 120 s |

## growing

| problem | families: classical auto j1 |
|---|--:|
| growing/16 | 142 µs ? bound |
| growing/64 | 2.1 ms ? bound |
| growing/256 | 79.3 ms ? bound |
| growing/1024 | 567.8 ms ? depth |

## chain

| problem | families: classical auto j1 | intuitionistic: intuitionistic auto j1 |
|---|--:|--:|
| chain/16 | 247 µs ✓ | 273 µs ✓ |
| chain/64 | 22.1 ms ✓ | 22.6 ms ✓ |
| chain/128 | 269.7 ms ✓ | 275.9 ms ✓ |
| chain/256 | 2.82 s ✓ | 2.89 s ✓ |
