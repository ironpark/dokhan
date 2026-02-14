use super::{headers::read_u32_le, ChmError, DirectoryEntry};

fn parse_cword(buf: &[u8], pos: &mut usize) -> Result<u64, ChmError> {
    let mut accum = 0u64;
    loop {
        let b = *buf.get(*pos).ok_or(ChmError::OutOfBounds)?;
        *pos += 1;
        if b < 0x80 {
            return Ok((accum << 7) + b as u64);
        }
        accum = (accum << 7) + (b as u64 & 0x7f);
    }
}

pub(crate) fn parse_directory_entries(
    data: &[u8],
    blocks_offset: usize,
    block_len: usize,
    num_blocks: usize,
) -> Result<Vec<DirectoryEntry>, ChmError> {
    let mut entries = Vec::new();

    for block_idx in 0..num_blocks {
        let base = blocks_offset + block_idx * block_len;
        let page = data.get(base..base + block_len).ok_or(ChmError::OutOfBounds)?;
        if page.get(0..4) != Some(b"PMGL") {
            continue;
        }
        let free_space = read_u32_le(page, 0x04)? as usize;
        if free_space > block_len {
            continue;
        }
        let mut pos = 0x14usize;
        let end = block_len - free_space;
        while pos < end {
            let name_len = match parse_cword(page, &mut pos) {
                Ok(v) => v as usize,
                Err(_) => break,
            };
            if name_len == 0 || pos + name_len > end {
                break;
            }
            let name = std::str::from_utf8(&page[pos..pos + name_len]).map_err(|_| ChmError::Utf8Path)?;
            pos += name_len;

            let space = match parse_cword(page, &mut pos) {
                Ok(v) => v,
                Err(_) => break,
            };
            let start = match parse_cword(page, &mut pos) {
                Ok(v) => v,
                Err(_) => break,
            };
            let length = match parse_cword(page, &mut pos) {
                Ok(v) => v,
                Err(_) => break,
            };
            entries.push(DirectoryEntry {
                path: name.to_string(),
                space,
                start,
                length,
            });
        }
    }

    Ok(entries)
}
