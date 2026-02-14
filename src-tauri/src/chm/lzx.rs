#![allow(dead_code)]

const LZX_MIN_MATCH: usize = 2;
const LZX_MAX_MATCH: usize = 257;
const LZX_NUM_CHARS: usize = 256;

const LZX_BLOCKTYPE_INVALID: u16 = 0;
const LZX_BLOCKTYPE_VERBATIM: u16 = 1;
const LZX_BLOCKTYPE_ALIGNED: u16 = 2;
const LZX_BLOCKTYPE_UNCOMPRESSED: u16 = 3;

const LZX_PRETREE_NUM_ELEMENTS: usize = 20;
const LZX_ALIGNED_NUM_ELEMENTS: usize = 8;
const LZX_NUM_PRIMARY_LENGTHS: usize = 7;
const LZX_NUM_SECONDARY_LENGTHS: usize = 249;

const LZX_PRETREE_MAXSYMBOLS: usize = LZX_PRETREE_NUM_ELEMENTS;
const LZX_PRETREE_TABLEBITS: usize = 6;
const LZX_MAINTREE_MAXSYMBOLS: usize = LZX_NUM_CHARS + 50 * 8;
const LZX_MAINTREE_TABLEBITS: usize = 12;
const LZX_LENGTH_MAXSYMBOLS: usize = LZX_NUM_SECONDARY_LENGTHS + 1;
const LZX_LENGTH_TABLEBITS: usize = 12;
const LZX_ALIGNED_MAXSYMBOLS: usize = LZX_ALIGNED_NUM_ELEMENTS;
const LZX_ALIGNED_TABLEBITS: usize = 7;
const LZX_LENTABLE_SAFETY: usize = 64;

const DECR_OK: i32 = 0;
const DECR_DATAFORMAT: i32 = 1;
const DECR_ILLEGALDATA: i32 = 2;

const EXTRA_BITS: [u8; 51] = [
    0, 0, 0, 0, 1, 1, 2, 2, 3, 3, 4, 4, 5, 5, 6, 6, 7, 7, 8, 8, 9, 9, 10, 10, 11, 11, 12, 12,
    13, 13, 14, 14, 15, 15, 16, 16, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17, 17,
];

const POSITION_BASE: [u32; 51] = [
    0, 1, 2, 3, 4, 6, 8, 12, 16, 24, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024, 1536,
    2048, 3072, 4096, 6144, 8192, 12288, 16384, 24576, 32768, 49152, 65536, 98304, 131072,
    196608, 262144, 393216, 524288, 655360, 786432, 917504, 1048576, 1179648, 1310720, 1441792,
    1572864, 1703936, 1835008, 1966080, 2097152,
];

#[derive(Debug, Clone)]
pub struct LzxState {
    pub window_bits: u8,
    pub window_size: usize,
    pub window_posn: usize,
    pub r0: u32,
    pub r1: u32,
    pub r2: u32,
    pub main_elements: usize,
    pub header_read: bool,
    pub block_type: u16,
    pub block_length: usize,
    pub block_remaining: usize,
    pub frames_read: u32,
    pub intel_filesize: i32,
    pub intel_curpos: i32,
    pub intel_started: bool,
    window: Vec<u8>,

    pretree_table: Vec<u16>,
    pretree_len: Vec<u8>,
    maintree_table: Vec<u16>,
    maintree_len: Vec<u8>,
    length_table: Vec<u16>,
    length_len: Vec<u8>,
    aligned_table: Vec<u16>,
    aligned_len: Vec<u8>,
}

#[derive(Debug, Clone)]
struct BitReader<'a> {
    input: &'a [u8],
    pos: usize,
    bitbuf: u32,
    bitsleft: u32,
}

impl<'a> BitReader<'a> {
    fn new(input: &'a [u8]) -> Self {
        Self {
            input,
            pos: 0,
            bitbuf: 0,
            bitsleft: 0,
        }
    }

    fn align_reset(&mut self) {
        self.bitbuf = 0;
        self.bitsleft = 0;
    }

    fn ensure_bits(&mut self, n: u32) -> Result<(), i32> {
        while self.bitsleft < n {
            let lo = *self.input.get(self.pos).ok_or(DECR_ILLEGALDATA)? as u32;
            let hi = *self.input.get(self.pos + 1).ok_or(DECR_ILLEGALDATA)? as u32;
            self.pos += 2;
            self.bitbuf |= (hi << 8 | lo) << (32 - 16 - self.bitsleft);
            self.bitsleft += 16;
        }
        Ok(())
    }

