use super::lzx;
use std::collections::BTreeMap;
use std::sync::Arc;

mod compression;
mod directory;
mod headers;

use compression::{parse_compression_context, CompressionContext};
use headers::parse_container_layout;

#[derive(Debug, Clone)]
pub struct DirectoryEntry {
    pub path: String,
    pub space: u64,
    pub start: u64,
    pub length: u64,
}

#[derive(Debug, Clone)]
pub struct ChmArchive {
    data: Arc<[u8]>,
    data_offset: u64,
    entries: Vec<DirectoryEntry>,
    by_path: BTreeMap<String, usize>,
    compression: Option<CompressionContext>,
    block_cache: BTreeMap<u64, Arc<[u8]>>,
    native_streams: BTreeMap<u64, NativeStream>,
}

#[derive(Debug, Clone)]
struct NativeStream {
    next_block: u64,
    state: lzx::LzxState,
}

#[derive(Debug)]
pub enum ChmError {
    InvalidFormat(&'static str),
    OutOfBounds,
    UnsupportedCompressedObject,
    DecompressionFailed(String),
    Utf8Path,
}

impl std::fmt::Display for ChmError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChmError::InvalidFormat(s) => write!(f, "invalid CHM format: {s}"),
            ChmError::OutOfBounds => write!(f, "CHM read out of bounds"),
            ChmError::UnsupportedCompressedObject => write!(f, "compressed object is not supported yet"),
            ChmError::DecompressionFailed(s) => write!(f, "CHM decompression failed: {s}"),
            ChmError::Utf8Path => write!(f, "invalid UTF-8 path in PMGL entry"),
        }
    }
}

impl std::error::Error for ChmError {}

impl ChmArchive {
    pub fn open(data: impl Into<Arc<[u8]>>) -> Result<Self, ChmError> {
        let data = data.into();
        let layout = parse_container_layout(&data)?;
        let entries = directory::parse_directory_entries(
            &data,
            layout.blocks_offset,
            layout.block_len,
            layout.num_blocks,
        )?;

        let mut by_path = BTreeMap::new();
        for (i, e) in entries.iter().enumerate() {
            by_path.insert(e.path.to_ascii_lowercase(), i);
        }
        let compression = parse_compression_context(&data, layout.data_offset, &entries, &by_path)?;

        Ok(Self {
            data,
            data_offset: layout.data_offset,
            entries,
            by_path,
            compression,
            block_cache: BTreeMap::new(),
            native_streams: BTreeMap::new(),
        })
    }

    pub fn entries(&self) -> &[DirectoryEntry] {
        &self.entries
    }

    pub fn find_entry(&self, path: &str) -> Option<&DirectoryEntry> {
        let raw = path.to_ascii_lowercase();
        if let Some(idx) = self.by_path.get(&raw) {
            return self.entries.get(*idx);
        }
        let trimmed = path.trim_start_matches('/').to_ascii_lowercase();
        let idx = self.by_path.get(&trimmed)?;
        self.entries.get(*idx)
    }

    pub fn read_object(&mut self, path: &str) -> Result<Vec<u8>, ChmError> {
        let entry = self
            .find_entry(path)
            .ok_or(ChmError::InvalidFormat("entry not found"))?;
        if entry.space != 0 {
            return self.read_compressed_object(entry.start, entry.length);
        }
        let start = self.data_offset as usize + entry.start as usize;
        let end = start + entry.length as usize;
        let bytes = self.data.get(start..end).ok_or(ChmError::OutOfBounds)?;
        Ok(bytes.to_vec())
    }

    fn read_compressed_object(&mut self, start: u64, len: u64) -> Result<Vec<u8>, ChmError> {
        let ctx = self
            .compression
            .as_ref()
            .ok_or(ChmError::UnsupportedCompressedObject)?
            .clone();
        if ctx.block_len == 0 {
            return Err(ChmError::UnsupportedCompressedObject);
        }

        let mut out = Vec::with_capacity(len as usize);
        let mut remaining = len;
        let mut pos = start;

        while remaining > 0 {
            let block = pos / ctx.block_len;
            let offset_in_block = (pos % ctx.block_len) as usize;
            let take = std::cmp::min(remaining, ctx.block_len - (offset_in_block as u64)) as usize;
            let block_data = self.decompress_block(&ctx, block)?;
            if offset_in_block + take > block_data.len() {
                return Err(ChmError::OutOfBounds);
            }
            out.extend_from_slice(&block_data[offset_in_block..offset_in_block + take]);
            remaining -= take as u64;
            pos += take as u64;
        }

        Ok(out)
    }

