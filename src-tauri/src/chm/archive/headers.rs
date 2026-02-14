use super::ChmError;

#[derive(Debug, Clone, Copy)]
pub(crate) struct ContainerLayout {
    pub(crate) data_offset: u64,
    pub(crate) blocks_offset: usize,
    pub(crate) block_len: usize,
    pub(crate) num_blocks: usize,
    pub(crate) index_head: i32,
}

pub(crate) fn read_u32_le(buf: &[u8], off: usize) -> Result<u32, ChmError> {
    let s = buf.get(off..off + 4).ok_or(ChmError::OutOfBounds)?;
    Ok(u32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

pub(crate) fn read_i32_le(buf: &[u8], off: usize) -> Result<i32, ChmError> {
    Ok(read_u32_le(buf, off)? as i32)
}

pub(crate) fn read_u64_le(buf: &[u8], off: usize) -> Result<u64, ChmError> {
    let s = buf.get(off..off + 8).ok_or(ChmError::OutOfBounds)?;
    Ok(u64::from_le_bytes([
        s[0], s[1], s[2], s[3], s[4], s[5], s[6], s[7],
    ]))
}

pub(crate) fn parse_container_layout(data: &[u8]) -> Result<ContainerLayout, ChmError> {
    if data.get(0..4) != Some(b"ITSF") {
        return Err(ChmError::InvalidFormat("missing ITSF signature"));
    }
    let itsf_version = read_i32_le(data, 0x04)?;
    let itsf_header_len = read_i32_le(data, 0x08)? as usize;
    if itsf_version != 2 && itsf_version != 3 {
        return Err(ChmError::InvalidFormat("unsupported ITSF version"));
    }
    if (itsf_version == 2 && itsf_header_len < 0x58) || (itsf_version == 3 && itsf_header_len < 0x60) {
        return Err(ChmError::InvalidFormat("invalid ITSF header length"));
    }

    let dir_offset = read_u64_le(data, 0x48)? as usize;
    let dir_len = read_u64_le(data, 0x50)? as usize;
    let mut data_offset = if itsf_version == 3 {
        read_u64_le(data, 0x58)?
    } else {
        (dir_offset + dir_len) as u64
    };
    if data_offset == 0 {
        data_offset = (dir_offset + dir_len) as u64;
    }

    let itsp = data
        .get(dir_offset..dir_offset + 0x54)
        .ok_or(ChmError::OutOfBounds)?;
    if itsp.get(0..4) != Some(b"ITSP") {
        return Err(ChmError::InvalidFormat("missing ITSP signature"));
    }
    let block_len = read_u32_le(itsp, 0x10)? as usize;
    let header_len = read_i32_le(itsp, 0x08)? as usize;
    let num_blocks_raw = read_u32_le(itsp, 0x28)?;
    let index_head = read_i32_le(itsp, 0x20)?;
    let dir_blocks_len = dir_len
        .checked_sub(header_len)
        .ok_or(ChmError::InvalidFormat("invalid ITSP header len"))?;
    let num_blocks = if num_blocks_raw == u32::MAX {
        dir_blocks_len / block_len
    } else {
        num_blocks_raw as usize
    };
    if block_len == 0 || num_blocks == 0 {
        return Err(ChmError::InvalidFormat("invalid directory block info"));
    }

    Ok(ContainerLayout {
        data_offset,
        blocks_offset: dir_offset + header_len,
        block_len,
        num_blocks,
        index_head,
    })
}
