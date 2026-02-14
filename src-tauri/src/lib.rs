use encoding_rs::EUC_KR;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use zip::ZipArchive;

mod chm;
mod commands;
mod link_media;
mod model;
mod runtime_zip;
mod runtime_state;
mod search_index;
pub use commands::run;
pub(crate) use model::*;
pub(crate) use link_media::{
    normalize_path,
    parse_internal_ref,
    read_chm_binary_object,
    read_debug_binary_object,
    resolve_link_target_impl,
    resolve_media_data_url_impl,
};
pub(crate) use search_index::{
    build_entry_search_keys,
    eq_search_key,
    get_index_entries_impl,
    normalize_search_key,
    search_entries_impl,
};
pub(crate) use runtime_zip::{
    hydrate_zip_entry_detail,
    parse_runtime_from_zip_with_progress,
    read_content_page_from_debug,
    read_content_page_from_zip,
    read_named_chm_from_zip,
};
pub(crate) use runtime_state::{
    build_runtime_index,
    get_content_page_impl,
    get_entry_detail_impl,
    get_master_build_status_impl,
    get_master_contents_impl,
    get_runtime,
    start_master_build_impl,
};

fn summarize_zip(zip_path: &Path) -> Result<DatasetSummary, String> {
    let file = File::open(zip_path).map_err(|e| format!("failed to open zip: {e}"))?;
    let mut archive = ZipArchive::new(file).map_err(|e| format!("invalid zip archive: {e}"))?;

    let mut total_files = 0usize;
    let mut total_dirs = 0usize;
    let mut uncompressed_bytes = 0u64;
    let mut compressed_bytes = 0u64;
    let mut names = Vec::with_capacity(archive.len());
    let mut ext_counts: BTreeMap<String, usize> = BTreeMap::new();

    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry #{i}: {e}"))?;
        let name = entry.name().to_string();

        if entry.is_dir() {
            total_dirs += 1;
        } else {
            total_files += 1;
            uncompressed_bytes += entry.size();
            compressed_bytes += entry.compressed_size();

            let ext = name
                .rsplit_once('.')
                .map(|(_, ext)| ext.to_ascii_lowercase())
                .unwrap_or_default();
            *ext_counts.entry(ext).or_insert(0) += 1;
        }
        names.push(name);
    }

    let name_set: std::collections::BTreeSet<_> = names.iter().cloned().collect();
    let missing_main_files_only = (1..=36)
        .map(|n| format!("merge{n:02}.chm"))
        .filter(|n| !name_set.contains(n))
        .collect::<Vec<_>>();

    let main_volume_coverage = build_main_volume_coverage(&names);

    let missing_main_volumes = main_volume_coverage
        .iter()
        .filter(|x| !x.covered)
        .map(|x| format!("merge{:02}.chm", x.volume))
        .collect::<Vec<_>>();

    let sample_chm_files = names
        .iter()
        .filter(|n| n.to_ascii_lowercase().ends_with(".chm"))
        .take(8)
        .cloned()
        .collect::<Vec<_>>();

    let extension_counts = ext_counts
        .into_iter()
        .map(|(extension, count)| FileTypeCount { extension, count })
        .collect::<Vec<_>>();

    Ok(DatasetSummary {
        zip_path: zip_path.to_string_lossy().to_string(),
        total_entries: archive.len(),
        total_files,
        total_dirs,
        chm_count: names
            .iter()
            .filter(|n| n.to_ascii_lowercase().ends_with(".chm"))
            .count(),
        lnk_count: names
            .iter()
            .filter(|n| n.to_ascii_lowercase().ends_with(".lnk"))
            .count(),
        txt_count: names
            .iter()
            .filter(|n| n.to_ascii_lowercase().ends_with(".txt"))
            .count(),
        uncompressed_bytes,
        compressed_bytes,
        compression_ratio: if uncompressed_bytes == 0 {
            0.0
        } else {
            compressed_bytes as f64 / uncompressed_bytes as f64
        },
        has_master_chm: name_set.contains("master.chm"),
        has_readme_txt: name_set.contains("readme.txt"),
        missing_main_volumes,
        missing_main_files_only,
        main_volume_coverage,
        extension_counts,
        sample_chm_files,
    })
}