    fn decompress_block(&mut self, ctx: &CompressionContext, block: u64) -> Result<Arc<[u8]>, ChmError> {
        if let Some(v) = self.block_cache.get(&block) {
            return Ok(Arc::clone(v));
        }

        let out = self.decompress_block_native(ctx, block)?;

        self.block_cache.insert(block, Arc::clone(&out));
        Ok(out)
    }

    fn decompress_block_native(&mut self, ctx: &CompressionContext, block: u64) -> Result<Arc<[u8]>, ChmError> {
        let window_size = ctx.lzx_params.window_size;
        let window_bits = (32 - window_size.leading_zeros()) as u8 - 1;
        let reset_blkcount = std::cmp::max(ctx.lzx_params.reset_blkcount as u64, 1);
        let reset_base = block - (block % reset_blkcount);
        let mut stream = self.native_streams.remove(&reset_base).unwrap_or(NativeStream {
            next_block: reset_base,
            state: lzx::LzxState::new(window_bits).map_err(ChmError::DecompressionFailed)?,
        });

        // If the requested block is behind stream progress, rebuild from reset base.
        if stream.next_block > block {
            stream = NativeStream {
                next_block: reset_base,
                state: lzx::LzxState::new(window_bits).map_err(ChmError::DecompressionFailed)?,
            };
        }

        let mut target = None::<Arc<[u8]>>;
        let mut failed = None;
        for b in stream.next_block..=block {
            let cmp = self.read_compressed_block_bytes(ctx, b)?;
            let out_len = compression::block_output_len(ctx, b) as usize;
            let padded_cmp = pad_for_lzx(cmp);
            match lzx::decompress_block(&mut stream.state, &padded_cmp, out_len) {
                Ok(out) => {
                    let out: Arc<[u8]> = Arc::from(out.into_boxed_slice());
                    self.block_cache.insert(b, Arc::clone(&out));
                    if b == block {
                        target = Some(out);
                    }
                }
                Err(e) => {
                    failed = Some((b, e));
                    break;
                }
            }
        }
        if let Some((failed_block, failed_err)) = failed {
            // Correctness-first fallback: retry once from reset base with a fresh state.
            let fresh = self.decompress_block_native_fresh(ctx, block, window_bits);
            match fresh {
                Ok((v, fresh_state)) => {
                    self.native_streams.insert(
                        reset_base,
                        NativeStream {
                            next_block: block.saturating_add(1),
                            state: fresh_state,
                        },
                    );
                    return Ok(v);
                }
                Err(fresh_err) => {
                    return Err(ChmError::DecompressionFailed(format!(
                        "stream decode failed at block {failed_block}: {failed_err}; fresh retry failed: {fresh_err}"
                    )));
                }
            }
        }

        stream.next_block = block.saturating_add(1);
        self.native_streams.insert(reset_base, stream);
        Ok(target.unwrap_or_else(|| Arc::<[u8]>::from(&[][..])))
    }

    fn decompress_block_native_fresh(
        &mut self,
        ctx: &CompressionContext,
        block: u64,
        bits: u8,
    ) -> Result<(Arc<[u8]>, lzx::LzxState), String> {
        let reset_blkcount = std::cmp::max(ctx.lzx_params.reset_blkcount as u64, 1);
        let reset_base = block - (block % reset_blkcount);
        let mut state = lzx::LzxState::new(bits)?;
        let mut target = None::<Arc<[u8]>>;
        for b in reset_base..=block {
            let cmp = self
                .read_compressed_block_bytes(ctx, b)
                .map_err(|e| format!("read compressed block {b} failed: {e}"))?;
            let out_len = compression::block_output_len(ctx, b) as usize;
            let padded_cmp = pad_for_lzx(cmp);
            let out = lzx::decompress_block(&mut state, &padded_cmp, out_len)
                .map_err(|e| format!("fresh decode failed at block {b}: {e}"))?;
            let out: Arc<[u8]> = Arc::from(out.into_boxed_slice());
            self.block_cache.insert(b, Arc::clone(&out));
            if b == block {
                target = Some(out);
            }
        }
        Ok((target.unwrap_or_else(|| Arc::<[u8]>::from(&[][..])), state))
    }