    fn peek_bits(&self, n: u32) -> u32 {
        self.bitbuf >> (32 - n)
    }

    fn remove_bits(&mut self, n: u32) {
        self.bitbuf <<= n;
        self.bitsleft -= n;
    }

    fn read_bits(&mut self, n: u32) -> Result<u32, i32> {
        self.ensure_bits(n)?;
        let v = self.peek_bits(n);
        self.remove_bits(n);
        Ok(v)
    }
}

fn make_decode_table(nsyms: usize, nbits: usize, length: &[u8], table: &mut [u16]) -> Result<(), i32> {
    let mut bit_num = 1usize;
    let mut pos = 0usize;
    let mut table_mask = 1usize << nbits;
    let mut bit_mask = table_mask >> 1;
    let mut next_symbol = bit_mask;

    while bit_num <= nbits {
        for (sym, &len) in length.iter().take(nsyms).enumerate() {
            if len as usize == bit_num {
                let mut leaf = pos;
                pos += bit_mask;
                if pos > table_mask {
                    return Err(DECR_ILLEGALDATA);
                }
                let mut fill = bit_mask;
                while fill > 0 {
                    table[leaf] = sym as u16;
                    leaf += 1;
                    fill -= 1;
                }
            }
        }
        bit_mask >>= 1;
        bit_num += 1;
    }

    if pos != table_mask {
        for v in table.iter_mut().take(table_mask).skip(pos) {
            *v = 0;
        }

        pos <<= 16;
        table_mask <<= 16;
        bit_mask = 1 << 15;

        while bit_num <= 16 {
            for (sym, &len) in length.iter().take(nsyms).enumerate() {
                if len as usize == bit_num {
                    let mut leaf = pos >> 16;
                    for fill in 0..(bit_num - nbits) {
                        if table[leaf] == 0 {
                            let left = next_symbol << 1;
                            if left + 1 >= table.len() {
                                return Err(DECR_ILLEGALDATA);
                            }
                            table[left] = 0;
                            table[left + 1] = 0;
                            table[leaf] = next_symbol as u16;
                            next_symbol += 1;
                        }
                        leaf = (table[leaf] as usize) << 1;
                        if ((pos >> (15 - fill)) & 1) != 0 {
                            leaf += 1;
                        }
                    }
                    table[leaf] = sym as u16;
                    pos += bit_mask;
                    if pos > table_mask {
                        return Err(DECR_ILLEGALDATA);
                    }
                }
            }
            bit_mask >>= 1;
            bit_num += 1;
        }
    }

    if pos == table_mask {
        return Ok(());
    }

    if length.iter().take(nsyms).any(|&x| x != 0) {
        return Err(DECR_ILLEGALDATA);
    }
    Ok(())
}

fn read_huffsym(
    br: &mut BitReader<'_>,
    tablebits: usize,
    maxsymbols: usize,
    lentable: &[u8],
    hufftbl: &[u16],
) -> Result<usize, i32> {
    br.ensure_bits(16)?;
    let mut i = hufftbl[br.peek_bits(tablebits as u32) as usize] as usize;
    if i >= maxsymbols {
        let mut j = 1u32 << (32 - tablebits as u32);
        loop {
            j >>= 1;
            i <<= 1;
            if (br.bitbuf & j) != 0 {
                i |= 1;
            }
            if j == 0 {
                return Err(DECR_ILLEGALDATA);
            }
            i = hufftbl[i] as usize;
            if i < maxsymbols {
                break;
            }
        }
    }
    let n = *lentable.get(i).ok_or(DECR_ILLEGALDATA)? as u32;
    br.remove_bits(n);
    Ok(i)
}

