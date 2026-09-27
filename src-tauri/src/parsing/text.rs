//! Text/HTML utility helpers used by CHM parsing and runtime decoding.
use encoding_rs::EUC_KR;

/// Key HTML fragments borrowed from the decoded page.
pub(crate) struct HtmlFragments<'a> {
    pub(crate) title: Option<&'a str>,
    pub(crate) body_html: Option<&'a str>,
    pub(crate) first_paragraph_html: Option<&'a str>,
}

/// Decode EUC-KR bytes used by this dictionary dataset.
pub(crate) fn decode_euc_kr(bytes: &[u8]) -> String {
    let (s, _, _) = EUC_KR.decode(bytes);
    s.into_owned()
}

/// Decode a minimal set of HTML entities used in legacy pages.
pub(crate) fn decode_basic_html_entities(s: &str) -> String {
    let mut out = s.to_string();
    for (entity, replacement) in [
        ("&amp;", "&"),
        ("&lt;", "<"),
        ("&gt;", ">"),
        ("&quot;", "\""),
        ("&#39;", "'"),
    ] {
        if out.contains(entity) {
            out = out.replace(entity, replacement);
        }
    }
    out
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
    if !out.contains('&') {
        return out;
    }
    let mut out = decode_basic_html_entities(&out);
    if out.contains("&nbsp;") {
        out = out.replace("&nbsp;", " ");
    }
    if out.contains("&middot;") {
        out = out.replace("&middot;", "·");
    }
    out
}

/// Collapse consecutive whitespace to single spaces.
pub(crate) fn compact_ws(input: &str) -> String {
    let mut words = input.split_whitespace();
    let Some(first) = words.next() else {
        return String::new();
    };
    let mut out = String::with_capacity(input.len());
    out.push_str(first);
    for word in words {
        out.push(' ');
        out.push_str(word);
    }
    out
}

/// Strip tags and compact visible text, using one scan when there are no entities.
pub(crate) fn compact_html_text(input: &str) -> String {
    if input.contains('&') {
        return compact_ws(&strip_html_tags(input));
    }

    let mut out = String::with_capacity(input.len());
    let mut in_tag = false;
    let mut pending_space = false;
    for c in input.chars() {
        if c == '<' {
            in_tag = true;
            continue;
        }
        if c == '>' {
            in_tag = false;
            continue;
        }
        if in_tag {
            continue;
        }
        if c.is_whitespace() {
            pending_space = !out.is_empty();
        } else {
            if pending_space {
                out.push(' ');
                pending_space = false;
            }
            out.push(c);
        }
    }
    out
}

/// Extract filename stem from path-like string.
pub(crate) fn path_stem(path: &str) -> String {
    let base = path.rsplit('/').next().unwrap_or(path);
    base.rsplit_once('.')
        .map_or(base, |(stem, _)| stem)
        .trim()
        .to_string()
}

fn first_tag_inner_from<'a>(
    lower: &str,
    text: &'a str,
    open: &str,
    close: &str,
    start: usize,
    end: usize,
) -> Option<&'a str> {
    let region = &lower[start..end];
    let start_rel = region.find(open)?;
    let tag_start = start + start_rel;
    let open_end = lower[tag_start..end].find('>')? + tag_start;
    let close_rel = lower[open_end + 1..end].find(close)?;
    let close_start = open_end + 1 + close_rel;
    Some(&text[open_end + 1..close_start])
}

/// Extract title/body/first-paragraph HTML with shared lowercase scan.
pub(crate) fn extract_html_fragments(text: &str) -> HtmlFragments<'_> {
    let lower = text.to_ascii_lowercase();
    let title = first_tag_inner_from(&lower, text, "<title", "</title>", 0, lower.len());

    let body_range = if let Some(body_start_rel) = lower.find("<body") {
        if let Some(tag_end_rel) = lower[body_start_rel..].find('>') {
            let body_open_end = body_start_rel + tag_end_rel;
            if let Some(close_rel) = lower[body_open_end + 1..].find("</body>") {
                let body_close = body_open_end + 1 + close_rel;
                Some((body_open_end + 1, body_close))
            } else {
                None
            }
        } else {
            None
        }
    } else {
        None
    };

    let (paragraph_start, paragraph_end) = body_range.unwrap_or((0, lower.len()));
    let first_paragraph_html =
        first_tag_inner_from(&lower, text, "<p", "</p>", paragraph_start, paragraph_end);
    let body_html = body_range.map(|(start, end)| &text[start..end]);

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
    extract_first_bold_text_from_paragraph(&p_html)
}

