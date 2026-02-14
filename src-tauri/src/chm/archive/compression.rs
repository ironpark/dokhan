use std::collections::BTreeMap;

use super::{
    headers::{read_u32_le, read_u64_le},
    ChmError, DirectoryEntry,
};

const CHMU_RESET_TABLE: &str =
    "::DataSpace/Storage/MSCompressed/Transform/{7FC28940-9D31-11D0-9B27-00A0C91E9C7C}/InstanceData/ResetTable";
const CHMU_LZXC_CONTROLDATA: &str = "::DataSpace/Storage/MSCompressed/ControlData";
const CHMU_CONTENT: &str = "::DataSpace/Storage/MSCompressed/Content";

#[derive(Debug, Clone)]
pub(crate) struct CompressionContext {
    pub(crate) content_start: u64,
    pub(crate) block_len: u64,
    pub(crate) uncompressed_len: u64,
    pub(crate) compressed_len: u64,
    pub(crate) block_count: u32,
    pub(crate) block_offsets: Vec<u64>,
    pub(crate) lzx_params: LzxParams,
}

#[derive(Debug, Clone)]
pub(crate) struct LzxParams {
    pub(crate) window_size: u32,
    pub(crate) reset_blkcount: u32,
}

pub(crate) fn parse_compression_context(
    data: &[u8],
    data_offset: u64,
    entries: &[DirectoryEntry],
    by_path: &BTreeMap<String, usize>,
) -> Result<Option<CompressionContext>, ChmError> {
    let rt = by_path.get(&CHMU_RESET_TABLE.to_ascii_lowercase()).copied();
    let cd = by_path.get(&CHMU_LZXC_CONTROLDATA.to_ascii_lowercase()).copied();
    let cn = by_path.get(&CHMU_CONTENT.to_ascii_lowercase()).copied();
    let (Some(rt_idx), Some(cd_idx), Some(cn_idx)) = (rt, cd, cn) else {
        return Ok(None);
    };

    let rt_entry = entries.get(rt_idx).ok_or(ChmError::OutOfBounds)?;
    let cd_entry = entries.get(cd_idx).ok_or(ChmError::OutOfBounds)?;
    let cn_entry = entries.get(cn_idx).ok_or(ChmError::OutOfBounds)?;
    if rt_entry.space != 0 || cd_entry.space != 0 || cn_entry.space != 0 {
        return Ok(None);
    }

    let rt_bytes = read_uncompressed_bytes(data, data_offset, rt_entry)?;
    let cd_bytes = read_uncompressed_bytes(data, data_offset, cd_entry)?;

    let (window_size, reset_blkcount) = parse_lzxc_control_data(cd_bytes)?;
    let (block_len, uncompressed_len, compressed_len, block_count, block_offsets) =
        parse_lzxc_reset_table(rt_bytes)?;

    Ok(Some(CompressionContext {
        content_start: cn_entry.start,
        block_len,
        uncompressed_len,
        compressed_len,
        block_count,
        block_offsets,
        lzx_params: LzxParams {
            window_size,
            reset_blkcount,
        },
    }))
}

pub(crate) fn block_output_len(ctx: &CompressionContext, block: u64) -> u64 {
    let start = block.saturating_mul(ctx.block_len);
    if start >= ctx.uncompressed_len {
        return 0;
    }
    std::cmp::min(ctx.block_len, ctx.uncompressed_len - start)
}

fn read_uncompressed_bytes<'a>(
    data: &'a [u8],
    data_offset: u64,
    entry: &DirectoryEntry,
) -> Result<&'a [u8], ChmError> {
    let start = data_offset as usize + entry.start as usize;
    let end = start + entry.length as usize;
    data.get(start..end).ok_or(ChmError::OutOfBounds)
}

fn parse_lzxc_control_data(bytes: &[u8]) -> Result<(u32, u32), ChmError> {
    if bytes.len() < 0x18 {
        return Err(ChmError::InvalidFormat("LZXC ControlData too short"));
    }
    let signature = bytes.get(4..8).ok_or(ChmError::OutOfBounds)?;
    if signature != b"LZXC" {
        return Err(ChmError::InvalidFormat("invalid LZXC signature"));
    }
    let version = read_u32_le(bytes, 0x08)?;
    let mut reset_interval = read_u32_le(bytes, 0x0c)?;
    let mut window_size = read_u32_le(bytes, 0x10)?;
    let windows_per_reset = read_u32_le(bytes, 0x14)?;

    if version == 2 {
        reset_interval = reset_interval.saturating_mul(0x8000);
        window_size = window_size.saturating_mul(0x8000);
    }
    if window_size == 0 || reset_interval == 0 {
        return Err(ChmError::InvalidFormat("invalid LZXC control values"));
    }
    if !window_size.is_power_of_two() {
        return Err(ChmError::InvalidFormat("LZX window size must be power-of-two"));
    }
    let half_window = window_size / 2;
    if half_window == 0 || reset_interval % half_window != 0 {
        return Err(ChmError::InvalidFormat("unsupported reset/window relation"));
    }
    let reset_blkcount = (reset_interval / half_window).saturating_mul(windows_per_reset);
    if reset_blkcount == 0 {
        return Err(ChmError::InvalidFormat("invalid reset block count"));
    }
    Ok((window_size, reset_blkcount))
}

fn parse_lzxc_reset_table(bytes: &[u8]) -> Result<(u64, u64, u64, u32, Vec<u64>), ChmError> {
    if bytes.len() < 0x28 {
        return Err(ChmError::InvalidFormat("LZXC ResetTable too short"));
    }
    let version = read_u32_le(bytes, 0x00)?;
    if version != 2 {
        return Err(ChmError::InvalidFormat("unsupported ResetTable version"));
    }
    let block_count = read_u32_le(bytes, 0x04)?;
    let table_offset = read_u32_le(bytes, 0x0c)? as usize;
    let uncompressed_len = read_u64_le(bytes, 0x10)?;
    let compressed_len = read_u64_le(bytes, 0x18)?;
    let block_len = read_u64_le(bytes, 0x20)?;
    if block_count == 0 || block_len == 0 {
        return Err(ChmError::InvalidFormat("invalid ResetTable values"));
    }
    let table_bytes = (block_count as usize) * 8;
    if table_offset + table_bytes > bytes.len() {
        return Err(ChmError::OutOfBounds);
    }
    let mut block_offsets = Vec::with_capacity(block_count as usize);
    for i in 0..(block_count as usize) {
        block_offsets.push(read_u64_le(bytes, table_offset + i * 8)?);
    }
    Ok((block_len, uncompressed_len, compressed_len, block_count, block_offsets))
}
