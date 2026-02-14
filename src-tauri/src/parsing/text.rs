//! Text/HTML utility helpers used by CHM parsing and runtime decoding.
use encoding_rs::EUC_KR;

/// Decode EUC-KR bytes used by this dictionary dataset.
pub(crate) fn decode_euc_kr(bytes: &[u8]) -> String {
    let (s, _, _) = EUC_KR.decode(bytes);
    s.into_owned()
}

/// Decode a minimal set of HTML entities used in legacy pages.
pub(crate) fn decode_basic_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

/// Strip HTML tags and keep only visible text.
pub(crate) fn strip_html_tags(input: &str) -> String {
    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    for c in input.chars() {
        if c == '<' {
            in_tag = true;
            continue;
        }
        if c == '>' {
            in_tag = false;
            continue;
        }
        if !in_tag {
            out.push(c);
        }
    }
    decode_basic_html_entities(&out)
        .replace("&nbsp;", " ")
        .replace("&middot;", "·")
}

/// Collapse consecutive whitespace to single spaces.
pub(crate) fn compact_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

/// Extract filename stem from path-like string.
pub(crate) fn path_stem(path: &str) -> String {
    let base = path.rsplit('/').next().unwrap_or(path);
    base.rsplit_once('.')
        .map_or(base, |(stem, _)| stem)
        .trim()
        .to_string()
}

/// Extract all raw values inside simple `<tag>...</tag>` pairs.
pub(crate) fn find_all_tag_values(text: &str, tag: &str) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut offset = 0usize;

    while let Some(start_rel) = lower[offset..].find(&open) {
        let value_start = offset + start_rel + open.len();
        if let Some(end_rel) = lower[value_start..].find(&close) {
            let value_end = value_start + end_rel;
            out.push(text[value_start..value_end].trim().to_string());
            offset = value_end + close.len();
        } else {
            break;
        }
    }
    out
}

/// Return first paragraph inner HTML if present.
pub(crate) fn first_paragraph_html(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let p_start = lower.find("<p")?;
    let tag_end = lower[p_start..].find('>')? + p_start;
    let p_end = lower[tag_end + 1..].find("</p>")? + tag_end + 1;
    Some(text[tag_end + 1..p_end].to_string())
}

/// Return body inner HTML if present.
pub(crate) fn body_html(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let b_start = lower.find("<body")?;
    let tag_end = lower[b_start..].find('>')? + b_start;
    let b_end = lower[tag_end + 1..].find("</body>")? + tag_end + 1;
    Some(text[tag_end + 1..b_end].to_string())
}

/// Extract first `<b>` text from first paragraph.
pub(crate) fn extract_first_bold_text(text: &str) -> Option<String> {
    let p_html = first_paragraph_html(text)?;
    let p_lower = p_html.to_ascii_lowercase();
    let b_start = p_lower.find("<b")?;
    let tag_end = p_lower[b_start..].find('>')? + b_start;
    let b_end = p_lower[tag_end + 1..].find("</b>")? + tag_end + 1;
    Some(compact_ws(&strip_html_tags(&p_html[tag_end + 1..b_end])))
}

/// Extract quoted attribute value from a tag snippet.
pub(crate) fn extract_attr_value(tag: &str, attr_name: &str) -> Option<String> {
    let bytes = tag.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        let key_start = i;
        while i < bytes.len()
            && (bytes[i].is_ascii_alphanumeric() || bytes[i] == b'_' || bytes[i] == b'-')
        {
            i += 1;
        }
        if key_start == i {
            i += 1;
            continue;
        }
        let key = &tag[key_start..i];
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() || bytes[i] != b'=' {
            continue;
        }
        i += 1;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let quote = bytes[i];
        if quote != b'"' && quote != b'\'' {
            continue;
        }
        i += 1;
        let val_start = i;
        while i < bytes.len() && bytes[i] != quote {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        if key.eq_ignore_ascii_case(attr_name) {
            return Some(tag[val_start..i].to_string());
        }
        i += 1;
    }
    None
}
