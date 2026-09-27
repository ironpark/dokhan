# CHM benchmark

Run the ignored benchmarks with a ZIP containing `merge17.chm`:

```sh
cd src-tauri
DOKHAN_TEST_ZIP=/absolute/path/to/dictionary8.zip cargo test --lib bench_chm_runtime_build -- --ignored --nocapture --test-threads=1
DOKHAN_TEST_ZIP=/absolute/path/to/dictionary8.zip cargo test --lib bench_chm_page_reads -- --ignored --nocapture --test-threads=1
cargo test --lib bench_basename_lookup -- --ignored --nocapture
DOKHAN_TEST_ZIP=/absolute/path/to/dictionary8.zip cargo test --lib bench_tantivy_writer_configs -- --ignored --nocapture --test-threads=1
```

Measurements below used `dictionary8.zip` (121 CHMs, 80,024 index entries), the Rust test profile, and the same Mac. Times are elapsed time inside each benchmark; they exclude compilation. These are comparison samples, not a statistically controlled benchmark.

| Operation | Before | After |
| --- | ---: | ---: |
| CHM parse, text extraction, and search key precompute | 6.31 s | 1.59–1.81 s |
| Read 100 HTML pages, cloning a fresh archive per page | 144 ms | — |
| Read 100 HTML pages through the shared runtime cache | — | 8.7–10.0 ms |
| Render 100 entry details from one CHM after index build | — | 36.6 ms |

The CHM build benchmark excludes Tantivy index creation and runtime cache persistence. It also prints scan, parallel parse, and finalization/search key times. The page benchmark prints reusable-archive and fresh-clone times. The shared runtime cache keeps decompressed blocks and LZX stream state for later page requests. The build improvements come from avoiding unnecessary LZX state allocations, deferring HTML sanitization until entry detail is requested, reducing text-processing allocations, and computing search keys without filling global normalization caches. Plain text is still built up front for full-text search. `DOKHAN_TEST_ZIP` enables the real ZIP regression tests; without it, those tests return early.

The synthetic basename benchmark uses 20,000 directory entries and 1,000 missing-path lookups. In the test profile, scanning all entries took 4.28 s; indexed lookup took 1.35 ms, with 9.81 ms to build the index. This isolates lookup cost and does not estimate the speedup of normal page reads.

## Tantivy index creation

The writer benchmark parses the ZIP once, then builds a fresh on-disk index for each writer configuration. On this 10-core, 16 GB Mac, the 80,024-entry index took 4.84–5.05 s with Tantivy's automatic worker count and a 50 MB total writer budget. Raising the total budget to 200 MB lets Tantivy use eight workers and reduced the build to 1.79–1.85 s. A 160 MB budget with eight workers took 1.94 s; 100 MB with four workers took 3.05–3.22 s. These are test-profile samples and exclude ZIP parsing and compilation. The 200 MB budget is a maximum writer target, not a fixed allocation; the worker count adjusts to available CPUs.

In separate single-configuration runs with `/usr/bin/time -l`, peak resident memory was about 325 MB for the 50 MB writer and 395 MB for the 200 MB writer. This includes the parsed runtime and test process. The larger writer budget trades memory for a faster first index build. Later loads reuse the persisted index. During a new index build, runtime loading completes and simple searches use the existing linear fallback until Tantivy is ready. Queries with Tantivy syntax wait for the index to preserve their meaning. If an index build is interrupted, the next load checks its document count and rebuilds incomplete indexes.