fn lzx_read_lens(
    state: &mut LzxState,
    br: &mut BitReader<'_>,
    lens: &mut [u8],
    first: usize,
    last: usize,
) -> Result<(), i32> {
    for x in 0..LZX_PRETREE_NUM_ELEMENTS {
        state.pretree_len[x] = br.read_bits(4)? as u8;
    }
    make_decode_table(
        LZX_PRETREE_MAXSYMBOLS,
        LZX_PRETREE_TABLEBITS,
        &state.pretree_len,
        &mut state.pretree_table,
    )?;

    let mut x = first;
    while x < last {
        let z = read_huffsym(
            br,
            LZX_PRETREE_TABLEBITS,
            LZX_PRETREE_MAXSYMBOLS,
            &state.pretree_len,
            &state.pretree_table,
        )? as i32;
        if z == 17 {
            let mut y = br.read_bits(4)? as usize + 4;
            while y > 0 && x < last {
                lens[x] = 0;
                x += 1;
                y -= 1;
            }
        } else if z == 18 {
            let mut y = br.read_bits(5)? as usize + 20;
            while y > 0 && x < last {
                lens[x] = 0;
                x += 1;
                y -= 1;
            }
        } else if z == 19 {
            let mut y = br.read_bits(1)? as usize + 4;
            let z2 = read_huffsym(
                br,
                LZX_PRETREE_TABLEBITS,
                LZX_PRETREE_MAXSYMBOLS,
                &state.pretree_len,
                &state.pretree_table,
            )? as i32;
            let mut v = lens[x] as i32 - z2;
            if v < 0 {
                v += 17;
            }
            while y > 0 && x < last {
                lens[x] = v as u8;
                x += 1;
                y -= 1;
            }
        } else {
            let mut v = lens[x] as i32 - z;
            if v < 0 {
                v += 17;
            }
            lens[x] = v as u8;
            x += 1;
        }
    }

    Ok(())
}

fn copy_match(window: &mut [u8], window_size: usize, window_posn: &mut usize, match_offset: usize, mut match_len: usize) {
    while match_len > 0 {
        let dst = *window_posn;
        let src = (window_size + dst - match_offset) & (window_size - 1);
        window[dst] = window[src];
        *window_posn = (*window_posn + 1) & (window_size - 1);
        match_len -= 1;
    }
}

impl LzxState {
    pub fn new(window_bits: u8) -> Result<Self, String> {
        if !(15..=21).contains(&window_bits) {
            return Err(format!("unsupported LZX window bits: {window_bits}"));
        }

        let window_size = 1usize << window_bits;
        let posn_slots = match window_bits {
            20 => 42,
            21 => 50,
            _ => (window_bits as usize) << 1,
        };
        let main_elements = LZX_NUM_CHARS + (posn_slots << 3);

        Ok(Self {
            window_bits,
            window_size,
            window_posn: 0,
            r0: 1,
            r1: 1,
            r2: 1,
            main_elements,
            header_read: false,
            block_type: LZX_BLOCKTYPE_INVALID,
            block_length: 0,
            block_remaining: 0,
            frames_read: 0,
            intel_filesize: 0,
            intel_curpos: 0,
            intel_started: false,
            window: vec![0; window_size],
            pretree_table: vec![0; (1 << LZX_PRETREE_TABLEBITS) + (LZX_PRETREE_MAXSYMBOLS << 1)],
            pretree_len: vec![0; LZX_PRETREE_MAXSYMBOLS + LZX_LENTABLE_SAFETY],
            maintree_table: vec![0; (1 << LZX_MAINTREE_TABLEBITS) + (LZX_MAINTREE_MAXSYMBOLS << 1)],
            maintree_len: vec![0; LZX_MAINTREE_MAXSYMBOLS + LZX_LENTABLE_SAFETY],
            length_table: vec![0; (1 << LZX_LENGTH_TABLEBITS) + (LZX_LENGTH_MAXSYMBOLS << 1)],
            length_len: vec![0; LZX_LENGTH_MAXSYMBOLS + LZX_LENTABLE_SAFETY],
            aligned_table: vec![0; (1 << LZX_ALIGNED_TABLEBITS) + (LZX_ALIGNED_MAXSYMBOLS << 1)],
            aligned_len: vec![0; LZX_ALIGNED_MAXSYMBOLS + LZX_LENTABLE_SAFETY],
        })
    }

    pub fn reset(&mut self) {
        self.r0 = 1;
        self.r1 = 1;
        self.r2 = 1;
        self.header_read = false;
        self.frames_read = 0;
        self.block_remaining = 0;
        self.block_type = LZX_BLOCKTYPE_INVALID;
        self.intel_curpos = 0;
        self.intel_started = false;
        self.window_posn = 0;

        for v in &mut self.maintree_len {
            *v = 0;
        }
        for v in &mut self.length_len {
            *v = 0;
        }
    }
}

