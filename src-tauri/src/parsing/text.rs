use encoding_rs::EUC_KR;

pub(crate) fn decode_euc_kr(bytes: &[u8]) -> String {
    let (s, _, _) = EUC_KR.decode(bytes);
    s.into_owned()
}

pub(crate) fn decode_basic_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

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

pub(crate) fn compact_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

pub(crate) fn path_stem(path: &str) -> String {
    let base = path.rsplit('/').next().unwrap_or(path);
    base.rsplit_once('.')
        .map_or(base, |(stem, _)| stem)
        .trim()
        .to_string()
}

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

pub(crate) fn first_paragraph_html(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let p_start = lower.find("<p")?;
    let tag_end = lower[p_start..].find('>')? + p_start;
    let p_end = lower[tag_end + 1..].find("</p>")? + tag_end + 1;
    Some(text[tag_end + 1..p_end].to_string())
}

pub(crate) fn body_html(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let b_start = lower.find("<body")?;
    let tag_end = lower[b_start..].find('>')? + b_start;
    let b_end = lower[tag_end + 1..].find("</body>")? + tag_end + 1;
    Some(text[tag_end + 1..b_end].to_string())
}

pub(crate) fn extract_first_bold_text(text: &str) -> Option<String> {
    let p_html = first_paragraph_html(text)?;
    let p_lower = p_html.to_ascii_lowercase();
    let b_start = p_lower.find("<b")?;
    let tag_end = p_lower[b_start..].find('>')? + b_start;
    let b_end = p_lower[tag_end + 1..].find("</b>")? + tag_end + 1;
    Some(compact_ws(&strip_html_tags(&p_html[tag_end + 1..b_end])))
}

pub(crate) fn extract_attr_value(tag: &str, attr_name: &str) -> Option<String> {
    let lower = tag.to_ascii_lowercase();
    let key = format!("{attr_name}=");
    let pos = lower.find(&key)?;
    let rest = &tag[pos + key.len()..];
    let mut chars = rest.chars();
    let quote = chars.next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }

    let mut value = String::new();
    for c in chars {
        if c == quote {
            break;
        }
        value.push(c);
    }
    Some(value)
}
