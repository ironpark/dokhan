use std::collections::BTreeSet;

use crate::chm;
use crate::app::model::{ContentItem, EntryDetail};
use crate::parsing::text::{compact_ws, decode_euc_kr, extract_attr_value, path_stem};
use crate::runtime::link_media::{normalize_path, parse_internal_ref};

fn extract_ascii_runs(bytes: &[u8], min_len: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;

    for (index, byte) in bytes.iter().enumerate() {
        if byte.is_ascii_graphic() || *byte == b' ' {
            if start.is_none() {
                start = Some(index);
            }
        } else if let Some(s) = start.take() {
            if index - s >= min_len {
                out.push(String::from_utf8_lossy(&bytes[s..index]).to_string());
            }
        }
    }

    if let Some(s) = start {
        if bytes.len() - s >= min_len {
            out.push(String::from_utf8_lossy(&bytes[s..]).to_string());
        }
    }
    out
}

fn trim_path_noise(s: &str) -> String {
    s.trim_matches(|c: char| {
        c.is_control()
            || c.is_whitespace()
            || matches!(c, '\'' | '"' | ',' | ';' | '(' | ')' | '[' | ']')
    })
    .to_string()
}

pub(crate) fn extract_chm_paths(
    chm_bytes: &[u8],
    sample_limit: usize,
) -> (usize, Vec<String>, Vec<String>, Vec<String>) {
    let runs = extract_ascii_runs(chm_bytes, 6);
    let mut html = BTreeSet::new();
    let mut hhk = BTreeSet::new();
    let mut hhc = BTreeSet::new();

    for raw in runs {
        let path = trim_path_noise(&raw);
        let lower = path.to_ascii_lowercase();

        if (lower.ends_with(".htm") || lower.ends_with(".html")) && path.contains('/') {
            html.insert(path);
            continue;
        }
        if lower.ends_with(".hhk") && path.contains('/') {
            hhk.insert(path);
            continue;
        }
        if lower.ends_with(".hhc") && path.contains('/') {
            hhc.insert(path);
        }
    }

    let html_count = html.len();
    let sample_html_paths = html.into_iter().take(sample_limit).collect::<Vec<_>>();
    let hhk_files = hhk.into_iter().take(sample_limit).collect::<Vec<_>>();
    let hhc_files = hhc.into_iter().take(sample_limit).collect::<Vec<_>>();
    (html_count, sample_html_paths, hhk_files, hhc_files)
}

pub(crate) fn parse_master_hhc_text(text: &str) -> Vec<ContentItem> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut offset = 0usize;

    while let Some(start_rel) = lower[offset..].find("<object type=\"text/sitemap\"") {
        let block_start = offset + start_rel;
        let Some(end_rel) = lower[block_start..].find("</object>") else {
            break;
        };
        let block_end = block_start + end_rel + "</object>".len();
        let block = &text[block_start..block_end];

        let mut name = String::new();
        let mut local = String::new();
        let block_lower = block.to_ascii_lowercase();
        let mut param_offset = 0usize;
        while let Some(param_rel) = block_lower[param_offset..].find("<param") {
            let param_start = param_offset + param_rel;
            let Some(param_end_rel) = block[param_start..].find('>') else {
                break;
            };
            let param_end = param_start + param_end_rel + 1;
            let tag = &block[param_start..param_end];
            let tag_lower = tag.to_ascii_lowercase();

            if tag_lower.contains("name=\"name\"") || tag_lower.contains("name='name'") {
                if let Some(value) = extract_attr_value(tag, "value") {
                    name = compact_ws(value.trim());
                }
            }
            if tag_lower.contains("name=\"local\"") || tag_lower.contains("name='local'") {
                if let Some(value) = extract_attr_value(tag, "value") {
                    local = compact_ws(value.trim());
                }
            }
            param_offset = param_end;
        }

        if !name.is_empty() && !local.is_empty() {
            out.push(ContentItem { title: name, local });
        }
        offset = block_end;
    }
    out
}