pub fn decompress_block(
    state: &mut LzxState,
    input: &[u8],
    expected_out_len: usize,
) -> Result<Vec<u8>, String> {
    if expected_out_len == 0 {
        return Ok(Vec::new());
    }

    let mut br = BitReader::new(input);

    let mut window_posn = state.window_posn;
    let window_size = state.window_size;
    let mut r0 = state.r0;
    let mut r1 = state.r1;
    let mut r2 = state.r2;

    if !state.header_read {
        let mut i = 0u32;
        let mut j = 0u32;
        let k = br.read_bits(1).map_err(|e| format!("header read bit error: {e}"))?;
        if k != 0 {
            i = br.read_bits(16).map_err(|e| format!("header read i error: {e}"))?;
            j = br.read_bits(16).map_err(|e| format!("header read j error: {e}"))?;
        }
        state.intel_filesize = ((i << 16) | j) as i32;
        state.header_read = true;
    }

    let mut togo = expected_out_len;
    while togo > 0 {
        if state.block_remaining == 0 {
            if state.block_type == LZX_BLOCKTYPE_UNCOMPRESSED {
                if (state.block_length & 1) != 0 {
                    br.pos = br.pos.saturating_add(1);
                }
                br.align_reset();
            }

            state.block_type = br.read_bits(3).map_err(|e| format!("block type read error: {e}"))? as u16;
            let i = br.read_bits(16).map_err(|e| format!("block i read error: {e}"))?;
            let j = br.read_bits(8).map_err(|e| format!("block j read error: {e}"))?;
            state.block_remaining = ((i << 8) | j) as usize;
            state.block_length = state.block_remaining;

            match state.block_type {
                LZX_BLOCKTYPE_ALIGNED => {
                    for k in 0..LZX_ALIGNED_NUM_ELEMENTS {
                        state.aligned_len[k] = br.read_bits(3).map_err(|e| format!("aligned len read error: {e}"))? as u8;
                    }
                    make_decode_table(
                        LZX_ALIGNED_MAXSYMBOLS,
                        LZX_ALIGNED_TABLEBITS,
                        &state.aligned_len,
                        &mut state.aligned_table,
                    )
                    .map_err(|e| format!("aligned table build error: {e}"))?;

                    let mut left = state.maintree_len[0..256].to_vec();
                    lzx_read_lens(state, &mut br, &mut left, 0, 256)
                        .map_err(|e| format!("maintree lens(0,256) error: {e}"))?;
                    state.maintree_len[0..256].copy_from_slice(&left);

                    let main_elements = state.main_elements;
                    let mut right = state.maintree_len[256..main_elements].to_vec();
                    lzx_read_lens(state, &mut br, &mut right, 0, main_elements - 256)
                        .map_err(|e| format!("maintree lens(256,me) error: {e}"))?;
                    state.maintree_len[256..main_elements].copy_from_slice(&right);

                    make_decode_table(
                        LZX_MAINTREE_MAXSYMBOLS,
                        LZX_MAINTREE_TABLEBITS,
                        &state.maintree_len,
                        &mut state.maintree_table,
                    )
                    .map_err(|e| format!("maintree table build error: {e}"))?;
                    if state.maintree_len[0xE8] != 0 {
                        state.intel_started = true;
                    }

                    let mut llen = state.length_len[0..LZX_NUM_SECONDARY_LENGTHS].to_vec();
                    lzx_read_lens(state, &mut br, &mut llen, 0, LZX_NUM_SECONDARY_LENGTHS)
                        .map_err(|e| format!("length lens error: {e}"))?;
                    state.length_len[0..LZX_NUM_SECONDARY_LENGTHS].copy_from_slice(&llen);

                    make_decode_table(
                        LZX_LENGTH_MAXSYMBOLS,
                        LZX_LENGTH_TABLEBITS,
                        &state.length_len,
                        &mut state.length_table,
                    )
                    .map_err(|e| format!("length table build error: {e}"))?;
                }
                LZX_BLOCKTYPE_VERBATIM => {
                    let mut left = state.maintree_len[0..256].to_vec();
                    lzx_read_lens(state, &mut br, &mut left, 0, 256)
                        .map_err(|e| format!("maintree lens(0,256) error: {e}"))?;
                    state.maintree_len[0..256].copy_from_slice(&left);

                    let main_elements = state.main_elements;
                    let mut right = state.maintree_len[256..main_elements].to_vec();
                    lzx_read_lens(state, &mut br, &mut right, 0, main_elements - 256)
                        .map_err(|e| format!("maintree lens(256,me) error: {e}"))?;
                    state.maintree_len[256..main_elements].copy_from_slice(&right);

                    make_decode_table(
                        LZX_MAINTREE_MAXSYMBOLS,
                        LZX_MAINTREE_TABLEBITS,
                        &state.maintree_len,
                        &mut state.maintree_table,
                    )
                    .map_err(|e| format!("maintree table build error: {e}"))?;
                    if state.maintree_len[0xE8] != 0 {
                        state.intel_started = true;
                    }

                    let mut llen = state.length_len[0..LZX_NUM_SECONDARY_LENGTHS].to_vec();
                    lzx_read_lens(state, &mut br, &mut llen, 0, LZX_NUM_SECONDARY_LENGTHS)
                        .map_err(|e| format!("length lens error: {e}"))?;
                    state.length_len[0..LZX_NUM_SECONDARY_LENGTHS].copy_from_slice(&llen);

                    make_decode_table(
                        LZX_LENGTH_MAXSYMBOLS,
                        LZX_LENGTH_TABLEBITS,
                        &state.length_len,
                        &mut state.length_table,
                    )
                    .map_err(|e| format!("length table build error: {e}"))?;
                }
                LZX_BLOCKTYPE_UNCOMPRESSED => {
                    state.intel_started = true;
                    br.ensure_bits(16).map_err(|e| format!("uncompressed ensure bits error: {e}"))?;
                    if br.bitsleft > 16 {
                        if br.pos < 2 {
                            return Err("uncompressed align underflow".to_string());
                        }
                        br.pos -= 2;
                    }

                    let read_u32 = |src: &[u8], p: usize| -> Result<u32, String> {
                        let b0 = *src.get(p).ok_or_else(|| "r0 read b0 out of range".to_string())?;
                        let b1 = *src.get(p + 1).ok_or_else(|| "r0 read b1 out of range".to_string())?;
                        let b2 = *src.get(p + 2).ok_or_else(|| "r0 read b2 out of range".to_string())?;
                        let b3 = *src.get(p + 3).ok_or_else(|| "r0 read b3 out of range".to_string())?;
                        Ok((b0 as u32) | ((b1 as u32) << 8) | ((b2 as u32) << 16) | ((b3 as u32) << 24))
                    };
                    r0 = read_u32(br.input, br.pos)?;
                    br.pos += 4;
                    r1 = read_u32(br.input, br.pos)?;
                    br.pos += 4;
                    r2 = read_u32(br.input, br.pos)?;
                    br.pos += 4;
                }
                _ => return Err("illegal block type".to_string()),
            }
        }

        while state.block_remaining > 0 && togo > 0 {
            let mut this_run = state.block_remaining;
            if this_run > togo {
                this_run = togo;
            }
            togo -= this_run;
            state.block_remaining -= this_run;

            window_posn &= window_size - 1;
            if window_posn + this_run > window_size {
                return Err("dataformat: run crosses window boundary".to_string());
            }

            match state.block_type {
                LZX_BLOCKTYPE_VERBATIM | LZX_BLOCKTYPE_ALIGNED => {
                    while this_run > 0 {
                        let mut main_element = read_huffsym(
                            &mut br,
                            LZX_MAINTREE_TABLEBITS,
                            LZX_MAINTREE_MAXSYMBOLS,
                            &state.maintree_len,
                            &state.maintree_table,
                        )
                        .map_err(|e| format!("maintree symbol read error: {e}"))?;

                        if main_element < LZX_NUM_CHARS {
                            state.window[window_posn] = main_element as u8;
                            window_posn = (window_posn + 1) & (window_size - 1);
                            this_run -= 1;
                        } else {
                            main_element -= LZX_NUM_CHARS;
                            let mut match_length = main_element & LZX_NUM_PRIMARY_LENGTHS;
                            if match_length == LZX_NUM_PRIMARY_LENGTHS {
                                let length_footer = read_huffsym(
                                    &mut br,
                                    LZX_LENGTH_TABLEBITS,
                                    LZX_LENGTH_MAXSYMBOLS,
                                    &state.length_len,
                                    &state.length_table,
                                )
                                .map_err(|e| format!("length symbol read error: {e}"))?;
                                match_length += length_footer;
                            }
                            match_length += LZX_MIN_MATCH;

                            let mut match_offset = (main_element >> 3) as u32;
                            if match_offset > 2 {
                                if state.block_type == LZX_BLOCKTYPE_VERBATIM {
                                    if match_offset != 3 {
                                        let extra = EXTRA_BITS[match_offset as usize] as u32;
                                        let verbatim_bits = br
                                            .read_bits(extra)
                                            .map_err(|e| format!("verbatim extra bits read error: {e}"))?;
                                        match_offset = POSITION_BASE[match_offset as usize] - 2 + verbatim_bits;
                                    } else {
                                        match_offset = 1;
                                    }
                                } else {
                                    let mut extra = EXTRA_BITS[match_offset as usize] as i32;
                                    match_offset = POSITION_BASE[match_offset as usize] - 2;
                                    if extra > 3 {
                                        extra -= 3;
                                        let verbatim_bits = br
                                            .read_bits(extra as u32)
                                            .map_err(|e| format!("aligned extra verbatim bits read error: {e}"))?;
                                        match_offset += verbatim_bits << 3;
                                        let aligned_bits = read_huffsym(
                                            &mut br,
                                            LZX_ALIGNED_TABLEBITS,
                                            LZX_ALIGNED_MAXSYMBOLS,
                                            &state.aligned_len,
                                            &state.aligned_table,
                                        )
                                        .map_err(|e| format!("aligned symbol read error: {e}"))?
                                            as u32;
                                        match_offset += aligned_bits;
                                    } else if extra == 3 {
                                        let aligned_bits = read_huffsym(
                                            &mut br,
                                            LZX_ALIGNED_TABLEBITS,
                                            LZX_ALIGNED_MAXSYMBOLS,
                                            &state.aligned_len,
                                            &state.aligned_table,
                                        )
                                        .map_err(|e| format!("aligned symbol read error: {e}"))?
                                            as u32;
                                        match_offset += aligned_bits;
                                    } else if extra > 0 {
                                        let verbatim_bits = br
                                            .read_bits(extra as u32)
                                            .map_err(|e| format!("aligned short verbatim bits read error: {e}"))?;
                                        match_offset += verbatim_bits;
                                    } else {
                                        match_offset = 1;
                                    }
                                }
                                r2 = r1;
                                r1 = r0;
                                r0 = match_offset;
                            } else if match_offset == 0 {
                                match_offset = r0;
                            } else if match_offset == 1 {
                                match_offset = r1;
                                r1 = r0;
                                r0 = match_offset;
                            } else {
                                match_offset = r2;
                                r2 = r0;
                                r0 = match_offset;
                            }

                            if match_offset == 0 || (match_offset as usize) > window_size {
                                return Err("illegal match offset".to_string());
                            }

                            copy_match(
                                &mut state.window,
                                window_size,
                                &mut window_posn,
                                match_offset as usize,
                                match_length,
                            );

                            if this_run < match_length {
                                return Err("illegal match length".to_string());
                            }
                            this_run -= match_length;
                        }
                    }
                }
                LZX_BLOCKTYPE_UNCOMPRESSED => {
                    if br.pos + this_run > br.input.len() {
                        return Err("illegal uncompressed read past input".to_string());
                    }
                    let src = &br.input[br.pos..br.pos + this_run];
                    state.window[window_posn..window_posn + this_run].copy_from_slice(src);
                    br.pos += this_run;
                    window_posn = (window_posn + this_run) & (window_size - 1);
                }
                _ => return Err("illegal block type in decode".to_string()),
            }
        }
    }

    if togo != 0 {
        return Err("illegal data: output not fully produced".to_string());
    }

    let mut out = vec![0u8; expected_out_len];
    let start = if window_posn >= expected_out_len {
        window_posn - expected_out_len
    } else {
        window_size + window_posn - expected_out_len
    };
    for (i, b) in out.iter_mut().enumerate() {
        *b = state.window[(start + i) & (window_size - 1)];
    }

    state.window_posn = window_posn;
    state.r0 = r0;
    state.r1 = r1;
    state.r2 = r2;

    if state.intel_started && state.intel_filesize != 0 {
        let mut curpos = state.intel_curpos;
        let filesize = state.intel_filesize;
        if state.frames_read < 32_768 && out.len() > 10 {
            let last = out.len() - 10;
            for i in 0..last {
                if out[i] == 0xE8 {
                    let abs_off =
                        i32::from_le_bytes([out[i + 1], out[i + 2], out[i + 3], out[i + 4]]);
                    let rel_off = if abs_off >= -curpos && abs_off < filesize {
                        abs_off - curpos
                    } else {
                        abs_off + filesize
                    };
                    let b = rel_off.to_le_bytes();
                    out[i + 1] = b[0];
                    out[i + 2] = b[1];
                    out[i + 3] = b[2];
                    out[i + 4] = b[3];
                }
                curpos = curpos.saturating_add(1);
            }
        }
        state.intel_curpos = state.intel_curpos.saturating_add(out.len() as i32);
    }
    state.frames_read = state.frames_read.saturating_add(1);

    Ok(out)
}