fn build_main_volume_coverage(names: &[String]) -> Vec<MainVolumeCoverage> {
    let name_set: std::collections::BTreeSet<_> = names.iter().cloned().collect();
    (1..=36)
        .map(|n| {
            let main_file = format!("merge{n:02}.chm");
            let split_prefix = format!("merge{n:02}-");
            let has_main_file = name_set.contains(&main_file);
            let split_file_count = names
                .iter()
                .filter(|x| x.to_ascii_lowercase().ends_with(".chm") && x.starts_with(&split_prefix))
                .count();
            MainVolumeCoverage {
                volume: n as u8,
                has_main_file,
                split_file_count,
                covered: has_main_file || split_file_count > 0,
            }
        })
        .collect::<Vec<_>>()
}

fn extract_ascii_runs(bytes: &[u8], min_len: usize) -> Vec<String> {
    let mut out = Vec::new();
    let mut start: Option<usize> = None;

    for (i, b) in bytes.iter().enumerate() {
        if b.is_ascii_graphic() || *b == b' ' {
            if start.is_none() {
                start = Some(i);
            }
        } else if let Some(s) = start.take() {
            if i - s >= min_len {
                out.push(String::from_utf8_lossy(&bytes[s..i]).to_string());
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

fn extract_chm_paths(chm_bytes: &[u8], sample_limit: usize) -> (usize, Vec<String>, Vec<String>, Vec<String>) {
    let runs = extract_ascii_runs(chm_bytes, 6);
    let mut html: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut hhk: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    let mut hhc: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();

    for raw in runs {
        let s = trim_path_noise(&raw);
        let lower = s.to_ascii_lowercase();

        if (lower.ends_with(".htm") || lower.ends_with(".html")) && s.contains('/') {
            html.insert(s);
            continue;
        }
        if lower.ends_with(".hhk") && s.contains('/') {
            hhk.insert(s);
            continue;
        }
        if lower.ends_with(".hhc") && s.contains('/') {
            hhc.insert(s);
        }
    }

    let html_count = html.len();
    let sample_html_paths = html.into_iter().take(sample_limit).collect::<Vec<_>>();
    let hhk_files = hhk.into_iter().take(sample_limit).collect::<Vec<_>>();
    let hhc_files = hhc.into_iter().take(sample_limit).collect::<Vec<_>>();

    (html_count, sample_html_paths, hhk_files, hhc_files)
}

fn decode_euc_kr(bytes: &[u8]) -> String {
    let (s, _, _) = EUC_KR.decode(bytes);
    s.into_owned()
}

fn read_text_auto(path: &Path) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| format!("failed to read {}: {e}", path.display()))?;
    Ok(decode_euc_kr(&bytes))
}

fn strip_html_tags(input: &str) -> String {
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

fn compact_ws(input: &str) -> String {
    input.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn path_stem(path: &str) -> String {
    let base = path.rsplit('/').next().unwrap_or(path);
    base.rsplit_once('.')
        .map(|(s, _)| s)
        .unwrap_or(base)
        .trim()
        .to_string()
}

fn find_all_tag_values(text: &str, tag: &str) -> Vec<String> {
    let lower = text.to_ascii_lowercase();
    let open = format!("<{tag}>");
    let close = format!("</{tag}>");
    let mut out = Vec::new();
    let mut off = 0usize;
    while let Some(s) = lower[off..].find(&open) {
        let abs_s = off + s + open.len();
        if let Some(e_rel) = lower[abs_s..].find(&close) {
            let abs_e = abs_s + e_rel;
            out.push(text[abs_s..abs_e].trim().to_string());
            off = abs_e + close.len();
        } else {
            break;
        }
    }
    out
}

fn first_paragraph_html(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let p_start = lower.find("<p")?;
    let start_tag_end = lower[p_start..].find('>')? + p_start;
    let p_end = lower[start_tag_end + 1..].find("</p>")? + start_tag_end + 1;
    Some(text[start_tag_end + 1..p_end].to_string())
}

fn body_html(text: &str) -> Option<String> {
    let lower = text.to_ascii_lowercase();
    let b_start = lower.find("<body")?;
    let b_tag_end = lower[b_start..].find('>')? + b_start;
    let b_end = lower[b_tag_end + 1..].find("</body>")? + b_tag_end + 1;
    Some(text[b_tag_end + 1..b_end].to_string())
}

fn extract_first_bold_text(text: &str) -> Option<String> {
    let p_html = first_paragraph_html(text)?;
    let p_lower = p_html.to_ascii_lowercase();
    let b_start = p_lower.find("<b")?;
    let b_tag_end = p_lower[b_start..].find('>')? + b_start;
    let b_end = p_lower[b_tag_end + 1..].find("</b>")? + b_tag_end + 1;
    Some(compact_ws(&strip_html_tags(&p_html[b_tag_end + 1..b_end])))
}

fn resolve_debug_root(input: Option<String>) -> Result<PathBuf, String> {
    let raw = PathBuf::from(input.unwrap_or_else(|| "asset/debug".to_string()));
    if raw.exists() {
        return Ok(raw);
    }
    let cwd = std::env::current_dir().map_err(|e| format!("failed to read current dir: {e}"))?;
    for ancestor in cwd.ancestors() {
        let candidate = ancestor.join(&raw);
        if candidate.exists() {
            return Ok(candidate);
        }
    }
    Err(format!("debug root not found: {}", raw.display()))
}

fn resolve_runtime_source(input: Option<String>) -> Result<RuntimeSource, String> {
    match input {
        Some(raw) => {
            if raw.to_ascii_lowercase().ends_with(".zip") {
                return Ok(RuntimeSource::ZipPath(resolve_zip_path(&raw)?));
            }
            Ok(RuntimeSource::DebugRoot(resolve_debug_root(Some(raw))?))
        }
        None => Ok(RuntimeSource::DebugRoot(resolve_debug_root(None)?)),
    }
}

fn parse_master_hhc_text(text: &str) -> Vec<ContentItem> {
    let lower = text.to_ascii_lowercase();
    let mut out = Vec::new();
    let mut off = 0usize;
    while let Some(s) = lower[off..].find("<object type=\"text/sitemap\"") {
        let abs_s = off + s;
        let Some(e_rel) = lower[abs_s..].find("</object>") else {
            break;
        };
        let abs_e = abs_s + e_rel + "</object>".len();
        let block = &text[abs_s..abs_e];
        let mut name = String::new();
        let mut local = String::new();
        let block_lower = block.to_ascii_lowercase();
        let mut p_off = 0usize;
        while let Some(ps) = block_lower[p_off..].find("<param") {
            let p_abs = p_off + ps;
            let Some(pe_rel) = block[p_abs..].find('>') else {
                break;
            };
            let p_end = p_abs + pe_rel + 1;
            let t = &block[p_abs..p_end];
            let t_lower = t.to_ascii_lowercase();
            if t_lower.contains("name=\"name\"") || t_lower.contains("name='name'") {
                if let Some(v) = extract_attr_value(t, "value") {
                    name = compact_ws(v.trim());
                }
            }
            if t_lower.contains("name=\"local\"") || t_lower.contains("name='local'") {
                if let Some(v) = extract_attr_value(t, "value") {
                    local = compact_ws(v.trim());
                }
            }
            p_off = p_end;
        }
        if !name.is_empty() && !local.is_empty() {
            out.push(ContentItem { title: name, local });
        }
        off = abs_e;
    }
    out
}

fn parse_master_hhc_from_debug(debug_root: &Path) -> Result<Vec<ContentItem>, String> {
    let path = debug_root.join("master").join("master.hhc");
    let text = read_text_auto(&path)?;
    Ok(parse_master_hhc_text(&text))
}

fn visit_htm_files(root: &Path) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    let mut stack = vec![root.to_path_buf()];
    while let Some(dir) = stack.pop() {
        let rd = std::fs::read_dir(&dir).map_err(|e| format!("failed to read dir {}: {e}", dir.display()))?;
        for entry in rd {
            let entry = entry.map_err(|e| format!("failed dir entry in {}: {e}", dir.display()))?;
            let path = entry.path();
            let ft = entry
                .file_type()
                .map_err(|e| format!("failed file type in {}: {e}", path.display()))?;
            if ft.is_dir() {
                stack.push(path);
            } else if path
                .extension()
                .map(|x| x.to_string_lossy().to_ascii_lowercase() == "htm")
                .unwrap_or(false)
            {
                out.push(path);
            }
        }
    }
    Ok(out)
}

fn build_entry_detail(source_path: String, html_text: &str, file_stem: &str) -> Option<EntryDetail> {
    let titles = find_all_tag_values(html_text, "title")
        .into_iter()
        .map(|x| compact_ws(&strip_html_tags(&x)))
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();
    let bold = extract_first_bold_text(html_text).unwrap_or_default();

    let file_stem_owned = file_stem.to_string();
    let mut aliases = Vec::<String>::new();
    for v in titles.iter().chain([&bold, &file_stem_owned]) {
        let x = compact_ws(v);
        if !x.is_empty() && !aliases.contains(&x) {
            aliases.push(x);
        }
    }
    if aliases.is_empty() {
        return None;
    }

    let headword = aliases[0].clone();
    let def_html = first_paragraph_html(html_text).unwrap_or_default();
    let def_text = compact_ws(&strip_html_tags(&def_html));
    Some(EntryDetail {
        id: 0,
        headword,
        aliases,
        source_path,
        target_local: file_stem.to_string(),
        definition_text: def_text,
        definition_html: def_html,
    })
}

fn parse_entries_from_debug(debug_root: &Path) -> Result<Vec<EntryDetail>, String> {
    let files = visit_htm_files(debug_root)?;
    let mut out = Vec::new();
    for path in files {
        let rel = path
            .strip_prefix(debug_root)
            .map_err(|e| format!("strip prefix failed for {}: {e}", path.display()))?;
        let rel_str = rel.to_string_lossy().to_string();
        let rel_lower = rel_str.to_ascii_lowercase();
        if !rel_lower.starts_with("merge") {
            continue;
        }
        if rel_lower.contains("zahl_der_woerter") || rel_lower.contains("woeterliste_") {
            continue;
        }

        let text = read_text_auto(&path)?;
        let file_stem = path
            .file_stem()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some(entry) = build_entry_detail(rel_str, &text, &file_stem) {
            out.push(entry);
        }
    }

    out.sort_by(|a, b| normalize_search_key(&a.headword).cmp(&normalize_search_key(&b.headword)));
    for (i, e) in out.iter_mut().enumerate() {
        e.id = i + 1;
    }
    Ok(out)
}

fn parse_entries_from_debug_with_progress(
    debug_root: &Path,
    mut progress: Option<&mut dyn FnMut(BuildProgress)>,
) -> Result<Vec<EntryDetail>, String> {
    let files = visit_htm_files(debug_root)?;
    if let Some(cb) = progress.as_mut() {
        cb(BuildProgress {
            phase: "scan".to_string(),
            current: 0,
            total: files.len(),
            message: "HTML files discovered".to_string(),
        });
    }

    let mut out = Vec::new();
    let total = files.len();
    for (idx, path) in files.into_iter().enumerate() {
        let rel = path
            .strip_prefix(debug_root)
            .map_err(|e| format!("strip prefix failed for {}: {e}", path.display()))?;
        let rel_str = rel.to_string_lossy().to_string();
        let rel_lower = rel_str.to_ascii_lowercase();
        if !rel_lower.starts_with("merge") {
            continue;
        }
        if rel_lower.contains("zahl_der_woerter") || rel_lower.contains("woeterliste_") {
            continue;
        }

        let text = read_text_auto(&path)?;
        let file_stem = path
            .file_stem()
            .map(|x| x.to_string_lossy().to_string())
            .unwrap_or_default();
        if let Some(entry) = build_entry_detail(rel_str, &text, &file_stem) {
            out.push(entry);
        }

        if idx % 400 == 0 {
            if let Some(cb) = progress.as_mut() {
                cb(BuildProgress {
                    phase: "parse".to_string(),
                    current: idx + 1,
                    total,
                    message: "Parsing entries".to_string(),
                });
            }
        }
    }

    out.sort_by(|a, b| normalize_search_key(&a.headword).cmp(&normalize_search_key(&b.headword)));
    for (i, e) in out.iter_mut().enumerate() {
        e.id = i + 1;
    }

    if let Some(cb) = progress.as_mut() {
        cb(BuildProgress {
            phase: "finalize".to_string(),
            current: out.len(),
            total: out.len(),
            message: "Finalizing index".to_string(),
        });
    }
    Ok(out)
}

fn extract_all_headwords_from_chm_bytes(chm_bytes: &[u8]) -> Vec<String> {
    let html_paths = if let Ok(chm) = chm::ChmArchive::open(chm_bytes.to_vec()) {
        chm.entries()
            .iter()
            .filter(|e| e.path.to_ascii_lowercase().ends_with(".htm"))
            .map(|e| e.path.clone())
            .collect::<Vec<_>>()
    } else {
        let (_, html_paths, _, _) = extract_chm_paths(chm_bytes, usize::MAX);
        html_paths
    };
    let mut words = std::collections::BTreeSet::new();
    for p in html_paths {
        let path = p.trim_start_matches('/');
        let filename = path.rsplit('/').next().unwrap_or(path);
        let base = filename
            .rsplit_once('.')
            .map(|(b, _)| b)
            .unwrap_or(filename)
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
    let mut off = 0usize;
    while let Some(s) = lower[off..].find("<object type=\"text/sitemap\"") {
        let abs_s = off + s;
        let Some(e_rel) = lower[abs_s..].find("</object>") else {
            break;
        };
        let abs_e = abs_s + e_rel + "</object>".len();
        let block = &text[abs_s..abs_e];
        let mut name = String::new();
        let mut local = String::new();
        let block_lower = block.to_ascii_lowercase();
        let mut p_off = 0usize;
        while let Some(ps) = block_lower[p_off..].find("<param") {
            let p_abs = p_off + ps;
            let Some(pe_rel) = block[p_abs..].find('>') else {
                break;
            };
            let p_end = p_abs + pe_rel + 1;
            let tag = &block[p_abs..p_end];
            let tag_lower = tag.to_ascii_lowercase();
            if tag_lower.contains("name=\"name\"") || tag_lower.contains("name='name'") {
                if let Some(v) = extract_attr_value(tag, "value") {
                    name = compact_ws(v.trim());
                }
            }
            if tag_lower.contains("name=\"local\"") || tag_lower.contains("name='local'") {
                if let Some(v) = extract_attr_value(tag, "value") {
                    local = compact_ws(v.trim());
                }
            }
            p_off = p_end;
        }

        if !name.is_empty() {
            let (source_override, local_path, _) = parse_internal_ref(&local)
                .unwrap_or((None, local.clone(), false));
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
        off = abs_e;
    }
    out
}

fn extract_index_entries_from_chm_bytes(chm_file_name: &str, chm_bytes: &[u8]) -> Vec<EntryDetail> {
    let mut out = Vec::<EntryDetail>::new();
    let Ok(mut chm) = chm::ChmArchive::open(chm_bytes.to_vec()) else {
        return out;
    };

    let hhk_paths = chm
        .entries()
        .iter()
        .filter(|e| e.path.to_ascii_lowercase().ends_with(".hhk"))
        .map(|e| e.path.clone())
        .collect::<Vec<_>>();

    for hhk_path in hhk_paths {
        if let Ok(bytes) = chm.read_object(&hhk_path) {
            let text = decode_euc_kr(&bytes);
            let mut rows = parse_hhk_entries_from_text(&text, chm_file_name);
            out.append(&mut rows);
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

fn decode_basic_html_entities(s: &str) -> String {
    s.replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&#39;", "'")
}

fn extract_attr_value(tag: &str, attr_name: &str) -> Option<String> {
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

#[cfg(test)]
fn extract_headwords_from_hhk_bytes(chm_bytes: &[u8], sample_limit: usize) -> (usize, Vec<String>, usize) {
    let text = String::from_utf8_lossy(chm_bytes);
    let text_lower = text.to_ascii_lowercase();
    let hhk_path_count = text_lower.matches(".hhk").count();

    let mut out = std::collections::BTreeSet::new();
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
        let is_name_param =
            tag_lower.contains("name=\"name\"") || tag_lower.contains("name='name'");
        if is_name_param {
            if let Some(raw_value) = extract_attr_value(tag, "value") {
                let value = decode_basic_html_entities(raw_value.trim());
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

fn resolve_zip_path(input: &str) -> Result<PathBuf, String> {
    let raw = PathBuf::from(input);
    if raw.is_absolute() && raw.exists() {
        return Ok(raw);
    }
    if raw.exists() {
        return Ok(raw);
    }

    let cwd = std::env::current_dir().map_err(|e| format!("failed to read current dir: {e}"))?;
    let mut attempts = Vec::new();

    for ancestor in cwd.ancestors() {
        let candidate = ancestor.join(&raw);
        attempts.push(candidate.display().to_string());
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "zip not found: '{}'. searched current/ancestor paths from '{}'. attempts: {}",
        input,
        cwd.display(),
        attempts.join(", ")
    ))
}

pub(crate) fn analyze_zip_dataset_impl(zip_path: String) -> Result<DatasetSummary, String> {
    let resolved = resolve_zip_path(&zip_path)?;
    summarize_zip(Path::new(&resolved))
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn split_volume_counts_as_covered() {
        let names = vec![
            "merge01.chm".to_string(),
            "merge03-01.chm".to_string(),
            "merge03-02.chm".to_string(),
        ];
        let coverage = build_main_volume_coverage(&names);
        let v1 = coverage.iter().find(|x| x.volume == 1).expect("v1");
        let v3 = coverage.iter().find(|x| x.volume == 3).expect("v3");
        let v4 = coverage.iter().find(|x| x.volume == 4).expect("v4");
        assert!(v1.covered && v1.has_main_file);
        assert!(v3.covered && !v3.has_main_file && v3.split_file_count == 2);
        assert!(!v4.covered);
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

    #[test]
    fn dataset_smoke_validation_if_present() {
        let candidates = [
            PathBuf::from("../asset/dictionary_v77.zip"),
            PathBuf::from("asset/dictionary_v77.zip"),
        ];
        let Some(path) = candidates.into_iter().find(|p| p.exists()) else {
            return;
        };

        let summary = summarize_zip(&path).expect("summary");
        assert_eq!(summary.chm_count, 120);
        assert_eq!(summary.lnk_count, 2);
        assert_eq!(summary.txt_count, 1);
        assert!(summary.has_master_chm);
        assert!(summary.has_readme_txt);
        assert!(summary.missing_main_volumes.is_empty());
    }

    #[test]
    fn dataset_hhk_smoke_if_present() {
        let candidates = [
            PathBuf::from("../asset/dictionary_v77.zip"),
            PathBuf::from("asset/dictionary_v77.zip"),
        ];
        let Some(path) = candidates.into_iter().find(|p| p.exists()) else {
            return;
        };

        let file = File::open(&path).expect("open zip");
        let mut archive = ZipArchive::new(file).expect("zip");
        let mut detected = 0usize;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("entry");
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_ascii_lowercase();
            if !name.ends_with(".chm") {
                continue;
            }
            let mut bytes = Vec::new();
            std::io::copy(&mut entry, &mut bytes).expect("copy");
            let (_count, _sample, hhk_paths) = extract_headwords_from_hhk_bytes(&bytes, 3);
            if hhk_paths > 0 {
                detected += 1;
            }
            if detected >= 3 {
                break;
            }
        }
        assert!(detected > 0, "expected at least one CHM to expose .hhk hints");
    }
}