/// Extract first `<b>` text from an already extracted first paragraph.
pub(crate) fn extract_first_bold_text_from_paragraph(p_html: &str) -> Option<String> {
    let p_lower = p_html.to_ascii_lowercase();
    let b_start = p_lower.find("<b")?;
    let tag_end = p_lower[b_start..].find('>')? + b_start;
    let b_end = p_lower[tag_end + 1..].find("</b>")? + tag_end + 1;
    Some(compact_html_text(&p_html[tag_end + 1..b_end]))
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

#[cfg(test)]
mod tests {
    use super::{
        compact_html_text, compact_ws, extract_first_bold_text,
        extract_first_bold_text_from_paragraph, extract_html_fragments,
    };

    // Keep the former pipeline here to check the optimized path against its exact semantics.
    fn legacy_compact_html_text(input: &str) -> String {
        let mut stripped = String::with_capacity(input.len());
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
                stripped.push(c);
            }
        }
        stripped = stripped
            .replace("&amp;", "&")
            .replace("&lt;", "<")
            .replace("&gt;", ">")
            .replace("&quot;", "\"")
            .replace("&#39;", "'")
            .replace("&nbsp;", " ")
            .replace("&middot;", "·");
        stripped.split_whitespace().collect::<Vec<_>>().join(" ")
    }

    #[test]
    fn compact_html_text_preserves_legacy_output() {
        let samples = [
            "",
            " \t\n ",
            "<P>  한글\u{2003}  words </P>",
            "<b>x</b>y",
            "x>y<broken",
            "<x a=' > '>foo",
            "one\u{00a0}two\u{200b}three",
            "<p>&amp;nbsp; &amp;amp;nbsp; &middot; &lt; &gt; &quot; &#39;</p>",
            "<p title='a & b'>content</p>",
            "prefix<&suffix>post",
        ];
        for sample in samples {
            assert_eq!(
                compact_html_text(sample),
                legacy_compact_html_text(sample),
                "{sample:?}"
            );
        }
        assert_eq!(compact_html_text("a &amp;nbsp; b"), "a b");
        assert_eq!(compact_html_text("a &amp;amp;nbsp; b"), "a &amp;nbsp; b");
        assert_eq!(compact_ws(" \u{2003}가\u{00a0} 나 \t"), "가 나");
    }

    #[test]
    fn html_fragments_and_bold_keep_body_scope() {
        let html = "<TITLE>제목</TITLE><p>outside</p><BODY class='x'><P>가 <B> 굵은&nbsp; 말 </B></P><p>later</p></BODY>";
        let fragments = extract_html_fragments(html);
        assert_eq!(fragments.title, Some("제목"));
        assert_eq!(
            fragments.first_paragraph_html,
            Some("가 <B> 굵은&nbsp; 말 </B>")
        );
        assert_eq!(
            fragments.body_html,
            Some("<P>가 <B> 굵은&nbsp; 말 </B></P><p>later</p>")
        );
        let title_start = html.find("제목").unwrap();
        let body_start = html.find("<P>가").unwrap();
        let paragraph_start = html.find("가 <B>").unwrap();
        assert_eq!(
            fragments.title.unwrap().as_ptr(),
            html[title_start..].as_ptr()
        );
        assert_eq!(
            fragments.body_html.unwrap().as_ptr(),
            html[body_start..].as_ptr()
        );
        assert_eq!(
            fragments.first_paragraph_html.unwrap().as_ptr(),
            html[paragraph_start..].as_ptr(),
        );
        assert_eq!(extract_first_bold_text(html).as_deref(), Some("굵은 말"));
        assert_eq!(
            extract_first_bold_text_from_paragraph(fragments.first_paragraph_html.unwrap())
                .as_deref(),
            Some("굵은 말"),
        );
        assert_eq!(
            extract_html_fragments("<p>fallback</p>").first_paragraph_html,
            Some("fallback")
        );
    }
}
