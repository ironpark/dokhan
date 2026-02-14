//! Text/HTML utility helpers used by CHM parsing and runtime decoding.
use encoding_rs::EUC_KR;

/// Key HTML fragments extracted in one pass-friendly flow.
pub(crate) struct HtmlFragments {
    pub(crate) title: Option<String>,
    pub(crate) body_html: Option<String>,
    pub(crate) first_paragraph_html: Option<String>,
}

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

fn first_tag_inner_from(lower: &str, text: &str, tag: &str, start: usize, end: usize) -> Option<String> {
    let open = format!("<{tag}");
    let close = format!("</{tag}>");
    let region = &lower[start..end];
    let start_rel = region.find(&open)?;
    let tag_start = start + start_rel;
    let open_end = lower[tag_start..end].find('>')? + tag_start;
    let close_rel = lower[open_end + 1..end].find(&close)?;
    let close_start = open_end + 1 + close_rel;
    Some(text[open_end + 1..close_start].to_string())
}

/// Extract title/body/first-paragraph HTML with shared lowercase scan.
pub(crate) fn extract_html_fragments(text: &str) -> HtmlFragments {
    let lower = text.to_ascii_lowercase();
    let title = first_tag_inner_from(&lower, text, "title", 0, lower.len());

    let body_html = if let Some(body_start_rel) = lower.find("<body") {
        if let Some(tag_end_rel) = lower[body_start_rel..].find('>') {
            let body_open_end = body_start_rel + tag_end_rel;
            if let Some(close_rel) = lower[body_open_end + 1..].find("</body>") {
                let body_close = body_open_end + 1 + close_rel;
                Some(text[body_open_end + 1..body_close].to_string())
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let first_paragraph_html = if let Some(ref body) = body_html {
        let body_lower = body.to_ascii_lowercase();
        first_tag_inner_from(&body_lower, body, "p", 0, body_lower.len())
    } else {
        first_tag_inner_from(&lower, text, "p", 0, lower.len())
    };

    HtmlFragments {
        title,
        body_html,
        first_paragraph_html,
    }
}

/// Sanitize HTML fragment to prevent script/event-handler execution in webview.
pub(crate) fn sanitize_html_fragment(fragment: &str) -> String {
    ammonia::Builder::default().clean(fragment).to_string()
}

/// Extract first `<b>` text from first paragraph.
pub(crate) fn extract_first_bold_text(text: &str) -> Option<String> {
    let p_html = extract_html_fragments(text).first_paragraph_html?;
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
