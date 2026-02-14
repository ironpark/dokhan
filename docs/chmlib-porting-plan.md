# CHMLib to Pure Rust Porting Plan

## Goal
- Remove external `7zz` dependency.
- Parse CHM (`ITSF/ITSP/PMGL/PMGI`) natively in Rust.
- Build dictionary index/content in process memory runtime.

## Source Reference
- Upstream C library: `third_party/CHMLib/src/chm_lib.c`, `third_party/CHMLib/src/lzx.c`

## Current Status
- Done:
- CHM header parser in Rust (`ITSF`, `ITSP`)
- Directory page parser in Rust (`PMGL`) with cword decoding
- Entry enumeration integrated into runtime index build path
- `master.hhc` read attempt for uncompressed object path
- Not done:
- PMGI branch search optimization
- LZX compressed object retrieval
- Full `chm_resolve_object` parity
- Optional persistent storage strategy (out of scope for CHM core port)

## Porting Phases
1. Directory/Metadata Parity
- Port `_unmarshal_itsf_header`, `_unmarshal_itsp_header`
- Port PMGL parser and `cword` decoding
- Add deterministic path normalization and case-insensitive lookup

2. Object Resolution Parity
- Port `_chm_find_in_PMGL` and `_chm_find_in_PMGI`
- Build `resolve_object(path)` equivalent to CHMLib
- Implement support for both uncompressed and compressed spaces

3. LZX Decompression
- Port `lzx.c` bitstream and huffman tables into Rust
- Add reset-table handling (`LZXC ControlData`, `ResetTable`)
- Add block cache for decompressed windows (simple ring cache first).
- Keep an optional `lzxd` experimental backend behind feature flag for A/B verification.
- Do not make `lzxd` mandatory runtime dependency; native port remains primary target.

4. App Integration
- Replace temporary entry-only indexing with resolved object parsing
- Parse `.hhk/.hhc/.htm` from CHM objects directly
- Keep runtime structures in memory (no DB dependency)

5. Validation
- Golden tests against known CHM set
- Compare entry count and lookup parity with `master.chm` behavior
- Fuzz invalid/corrupt CHM headers for safety

## Notes
- CHM strings are UTF-8-ish in directory entries, but content pages are often EUC-KR in this dataset.
- Keep parser zero-copy where practical, but prefer correctness first.