fn extract_all_headwords_from_chm_bytes(chm_bytes: &[u8]) -> Vec<String> {
    let html_paths = if let Ok(chm) = chm::ChmArchive::open(chm_bytes.to_vec()) {
        chm.entries()
            .iter()
            .filter(|entry| entry.path.to_ascii_lowercase().ends_with(".htm"))
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>()
    } else {
        let (_, html_paths, _, _) = extract_chm_paths(chm_bytes, usize::MAX);
        html_paths
    };

    let mut words = BTreeSet::new();
    for path in html_paths {
        let normalized = path.trim_start_matches('/');
        let filename = normalized.rsplit('/').next().unwrap_or(normalized);
        let base = filename
            .rsplit_once('.')
            .map_or(filename, |(stem, _)| stem)
            .trim();
        if base.is_empty() {
            continue;
        }
        let lower = base.to_ascii_lowercase();
        if matches!(lower.as_str(), "master" | "index" | "version_information" | "dictionary" | "a") {
            continue;
        }
        words.insert(base.to_string());
    }
    words.into_iter().collect::<Vec<_>>()
}

fn parse_hhk_entries_from_text(text: &str, default_source_path: &str) -> Vec<EntryDetail> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut offset = 0usize;

    while let Some(start_rel) = lower[offset..].find("<object type=\"text/sitemap\"") {
        let block_start = offset + start_rel;
        let Some(end_rel) = lower[block_start..].find("</object>") else {
            break;
        };
        let block_end = block_start + end_rel + "</object>".len();
        let block = &text[block_start..block_end];

        let mut name = String::new();
        let mut local = String::new();
        let block_lower = block.to_ascii_lowercase();
        let mut param_offset = 0usize;
        while let Some(param_rel) = block_lower[param_offset..].find("<param") {
            let param_start = param_offset + param_rel;
            let Some(param_end_rel) = block[param_start..].find('>') else {
                break;
            };
            let param_end = param_start + param_end_rel + 1;
            let tag = &block[param_start..param_end];
            let tag_lower = tag.to_ascii_lowercase();
            if tag_lower.contains("name=\"name\"") || tag_lower.contains("name='name'") {
                if let Some(value) = extract_attr_value(tag, "value") {
                    name = compact_ws(value.trim());
                }
            }
            if tag_lower.contains("name=\"local\"") || tag_lower.contains("name='local'") {
                if let Some(value) = extract_attr_value(tag, "value") {
                    local = compact_ws(value.trim());
                }
            }
            param_offset = param_end;
        }

        if !name.is_empty() {
            let (source_override, local_path, _) =
                parse_internal_ref(&local).unwrap_or((None, local.clone(), false));
            let source_path = source_override.unwrap_or_else(|| default_source_path.to_ascii_lowercase());
            let target_local = normalize_path(&local_path);
            let mut aliases = vec![name.clone()];
            let target_stem = compact_ws(&path_stem(&target_local));
            if !target_stem.is_empty() && !aliases.contains(&target_stem) {
                aliases.push(target_stem);
            }
            out.push(EntryDetail {
                id: 0,
                headword: name,
                aliases,
                source_path,
                target_local,
                definition_text: String::new(),
                definition_html: String::new(),
            });
        }
        offset = block_end;
    }
    out
}

pub(crate) fn extract_index_entries_from_chm_bytes(
    chm_file_name: &str,
    chm_bytes: &[u8],
) -> Vec<EntryDetail> {
    let mut out = Vec::new();
    let Ok(mut chm) = chm::ChmArchive::open(chm_bytes.to_vec()) else {
        return out;
    };

    let hhk_paths = chm
        .entries()
        .iter()
        .filter(|entry| entry.path.to_ascii_lowercase().ends_with(".hhk"))
        .map(|entry| entry.path.clone())
        .collect::<Vec<_>>();

    for hhk_path in hhk_paths {
        if let Ok(bytes) = chm.read_object(&hhk_path) {
            let text = decode_euc_kr(&bytes);
            out.append(&mut parse_hhk_entries_from_text(&text, chm_file_name));
        }
    }

    if out.is_empty() {
        for word in extract_all_headwords_from_chm_bytes(chm_bytes) {
            out.push(EntryDetail {
                id: 0,
                headword: word.clone(),
                aliases: vec![word],
                source_path: chm_file_name.to_ascii_lowercase(),
                target_local: String::new(),
                definition_text: String::new(),
                definition_html: String::new(),
            });
        }
    }
    out
}

