# CHM benchmark

Run the ignored benchmarks with a ZIP containing `merge17.chm`:

```sh
cd src-tauri
DOKHAN_TEST_ZIP=/absolute/path/to/dictionary8.zip cargo test --lib bench_chm -- --ignored --nocapture
cargo test --lib bench_basename_lookup -- --ignored --nocapture
```

Measurements below used `dictionary8.zip` (121 CHMs, 80,024 index entries), the Rust test profile, and the same Mac. Times are elapsed time inside each benchmark; they exclude compilation. These are comparison samples, not a statistically controlled benchmark.

| Operation | Before | After |
| --- | ---: | ---: |
| Full CHM runtime build | 6.31 s | 2.89–2.98 s |
| Read 100 HTML pages, cloning a fresh archive per page | 144 ms | — |
| Read 100 HTML pages through the shared runtime cache | — | 8.7–10.0 ms |
| Render 100 entry details from one CHM after index build | — | 36.6 ms |

The page benchmark also prints the reusable-archive and fresh-clone times in the current build. The shared runtime cache keeps decompressed blocks and LZX stream state for later page requests. The build improvements come from avoiding unnecessary LZX state allocations and deferring HTML sanitization until entry detail is requested. Plain text is still built up front for full-text search. `DOKHAN_TEST_ZIP` enables the real ZIP regression tests; without it, those tests return early.

The synthetic basename benchmark uses 20,000 directory entries and 1,000 missing-path lookups. In the test profile, scanning all entries took 4.28 s; indexed lookup took 1.35 ms, with 9.81 ms to build the index. This isolates lookup cost and does not estimate the speedup of normal page reads.
