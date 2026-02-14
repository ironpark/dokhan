#![allow(dead_code)]

use super::lzx;
use std::collections::BTreeMap;

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
    data: Vec<u8>,
    data_offset: u64,
    entries: Vec<DirectoryEntry>,
    by_path: BTreeMap<String, usize>,
    compression: Option<CompressionContext>,
    block_cache: BTreeMap<u64, Vec<u8>>,
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
    pub fn open(data: Vec<u8>) -> Result<Self, ChmError> {
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
        let compression = parse_compression_context(&data, layout.data_offset, &entries, &by_path).ok().flatten();

        Ok(Self {
            data,
            data_offset: layout.data_offset,
            entries,
            by_path,
            compression,
            block_cache: BTreeMap::new(),
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

    fn decompress_block(&mut self, ctx: &CompressionContext, block: u64) -> Result<Vec<u8>, ChmError> {
        if let Some(v) = self.block_cache.get(&block) {
            return Ok(v.clone());
        }

        let out = self.decompress_block_native(ctx, block)?;

        self.block_cache.insert(block, out.clone());
        Ok(out)
    }

    fn decompress_block_native(&self, ctx: &CompressionContext, block: u64) -> Result<Vec<u8>, ChmError> {
        let ws = ctx.lzx_params.window_size;
        let bits = (32 - ws.leading_zeros()) as u8 - 1;
        let mut state = lzx::LzxState::new(bits).map_err(ChmError::DecompressionFailed)?;
        let reset_blkcount = std::cmp::max(ctx.lzx_params.reset_blkcount as u64, 1);
        let reset_base = block - (block % reset_blkcount);

        let mut target = Vec::new();
        for b in reset_base..=block {
            let cmp = self.read_compressed_block_bytes(ctx, b)?;
            let out_len = compression::block_output_len(ctx, b) as usize;
            let out = lzx::decompress_block(&mut state, &cmp, out_len)
                .map_err(ChmError::DecompressionFailed)?;
            if b == block {
                target = out;
            }
        }

        Ok(target)
    }

    fn read_compressed_block_bytes(&self, ctx: &CompressionContext, block: u64) -> Result<Vec<u8>, ChmError> {
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
        Ok(bytes.to_vec())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
}