#[cfg(test)]
pub(crate) fn extract_headwords_from_hhk_bytes(
    chm_bytes: &[u8],
    sample_limit: usize,
) -> (usize, Vec<String>, usize) {
    let text = String::from_utf8_lossy(chm_bytes);
    let text_lower = text.to_ascii_lowercase();
    let hhk_path_count = text_lower.matches(".hhk").count();

    let mut out = BTreeSet::new();
    let mut offset = 0usize;
    while let Some(start) = text_lower[offset..].find("<param") {
        let abs_start = offset + start;
        let search = &text[abs_start..];
        let Some(end_rel) = search.find('>') else {
            break;
        };
        if end_rel > 800 {
            offset = abs_start + 6;
            continue;
        }
        let tag = &search[..=end_rel];
        let tag_lower = tag.to_ascii_lowercase();
        let is_name_param = tag_lower.contains("name=\"name\"") || tag_lower.contains("name='name'");
        if is_name_param {
            if let Some(raw_value) = extract_attr_value(tag, "value") {
                let value = crate::parsing::text::decode_basic_html_entities(raw_value.trim());
                let lower = value.to_ascii_lowercase();
                if !value.is_empty()
                    && value.len() <= 120
                    && !lower.ends_with(".htm")
                    && !value.contains('/')
                {
                    out.insert(value);
                }
            }
        }
        offset = abs_start + end_rel + 1;
    }

    let count = out.len();
    let sample = out.into_iter().take(sample_limit).collect::<Vec<_>>();
    (count, sample, hhk_path_count)
}

#[cfg(test)]
mod tests {
    use super::{extract_ascii_runs, extract_chm_paths, extract_headwords_from_hhk_bytes};

    #[test]
    fn extract_ascii_runs_works() {
        let bytes = b"\x00/abc.htm\x00xyz\x00/master.hhk\x00";
        let runs = extract_ascii_runs(bytes, 4);
        assert!(runs.iter().any(|x| x.contains("/abc.htm")));
        assert!(runs.iter().any(|x| x.contains("/master.hhk")));
    }

    #[test]
    fn extract_chm_paths_finds_html_hhk_hhc() {
        let bytes = b"\x00/a.htm\x00/master.hhk\x00/master.hhc\x00";
        let (html_count, html, hhk, hhc) = extract_chm_paths(bytes, 10);
        assert_eq!(html_count, 1);
        assert_eq!(html, vec!["/a.htm".to_string()]);
        assert_eq!(hhk, vec!["/master.hhk".to_string()]);
        assert_eq!(hhc, vec!["/master.hhc".to_string()]);
    }

    #[test]
    fn extract_hhk_headwords_from_param_tags() {
        let sample = br#"
            <UL>
              <LI> <OBJECT type="text/sitemap">
                <param name="Name" value="Aal">
                <param name="Local" value="Aal.htm">
              </OBJECT>
              <LI> <OBJECT type="text/sitemap">
                <param name='Name' value='abbrechen'>
                <param name='Local' value='abbrechen.htm'>
              </OBJECT>
            </UL>
        "#;
        let (count, words, _hhk_paths) = extract_headwords_from_hhk_bytes(sample, 10);
        assert_eq!(count, 2);
        assert!(words.contains(&"Aal".to_string()));
        assert!(words.contains(&"abbrechen".to_string()));
    }
}