    fn read_compressed_block_bytes<'a>(&'a self, ctx: &CompressionContext, block: u64) -> Result<&'a [u8], ChmError> {
        if block >= ctx.block_count as u64 {
            return Err(ChmError::OutOfBounds);
        }
        let start_off = *ctx
            .block_offsets
            .get(block as usize)
            .ok_or(ChmError::OutOfBounds)?;
        let end_off = if (block as usize + 1) < ctx.block_offsets.len() {
            ctx.block_offsets[block as usize + 1]
        } else {
            ctx.compressed_len
        };
        if end_off < start_off {
            return Err(ChmError::InvalidFormat("invalid block offset ordering"));
        }
        let abs_start = self.data_offset + ctx.content_start + start_off;
        let abs_end = self.data_offset + ctx.content_start + end_off;
        let bytes = self
            .data
            .get(abs_start as usize..abs_end as usize)
            .ok_or(ChmError::OutOfBounds)?;
        Ok(bytes)
    }
}

fn pad_for_lzx(bytes: &[u8]) -> Vec<u8> {
    let mut padded = Vec::with_capacity(bytes.len() + 2);
    padded.extend_from_slice(bytes);
    padded.extend_from_slice(&[0, 0]);
    padded
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;
    use std::fs;
    use std::fs::File;
    use std::io::Read;
    use std::path::PathBuf;
    use zip::ZipArchive;

    fn find_dataset_zip() -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        for ancestor in cwd.ancestors() {
            let p = ancestor.join("asset/dictionary_v77.zip");
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    fn load_chm_bytes_from_zip(chm_name: &str) -> Option<Vec<u8>> {
        let zip_path = find_dataset_zip()?;
        let file = File::open(zip_path).ok()?;
        let mut zip = ZipArchive::new(file).ok()?;
        let mut entry = zip.by_name(chm_name).ok()?;
        let mut buf = Vec::new();
        if entry.read_to_end(&mut buf).is_err() {
            return None;
        }
        Some(buf)
    }

    fn find_debug_dir(name: &str) -> Option<PathBuf> {
        let cwd = std::env::current_dir().ok()?;
        for ancestor in cwd.ancestors() {
            let p = ancestor.join("asset/debug").join(name);
            if p.exists() {
                return Some(p);
            }
        }
        None
    }

    fn entry_path_map_by_basename(chm: &ChmArchive) -> BTreeMap<String, String> {
        let mut m = BTreeMap::new();
        for e in chm.entries() {
            let base = e.path.rsplit('/').next().unwrap_or(&e.path).to_ascii_lowercase();
            m.entry(base).or_insert_with(|| e.path.clone());
        }
        m
    }

    #[test]
    fn can_open_real_chm_and_parse_entries() {
        let Some(bytes) = load_chm_bytes_from_zip("merge36.chm") else {
            return;
        };
        let layout = headers::parse_container_layout(&bytes).expect("layout");
        let entries = directory::parse_directory_entries(
            &bytes,
            layout.blocks_offset,
            layout.block_len,
            layout.num_blocks,
        )
        .expect("directory entries");
        assert!(!entries.is_empty());

        let chm = ChmArchive::open(bytes).expect("open chm");
        assert!(!chm.entries().is_empty());
    }

    #[test]
    fn can_read_uncompressed_control_data() {
        let Some(bytes) = load_chm_bytes_from_zip("merge36.chm") else {
            return;
        };
        let mut chm = ChmArchive::open(bytes).expect("open chm");
        let ctl = chm.read_object("::DataSpace/Storage/MSCompressed/ControlData");
        assert!(ctl.is_ok());
        let ctl = ctl.expect("control data");
        assert!(ctl.len() >= 0x18);
    }

    #[test]
    fn can_read_at_least_one_compressed_html_object() {
        let Some(bytes) = load_chm_bytes_from_zip("merge36.chm") else {
            return;
        };
        let mut chm = ChmArchive::open(bytes).expect("open chm");
        let candidates = chm
            .entries()
            .iter()
            .filter(|e| e.space != 0 && e.path.to_ascii_lowercase().ends_with(".htm"))
            .take(64)
            .map(|e| e.path.clone())
            .collect::<Vec<_>>();
        assert!(!candidates.is_empty());

        let mut success = 0usize;
        let mut failures = Vec::new();
        for p in candidates {
            match chm.read_object(&p) {
                Ok(v) => {
                    if !v.is_empty() {
                        success += 1;
                        break;
                    }
                }
                Err(e) => {
                    if failures.len() < 8 {
                        failures.push(e.to_string());
                    }
                }
            }
        }
        assert!(
            success > 0,
            "no compressed html object could be decoded; sample errors: {:?}",
            failures
        );
    }

    #[test]
    fn golden_merge36_html_matches_debug_extraction() {
        let Some(bytes) = load_chm_bytes_from_zip("merge36.chm") else {
            return;
        };
        let Some(debug_dir) = find_debug_dir("merge36") else {
            return;
        };
        let mut chm = ChmArchive::open(bytes).expect("open chm");

        let path_map = entry_path_map_by_basename(&chm);
        let mut golden_files = fs::read_dir(&debug_dir)
            .ok()
            .into_iter()
            .flat_map(|it| it.filter_map(Result::ok))
            .filter(|e| e.path().is_file())
            .filter_map(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy().to_string();
                let lower = name.to_ascii_lowercase();
                if lower.ends_with(".htm") && path_map.contains_key(&lower) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        golden_files.sort();
        golden_files.truncate(8);

        let mut compared = 0usize;
        let mut decode_errors = Vec::new();
        for name in golden_files {
            let expected_path = debug_dir.join(&name);
            let Ok(expected) = fs::read(&expected_path) else {
                continue;
            };
            let chm_path = path_map
                .get(&name.to_ascii_lowercase())
                .expect("name selected from path_map must exist")
                .clone();
            let actual = match chm.read_object(&chm_path) {
                Ok(v) => v,
                Err(e) => {
                    if decode_errors.len() < 8 {
                        decode_errors.push(format!("{name}: {e}"));
                    }
                    continue;
                }
            };
            assert_eq!(actual, expected, "decoded bytes mismatch for {name}");
            compared += 1;
        }

        assert!(
            compared >= 1,
            "no golden file compared successfully; decode errors: {:?}",
            decode_errors
        );
    }

    #[test]
    fn golden_master_nav_files_match_debug_extraction() {
        let Some(bytes) = load_chm_bytes_from_zip("master.chm") else {
            return;
        };
        let Some(debug_dir) = find_debug_dir("master") else {
            return;
        };
        let mut chm = ChmArchive::open(bytes).expect("open master chm");

        let path_map = entry_path_map_by_basename(&chm);
        let mut golden_files = fs::read_dir(&debug_dir)
            .ok()
            .into_iter()
            .flat_map(|it| it.filter_map(Result::ok))
            .filter(|e| e.path().is_file())
            .filter_map(|e| {
                let name = e.file_name();
                let name = name.to_string_lossy().to_string();
                let lower = name.to_ascii_lowercase();
                let is_nav = lower.ends_with(".hhc") || lower.ends_with(".hhk") || lower.ends_with(".html");
                if is_nav && path_map.contains_key(&lower) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        golden_files.sort();
        golden_files.truncate(6);
        let mut compared = 0usize;
        let mut decode_errors = Vec::new();
        for name in golden_files {
            let expected_path = debug_dir.join(&name);
            let Ok(expected) = fs::read(&expected_path) else {
                continue;
            };
            let chm_path = path_map
                .get(&name.to_ascii_lowercase())
                .expect("name selected from path_map must exist")
                .clone();
            let actual = match chm.read_object(&chm_path) {
                Ok(v) => v,
                Err(e) => {
                    if decode_errors.len() < 8 {
                        decode_errors.push(format!("{name}: {e}"));
                    }
                    continue;
                }
            };
            assert_eq!(actual, expected, "decoded bytes mismatch for {name}");
            compared += 1;
        }

        assert!(
            compared >= 1,
            "no master golden file compared successfully; decode errors: {:?}",
            decode_errors
        );
    }

    #[test]
    fn strict_golden_merge36_all_html_match_debug_extraction() {
        let Some(bytes) = load_chm_bytes_from_zip("merge36.chm") else {
            return;
        };
        let Some(debug_dir) = find_debug_dir("merge36") else {
            return;
        };
        let mut chm = ChmArchive::open(bytes).expect("open chm");
        let path_map = entry_path_map_by_basename(&chm);
        let mut mismatches = Vec::new();
        let mut decode_errors = Vec::new();
        let mut compared = 0usize;

        let mut names = fs::read_dir(&debug_dir)
            .ok()
            .into_iter()
            .flat_map(|it| it.filter_map(Result::ok))
            .filter(|e| e.path().is_file())
            .filter_map(|e| {
                let name = e.file_name().to_string_lossy().to_string();
                let lower = name.to_ascii_lowercase();
                if lower.ends_with(".htm") && path_map.contains_key(&lower) {
                    Some(name)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();
        names.sort();

        for name in names {
            let expected = match fs::read(debug_dir.join(&name)) {
                Ok(v) => v,
                Err(_) => continue,
            };
            let chm_path = path_map
                .get(&name.to_ascii_lowercase())
                .expect("selected name must exist")
                .clone();
            match chm.read_object(&chm_path) {
                Ok(actual) => {
                    compared += 1;
                    if actual != expected && mismatches.len() < 16 {
                        mismatches.push(name);
                    }
                }
                Err(e) => {
                    if decode_errors.len() < 16 {
                        decode_errors.push(format!("{name}: {e}"));
                    }
                }
            }
        }

        assert!(
            decode_errors.is_empty() && mismatches.is_empty(),
            "strict golden failed; compared={compared}, decode_errors={:?}, mismatches={:?}",
            decode_errors,
            mismatches
        );
    }

    #[test]
    fn exhaustive_decode_all_chm_in_dictionary_v77() {
        let cwd = std::env::current_dir().expect("cwd");
        let dict_dir = cwd
            .ancestors()
            .map(|p| p.join("asset/dictionary_v77"))
            .find(|p| p.exists())
            .expect("asset/dictionary_v77 not found");

        let mut chm_files = fs::read_dir(&dict_dir)
            .expect("read dictionary_v77")
            .filter_map(Result::ok)
            .map(|e| e.path())
            .filter(|p| p.is_file())
            .filter(|p| {
                p.extension()
                    .map(|e| e.to_string_lossy().eq_ignore_ascii_case("chm"))
                    .unwrap_or(false)
            })
            .collect::<Vec<_>>();
        chm_files.sort();
        assert!(!chm_files.is_empty(), "no .chm files found in {}", dict_dir.display());

        let mut total_chm = 0usize;
        let mut total_html = 0usize;
        let mut failures = Vec::new();

        for chm_path in chm_files {
            total_chm += 1;
            let name = chm_path
                .file_name()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| chm_path.display().to_string());
            let bytes = match fs::read(&chm_path) {
                Ok(v) => v,
                Err(e) => {
                    failures.push(format!("{name}: read failed: {e}"));
                    continue;
                }
            };
            let mut chm = match ChmArchive::open(bytes) {
                Ok(v) => v,
                Err(e) => {
                    failures.push(format!("{name}: open failed: {e}"));
                    continue;
                }
            };
            let html_paths = chm
                .entries()
                .iter()
                .filter(|e| {
                    let p = e.path.to_ascii_lowercase();
                    p.ends_with(".htm") || p.ends_with(".html")
                })
                .map(|e| e.path.clone())
                .collect::<Vec<_>>();

            for p in html_paths {
                total_html += 1;
                if let Err(e) = chm.read_object(&p) {
                    failures.push(format!("{name}: {p}: {e}"));
                    if failures.len() >= 200 {
                        break;
                    }
                }
            }
            if failures.len() >= 200 {
                break;
            }
        }

        assert!(
            failures.is_empty(),
            "exhaustive decode failed: chm={}, html={}, failures(sample up to 200)={:?}",
            total_chm,
            total_html,
            failures
        );
    }
}
