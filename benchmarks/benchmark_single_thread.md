# Copy vs GNU cp — Cold Cache Benchmark Report
**Single-threaded Performance Comparison (-j=1)**

---

## Executive Summary

This benchmark compares the performance of `copy` (single-threaded mode) against GNU `cp` for copying files of various sizes under cold cache conditions.

## Test Environment

| Parameter | Value |
|-----------|-------|
| **CPU Cores** | 8 |
| **Operating System** | Linux 6.17.0-8-generic |
| **Date** | Wed Jan 28 11:33:12 IST 2026 |
| **GNU cp Version** | cp (GNU coreutils) 9.5 |
| **copy Configuration** | Single-threaded (-j=1) |
| **Cache Mode** | Cold (dropped before each run) |
| **Runs per Test** | 5 |
| **Total Dataset Size** | 19G |

## Test Methodology

- Cache cleared before each run using `echo 3 > /proc/sys/vm/drop_caches`
- Both source and destination directories removed between runs
- File system synced before each test
- Random data generated using `/dev/urandom`

## Benchmark Results

---

### file_10GB 

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `/home/happy/copy/copy -j=1 data/file_10GB dest_copy` | 11.902 ± 0.278 | 11.628 | 12.306 | 1.00 |
| `cp -r data/file_10GB dest_cp` | 13.746 ± 1.929 | 10.917 | 15.609 | 1.15 ± 0.16 |

---

### file_5GB 

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `/home/happy/copy/copy -j=1 data/file_5GB dest_copy` | 5.451 ± 0.068 | 5.350 | 5.510 | 1.07 ± 0.02 |
| `cp -r data/file_5GB dest_cp` | 5.101 ± 0.043 | 5.053 | 5.155 | 1.00 |

---

### file_2GB 

| Command | Mean [s] | Min [s] | Max [s] | Relative |
|:---|---:|---:|---:|---:|
| `/home/happy/copy/copy -j=1 data/file_2GB dest_copy` | 1.653 ± 0.150 | 1.503 | 1.840 | 1.00 ± 0.10 |
| `cp -r data/file_2GB dest_cp` | 1.645 ± 0.049 | 1.579 | 1.714 | 1.00 |

---

### file_1GB 

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `/home/happy/copy/copy -j=1 data/file_1GB dest_copy` | 594.7 ± 8.7 | 588.1 | 609.7 | 1.25 ± 0.02 |
| `cp -r data/file_1GB dest_cp` | 476.5 ± 2.9 | 473.1 | 479.8 | 1.00 |

---

### file_500MB 

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `/home/happy/copy/copy -j=1 data/file_500MB dest_copy` | 284.3 ± 4.9 | 280.3 | 292.6 | 1.27 ± 0.04 |
| `cp -r data/file_500MB dest_cp` | 223.7 ± 4.9 | 218.9 | 230.6 | 1.00 |

---

### file_100MB 

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `/home/happy/copy/copy -j=1 data/file_100MB dest_copy` | 67.6 ± 0.8 | 66.6 | 68.6 | 1.29 ± 0.03 |
| `cp -r data/file_100MB dest_cp` | 52.5 ± 1.1 | 51.7 | 54.4 | 1.00 |

---

### file_10MB 

| Command | Mean [ms] | Min [ms] | Max [ms] | Relative |
|:---|---:|---:|---:|---:|
| `/home/happy/copy/copy -j=1 data/file_10MB dest_copy` | 17.9 ± 0.4 | 17.5 | 18.3 | 1.30 ± 0.03 |
| `cp -r data/file_10MB dest_cp` | 13.8 ± 0.2 | 13.7 | 14.2 | 1.00 |
