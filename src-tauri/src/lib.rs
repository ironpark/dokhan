use encoding_rs::EUC_KR;
use base64::Engine as _;
use serde::Serialize;
use std::collections::BTreeMap;
use std::fs::File;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, OnceLock};
use tauri::Emitter;
use zip::ZipArchive;

mod chm;
mod commands;
pub use commands::run;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct FileTypeCount {
    extension: String,
    count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatasetSummary {
    zip_path: String,
    total_entries: usize,
    total_files: usize,
    total_dirs: usize,
    chm_count: usize,
    lnk_count: usize,
    txt_count: usize,
    uncompressed_bytes: u64,
    compressed_bytes: u64,
    compression_ratio: f64,
    has_master_chm: bool,
    has_readme_txt: bool,
    missing_main_volumes: Vec<String>,
    missing_main_files_only: Vec<String>,
    main_volume_coverage: Vec<MainVolumeCoverage>,
    extension_counts: Vec<FileTypeCount>,
    sample_chm_files: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DatasetValidationReport {
    zip_path: String,
    chm_count: usize,
    covered_main_volumes: usize,
    missing_main_volumes: Vec<String>,
    has_master_chm: bool,
    has_readme_txt: bool,
    total_estimated_headwords: usize,
    chm_with_zero_headwords: Vec<String>,
    top_headword_chm: Vec<HeadwordCountRow>,
    warnings: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HeadwordCountRow {
    chm_file: String,
    headword_count: usize,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MainVolumeCoverage {
    volume: u8,
    has_main_file: bool,
    split_file_count: usize,
    covered: bool,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ChmPathPreview {
    chm_file: String,
    html_path_count: usize,
    sample_html_paths: Vec<String>,
    hhk_files: Vec<String>,
    hhc_files: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ContentItem {
    title: String,
    local: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct DictionaryIndexEntry {
    id: usize,
    headword: String,
    aliases: Vec<String>,
    source_path: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct SearchHit {
    id: usize,
    headword: String,
    source_path: String,
    score: usize,
    snippet: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct EntryDetail {
    id: usize,
    headword: String,
    aliases: Vec<String>,
    source_path: String,
    target_local: String,
    definition_text: String,
    definition_html: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ContentPage {
    local: String,
    source_path: String,
    title: String,
    body_text: String,
    body_html: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
enum LinkTarget {
    Content { local: String, source_path: String },
    Entry { id: usize },
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct MasterFeatureSummary {
    debug_root: String,
    content_count: usize,
    index_count: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BuildProgress {
    phase: String,
    current: usize,
    total: usize,
    message: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct BuildStatus {
    phase: String,
    current: usize,
    total: usize,
    message: String,
    done: bool,
    success: bool,
    error: Option<String>,
    summary: Option<MasterFeatureSummary>,
}

#[derive(Debug, Clone)]
struct RuntimeIndex {
    contents: Vec<ContentItem>,
    entries: Vec<EntryDetail>,
    content_pages: BTreeMap<String, ContentPage>,
    entry_keys: Vec<EntrySearchKey>,
}

#[derive(Debug, Clone)]
struct EntrySearchKey {
    headword: String,
    headword_loose: String,
    body: String,
    body_loose: String,
    aliases: Vec<String>,
    aliases_loose: Vec<String>,
}

#[derive(Debug, Clone)]
enum RuntimeSource {
    DebugRoot(PathBuf),
    ZipPath(PathBuf),
}

static RUNTIME_CACHE: OnceLock<Mutex<BTreeMap<String, Arc<RuntimeIndex>>>> = OnceLock::new();
static BUILD_STATUS: OnceLock<Mutex<BTreeMap<String, BuildStatus>>> = OnceLock::new();

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

fn normalize_search_key(s: &str) -> String {
    s.to_ascii_lowercase()
        .replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ü", "ue")
        .replace("ß", "ss")
}

fn normalize_search_key_loose(s: &str) -> String {
    normalize_search_key(s)
        .replace("ae", "a")
        .replace("oe", "o")
        .replace("ue", "u")
}

fn starts_with_search_key_precomputed(
    value_key: &str,
    value_loose: &str,
    prefix_key: &str,
    prefix_loose: &str,
) -> bool {
    value_key.starts_with(prefix_key) || value_loose.starts_with(prefix_loose)
}

fn contains_search_key_precomputed(
    value_key: &str,
    value_loose: &str,
    term_key: &str,
    term_loose: &str,
) -> bool {
    value_key.contains(term_key) || value_loose.contains(term_loose)
}

fn eq_search_key(a: &str, b: &str) -> bool {
    if normalize_search_key(a) == normalize_search_key(b) {
        return true;
    }
    normalize_search_key_loose(a) == normalize_search_key_loose(b)
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

fn cache_key(source: &RuntimeSource) -> String {
    match source {
        RuntimeSource::DebugRoot(path) => format!(
            "debug:{}",
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .to_string_lossy()
        ),
        RuntimeSource::ZipPath(path) => format!(
            "zip:{}",
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .to_string_lossy()
        ),
    }
}

fn source_label(source: &RuntimeSource) -> String {
    match source {
        RuntimeSource::DebugRoot(path) | RuntimeSource::ZipPath(path) => path.to_string_lossy().to_string(),
    }
}

fn status_key(source: &RuntimeSource) -> String {
    cache_key(source)
}

fn set_build_status(key: &str, status: BuildStatus) -> Result<(), String> {
    let map = BUILD_STATUS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = map.lock().map_err(|_| "build status lock poisoned".to_string())?;
    guard.insert(key.to_string(), status);
    Ok(())
}

fn update_build_status<F>(key: &str, updater: F) -> Result<(), String>
where
    F: FnOnce(&mut BuildStatus),
{
    let map = BUILD_STATUS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = map.lock().map_err(|_| "build status lock poisoned".to_string())?;
    if let Some(st) = guard.get_mut(key) {
        updater(st);
    }
    Ok(())
}

fn get_build_status_internal(key: &str) -> Result<Option<BuildStatus>, String> {
    let map = BUILD_STATUS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let guard = map.lock().map_err(|_| "build status lock poisoned".to_string())?;
    Ok(guard.get(key).cloned())
}

fn cache_get(source: &RuntimeSource) -> Result<Option<Arc<RuntimeIndex>>, String> {
    let key = cache_key(source);
    let cache = RUNTIME_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let guard = cache.lock().map_err(|_| "runtime cache lock poisoned".to_string())?;
    Ok(guard.get(&key).cloned())
}

fn cache_put(source: &RuntimeSource, runtime: Arc<RuntimeIndex>) -> Result<(), String> {
    let key = cache_key(source);
    let cache = RUNTIME_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = cache.lock().map_err(|_| "runtime cache lock poisoned".to_string())?;
    guard.insert(key, runtime);
    Ok(())
}

fn build_entry_search_keys(entries: &[EntryDetail]) -> Vec<EntrySearchKey> {
    entries
        .iter()
        .map(|e| {
            let aliases = e.aliases.iter().map(|a| normalize_search_key(a)).collect::<Vec<_>>();
            let aliases_loose = e
                .aliases
                .iter()
                .map(|a| normalize_search_key_loose(a))
                .collect::<Vec<_>>();
            EntrySearchKey {
                headword: normalize_search_key(&e.headword),
                headword_loose: normalize_search_key_loose(&e.headword),
                body: normalize_search_key(&e.definition_text),
                body_loose: normalize_search_key_loose(&e.definition_text),
                aliases,
                aliases_loose,
            }
        })
        .collect::<Vec<_>>()
}

fn build_runtime_index(
    contents: Vec<ContentItem>,
    entries: Vec<EntryDetail>,
    content_pages: BTreeMap<String, ContentPage>,
) -> RuntimeIndex {
    let entry_keys = build_entry_search_keys(&entries);
    RuntimeIndex {
        contents,
        entries,
        content_pages,
        entry_keys,
    }
}

fn get_runtime(source: &RuntimeSource) -> Result<Arc<RuntimeIndex>, String> {
    if let Some(v) = cache_get(source)? {
        return Ok(v);
    }
    let runtime = match source {
        RuntimeSource::DebugRoot(root) => {
            let contents = parse_master_hhc_from_debug(root)?;
            let entries = parse_entries_from_debug(root)?;
            Arc::new(build_runtime_index(contents, entries, BTreeMap::new()))
        }
        RuntimeSource::ZipPath(zip_path) => Arc::new(parse_runtime_from_zip_with_progress(zip_path, None)?),
    };
    cache_put(source, runtime.clone())?;
    Ok(runtime)
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HeadwordPreview {
    chm_file: String,
    headword_count: usize,
    sample_headwords: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct HhkHeadwordPreview {
    chm_file: String,
    hhk_path_count: usize,
    headword_count: usize,
    sample_headwords: Vec<String>,
}

pub(crate) fn extract_headwords_preview_impl(
    zip_path: Option<String>,
    chm_file: Option<String>,
    sample_limit: Option<usize>,
) -> Result<Vec<HeadwordPreview>, String> {
    let resolved_path = resolve_or_default_zip(zip_path)?;
    let mut archive = open_zip_archive(&resolved_path)?;
    let sample_limit = sample_limit.unwrap_or(20).clamp(1, 100);
    for_each_chm_entry(&mut archive, chm_file.as_deref(), |name, bytes| {
        let (headword_count, sample_headwords) = extract_headwords_from_chm_bytes(&bytes, sample_limit);
        Ok(HeadwordPreview {
            chm_file: name,
            headword_count,
            sample_headwords,
        })
    })
}

fn extract_headwords_from_chm_bytes(chm_bytes: &[u8], sample_limit: usize) -> (usize, Vec<String>) {
    let (_, html_paths, _, _) = extract_chm_paths(chm_bytes, 50_000);
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
        let base_lower = base.to_ascii_lowercase();
        if matches!(
            base_lower.as_str(),
            "master" | "index" | "version_information" | "dictionary" | "a"
        ) {
            continue;
        }
        words.insert(base.to_string());
    }

    let count = words.len();
    let sample = words.into_iter().take(sample_limit).collect::<Vec<_>>();
    (count, sample)
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

fn find_default_zip() -> Result<PathBuf, String> {
    resolve_zip_path("asset/dictionary_v77.zip")
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

fn resolve_or_default_zip(zip_path: Option<String>) -> Result<PathBuf, String> {
    match zip_path {
        Some(p) => resolve_zip_path(&p),
        None => find_default_zip(),
    }
}

fn open_zip_archive(path: &Path) -> Result<ZipArchive<File>, String> {
    let file = File::open(path).map_err(|e| format!("failed to open zip: {e}"))?;
    ZipArchive::new(file).map_err(|e| format!("invalid zip archive: {e}"))
}

fn decode_content_page(local: String, source_path: String, bytes: &[u8]) -> ContentPage {
    let text = decode_euc_kr(bytes);
    let title = find_all_tag_values(&text, "title")
        .into_iter()
        .map(|x| compact_ws(&strip_html_tags(&x)))
        .find(|x| !x.is_empty())
        .unwrap_or_else(|| local.clone());
    let b_html = body_html(&text).unwrap_or_default();
    let b_text = compact_ws(&strip_html_tags(&b_html));
    ContentPage {
        local,
        source_path,
        title,
        body_text: b_text,
        body_html: b_html,
    }
}

fn resolve_local_candidates(local: &str) -> Vec<String> {
    let raw = local.trim().trim_start_matches('/').to_string();
    let mut out = Vec::new();
    if !raw.is_empty() {
        out.push(raw.clone());
    }
    let raw_lower = raw.to_ascii_lowercase();
    if !raw_lower.contains('.') {
        out.push(format!("{raw}.html"));
        out.push(format!("{raw}.htm"));
    }
    if raw_lower == "master" {
        out.push("master.html".to_string());
        out.push("master.htm".to_string());
    }
    out.sort();
    out.dedup();
    out
}

fn read_chm_object_with_candidates(chm: &mut chm::ChmArchive, local: &str) -> Option<Vec<u8>> {
    let candidates = resolve_local_candidates(local);
    for c in &candidates {
        if let Ok(v) = chm.read_object(c) {
            return Some(v);
        }
        let slash = format!("/{c}");
        if let Ok(v) = chm.read_object(&slash) {
            return Some(v);
        }
    }

    let candidate_lowers = candidates
        .iter()
        .map(|x| x.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let matched_paths = chm
        .entries()
        .iter()
        .filter_map(|e| {
            let base = e.path.rsplit('/').next().unwrap_or(&e.path).to_ascii_lowercase();
            if candidate_lowers.iter().any(|c| c == &base) {
                Some(e.path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for p in matched_paths {
        if let Ok(v) = chm.read_object(&p) {
            return Some(v);
        }
    }
    None
}

fn read_named_chm_from_zip(zip_path: &Path, chm_name: &str) -> Result<Vec<u8>, String> {
    let mut archive = open_zip_archive(zip_path)?;
    let target = chm_name.to_ascii_lowercase();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry #{i}: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        if entry.name().to_ascii_lowercase() != target {
            continue;
        }
        let mut bytes = Vec::new();
        std::io::copy(&mut entry, &mut bytes)
            .map_err(|e| format!("failed to load {chm_name} from zip: {e}"))?;
        return Ok(bytes);
    }
    Err(format!("chm not found in zip: {chm_name}"))
}

fn read_entry_html_from_chm(chm: &mut chm::ChmArchive, headword: &str) -> Option<Vec<u8>> {
    for candidate in resolve_local_candidates(headword) {
        if let Ok(v) = chm.read_object(&candidate) {
            return Some(v);
        }
        let slash = format!("/{candidate}");
        if let Ok(v) = chm.read_object(&slash) {
            return Some(v);
        }
    }

    let wanted = normalize_search_key(headword);
    let mut matches = chm
        .entries()
        .iter()
        .filter_map(|e| {
            let base = e.path.rsplit('/').next().unwrap_or(&e.path);
            let stem = base
                .rsplit_once('.')
                .map(|(s, _)| s)
                .unwrap_or(base);
            let ext = base
                .rsplit_once('.')
                .map(|(_, x)| x.to_ascii_lowercase())
                .unwrap_or_default();
            if (ext == "htm" || ext == "html") && normalize_search_key(stem) == wanted {
                Some(e.path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    for path in matches {
        if let Ok(v) = chm.read_object(&path) {
            return Some(v);
        }
    }
    None
}

fn hydrate_zip_entry_detail(zip_path: &Path, mut entry: EntryDetail) -> EntryDetail {
    if !entry.definition_text.is_empty() {
        return entry;
    }
    let Ok(chm_bytes) = read_named_chm_from_zip(zip_path, &entry.source_path) else {
        return entry;
    };
    let Ok(mut chm) = chm::ChmArchive::open(chm_bytes) else {
        return entry;
    };
    let html_bytes = if !entry.target_local.is_empty() {
        read_chm_binary_object(&mut chm, &entry.target_local)
            .or_else(|| read_entry_html_from_chm(&mut chm, &entry.headword))
    } else {
        read_entry_html_from_chm(&mut chm, &entry.headword)
    };
    let Some(html_bytes) = html_bytes else {
        return entry;
    };

    let html_text = decode_euc_kr(&html_bytes);
    let paragraph_html = first_paragraph_html(&html_text).unwrap_or_default();
    let paragraph_text = compact_ws(&strip_html_tags(&paragraph_html));
    let body = body_html(&html_text).unwrap_or_default();
    let body_text = compact_ws(&strip_html_tags(&body));
    if !paragraph_html.is_empty() {
        entry.definition_html = paragraph_html.clone();
    } else if !body.is_empty() {
        entry.definition_html = body.clone();
    }
    if !paragraph_text.is_empty() {
        entry.definition_text = paragraph_text;
    } else if !body_text.is_empty() {
        entry.definition_text = body_text;
    }

    let title_aliases = find_all_tag_values(&html_text, "title")
        .into_iter()
        .map(|x| compact_ws(&strip_html_tags(&x)))
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();
    for alias in title_aliases {
        if !entry.aliases.contains(&alias) {
            entry.aliases.push(alias);
        }
    }
    if let Some(bold) = extract_first_bold_text(&html_text) {
        let bold = compact_ws(&bold);
        if !bold.is_empty() && !entry.aliases.contains(&bold) {
            entry.aliases.push(bold);
        }
    }
    entry
}

fn read_content_page_from_zip(
    zip_path: &Path,
    source_path: &str,
    local: &str,
) -> Result<ContentPage, String> {
    let bytes = read_named_chm_from_zip(zip_path, source_path)?;
    let mut chm = chm::ChmArchive::open(bytes).map_err(|e| format!("failed to open {source_path}: {e}"))?;
    if let Some(v) = read_chm_object_with_candidates(&mut chm, local) {
        return Ok(decode_content_page(local.to_string(), source_path.to_string(), &v));
    }
    Err(format!(
        "content page not found in zip runtime: {source_path}::{local}"
    ))
}

fn read_content_page_from_debug(
    root: &Path,
    source_path: &str,
    local: &str,
) -> Result<ContentPage, String> {
    let bytes = read_debug_binary_object(root, source_path, local)?;
    Ok(decode_content_page(
        local.to_string(),
        source_path.to_string(),
        &bytes,
    ))
}

fn for_each_chm_entry<T, F>(
    archive: &mut ZipArchive<File>,
    target: Option<&str>,
    mut f: F,
) -> Result<Vec<T>, String>
where
    F: FnMut(String, Vec<u8>) -> Result<T, String>,
{
    let mut out = Vec::new();
    let target = target.map(|x| x.to_ascii_lowercase());

    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry #{i}: {e}"))?;
        if entry.is_dir() {
            continue;
        }

        let name = entry.name().to_string();
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".chm") {
            continue;
        }
        if let Some(ref t) = target {
            if &lower != t {
                continue;
            }
        }

        let mut bytes = Vec::new();
        std::io::copy(&mut entry, &mut bytes).map_err(|e| format!("failed to load {name}: {e}"))?;
        out.push(f(name, bytes)?);
    }

    Ok(out)
}

fn parse_runtime_from_zip_with_progress(
    zip_path: &Path,
    mut progress: Option<&mut dyn FnMut(BuildProgress)>,
) -> Result<RuntimeIndex, String> {
    let mut archive = open_zip_archive(zip_path)?;
    let total = archive.len();
    let mut entries = Vec::<EntryDetail>::new();
    let mut contents = Vec::<ContentItem>::new();

    for i in 0..total {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry #{i}: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".chm") {
            continue;
        }
        let mut bytes = Vec::new();
        std::io::copy(&mut entry, &mut bytes).map_err(|e| format!("failed to load {name}: {e}"))?;

        if lower.ends_with("master.chm") {
            if let Ok(mut chm) = chm::ChmArchive::open(bytes.clone()) {
                if let Some(hhc) = read_chm_object_with_candidates(&mut chm, "master.hhc") {
                    let text = decode_euc_kr(&hhc);
                    let parsed = parse_master_hhc_text(&text);
                    if !parsed.is_empty() {
                        contents = parsed;
                    }
                }
            }
            if contents.is_empty() {
                contents.push(ContentItem {
                    title: "목차".to_string(),
                    local: "master".to_string(),
                });
            }
        }

        if lower.starts_with("merge") {
            entries.extend(extract_index_entries_from_chm_bytes(&name, &bytes));
        }

        if let Some(cb) = progress.as_mut() {
            cb(BuildProgress {
                phase: "parse".to_string(),
                current: i + 1,
                total,
                message: format!("Parsing {}", name),
            });
        }
    }

    entries.sort_by(|a, b| {
        normalize_search_key(&a.headword)
            .cmp(&normalize_search_key(&b.headword))
            .then_with(|| a.source_path.cmp(&b.source_path))
            .then_with(|| a.target_local.cmp(&b.target_local))
    });
    entries.dedup_by(|a, b| {
        normalize_search_key(&a.headword) == normalize_search_key(&b.headword)
            && a.source_path == b.source_path
            && a.target_local == b.target_local
    });
    for (i, e) in entries.iter_mut().enumerate() {
        e.id = i + 1;
    }
    Ok(build_runtime_index(contents, entries, BTreeMap::new()))
}

pub(crate) fn analyze_zip_dataset_impl(zip_path: String) -> Result<DatasetSummary, String> {
    let resolved = resolve_zip_path(&zip_path)?;
    summarize_zip(Path::new(&resolved))
}

pub(crate) fn analyze_default_dataset_impl() -> Result<DatasetSummary, String> {
    let path = find_default_zip()?;
    summarize_zip(&path)
}

pub(crate) fn preview_chm_paths_impl(
    zip_path: Option<String>,
    sample_limit: Option<usize>,
) -> Result<Vec<ChmPathPreview>, String> {
    let resolved_path = resolve_or_default_zip(zip_path)?;
    let mut archive = open_zip_archive(&resolved_path)?;
    let sample_limit = sample_limit.unwrap_or(10).clamp(1, 50);
    for_each_chm_entry(&mut archive, None, |name, bytes| {
        let (html_path_count, sample_html_paths, hhk_files, hhc_files) =
            extract_chm_paths(&bytes, sample_limit);
        Ok(ChmPathPreview {
            chm_file: name,
            html_path_count,
            sample_html_paths,
            hhk_files,
            hhc_files,
        })
    })
}

pub(crate) fn extract_headwords_from_hhk_impl(
    zip_path: Option<String>,
    chm_file: Option<String>,
    sample_limit: Option<usize>,
) -> Result<Vec<HhkHeadwordPreview>, String> {
    let resolved_path = resolve_or_default_zip(zip_path)?;
    let mut archive = open_zip_archive(&resolved_path)?;
    let sample_limit = sample_limit.unwrap_or(20).clamp(1, 100);
    for_each_chm_entry(&mut archive, chm_file.as_deref(), |name, bytes| {
        let (headword_count, sample_headwords, hhk_path_count) =
            extract_headwords_from_hhk_bytes(&bytes, sample_limit);
        Ok(HhkHeadwordPreview {
            chm_file: name,
            hhk_path_count,
            headword_count,
            sample_headwords,
        })
    })
}

pub(crate) fn validate_dataset_pipeline_impl(
    zip_path: Option<String>,
) -> Result<DatasetValidationReport, String> {
    let resolved_path = resolve_or_default_zip(zip_path)?;

    let summary = summarize_zip(&resolved_path)?;
    let mut archive = open_zip_archive(&resolved_path)?;

    let mut total_estimated_headwords = 0usize;
    let mut chm_with_zero_headwords = Vec::new();
    let mut rows = for_each_chm_entry(&mut archive, None, |name, bytes| {
        let (count, _) = extract_headwords_from_chm_bytes(&bytes, 1);
        let (hhk_count, _, _) = extract_headwords_from_hhk_bytes(&bytes, 1);
        total_estimated_headwords += count;

        if count == 0 && hhk_count == 0 {
            chm_with_zero_headwords.push(name.clone());
        }
        Ok(HeadwordCountRow {
            chm_file: name,
            headword_count: hhk_count.max(count),
        })
    })?;

    rows.sort_by(|a, b| b.headword_count.cmp(&a.headword_count).then_with(|| a.chm_file.cmp(&b.chm_file)));
    let top_headword_chm = rows.into_iter().take(10).collect::<Vec<_>>();
    let covered_main_volumes = summary.main_volume_coverage.iter().filter(|x| x.covered).count();

    let mut warnings = Vec::new();
    if !summary.has_master_chm {
        warnings.push("master.chm is missing".to_string());
    }
    if !summary.has_readme_txt {
        warnings.push("readme.txt is missing".to_string());
    }
    if !summary.missing_main_volumes.is_empty() {
        warnings.push(format!(
            "missing covered main volumes: {}",
            summary.missing_main_volumes.join(", ")
        ));
    }
    if !chm_with_zero_headwords.is_empty() {
        warnings.push(format!(
            "{} CHM files produced zero estimated headwords",
            chm_with_zero_headwords.len()
        ));
    }

    Ok(DatasetValidationReport {
        zip_path: summary.zip_path,
        chm_count: summary.chm_count,
        covered_main_volumes,
        missing_main_volumes: summary.missing_main_volumes,
        has_master_chm: summary.has_master_chm,
        has_readme_txt: summary.has_readme_txt,
        total_estimated_headwords,
        chm_with_zero_headwords,
        top_headword_chm,
        warnings,
    })
}

pub(crate) fn build_master_features_impl(
    debug_root: Option<String>,
) -> Result<MasterFeatureSummary, String> {
    let source = resolve_runtime_source(debug_root)?;
    let runtime = get_runtime(&source)?;
    Ok(MasterFeatureSummary {
        debug_root: source_label(&source),
        content_count: runtime.contents.len(),
        index_count: runtime.entries.len(),
    })
}

pub(crate) fn build_master_features_with_progress_impl(
    window: tauri::Window,
    debug_root: Option<String>,
) -> Result<MasterFeatureSummary, String> {
    let source = resolve_runtime_source(debug_root)?;
    if let Some(runtime) = cache_get(&source)? {
        let _ = window.emit(
            "master-build-progress",
            BuildProgress {
                phase: "done".to_string(),
                current: runtime.entries.len(),
                total: runtime.entries.len(),
                message: "Loaded from cache".to_string(),
            },
        );
        return Ok(MasterFeatureSummary {
            debug_root: source_label(&source),
            content_count: runtime.contents.len(),
            index_count: runtime.entries.len(),
        });
    }

    let _ = window.emit(
        "master-build-progress",
        BuildProgress {
            phase: "start".to_string(),
            current: 0,
            total: 1,
            message: "Starting build".to_string(),
        },
    );

    let runtime = match &source {
        RuntimeSource::DebugRoot(root) => {
            let contents = parse_master_hhc_from_debug(root)?;
            let mut emit = |p: BuildProgress| {
                let _ = window.emit("master-build-progress", p);
            };
            let entries = parse_entries_from_debug_with_progress(root, Some(&mut emit))?;
            Arc::new(build_runtime_index(contents, entries, BTreeMap::new()))
        }
        RuntimeSource::ZipPath(zip_path) => {
            let mut emit = |p: BuildProgress| {
                let _ = window.emit("master-build-progress", p);
            };
            Arc::new(parse_runtime_from_zip_with_progress(zip_path, Some(&mut emit))?)
        }
    };
    let summary = MasterFeatureSummary {
        debug_root: source_label(&source),
        content_count: runtime.contents.len(),
        index_count: runtime.entries.len(),
    };
    cache_put(&source, runtime)?;
    let _ = window.emit(
        "master-build-progress",
        BuildProgress {
            phase: "done".to_string(),
            current: summary.index_count,
            total: summary.index_count,
            message: "Build complete".to_string(),
        },
    );
    Ok(summary)
}

pub(crate) fn start_master_build_impl(debug_root: Option<String>) -> Result<String, String> {
    let source = resolve_runtime_source(debug_root)?;
    let key = status_key(&source);

    if let Some(runtime) = cache_get(&source)? {
        let summary = MasterFeatureSummary {
            debug_root: source_label(&source),
            content_count: runtime.contents.len(),
            index_count: runtime.entries.len(),
        };
        set_build_status(
            &key,
            BuildStatus {
                phase: "done".to_string(),
                current: summary.index_count,
                total: summary.index_count,
                message: "Loaded from cache".to_string(),
                done: true,
                success: true,
                error: None,
                summary: Some(summary),
            },
        )?;
        return Ok(key);
    }

    if let Some(st) = get_build_status_internal(&key)? {
        if !st.done {
            return Ok(key);
        }
    }

    set_build_status(
        &key,
        BuildStatus {
            phase: "start".to_string(),
            current: 0,
            total: 1,
            message: "Starting build".to_string(),
            done: false,
            success: false,
            error: None,
            summary: None,
        },
    )?;

    let source_clone = source.clone();
    let key_clone = key.clone();
    std::thread::spawn(move || {
        let runtime = match &source_clone {
            RuntimeSource::DebugRoot(root) => {
                let contents = match parse_master_hhc_from_debug(root) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = set_build_status(
                            &key_clone,
                            BuildStatus {
                                phase: "error".to_string(),
                                current: 0,
                                total: 1,
                                message: "Failed parsing master.hhc".to_string(),
                                done: true,
                                success: false,
                                error: Some(e),
                                summary: None,
                            },
                        );
                        return;
                    }
                };
                let mut cb = |p: BuildProgress| {
                    let _ = update_build_status(&key_clone, |st| {
                        st.phase = p.phase;
                        st.current = p.current;
                        st.total = p.total;
                        st.message = p.message;
                    });
                };
                let entries = match parse_entries_from_debug_with_progress(root, Some(&mut cb)) {
                    Ok(v) => v,
                    Err(e) => {
                        let _ = set_build_status(
                            &key_clone,
                            BuildStatus {
                                phase: "error".to_string(),
                                current: 0,
                                total: 1,
                                message: "Failed parsing entries".to_string(),
                                done: true,
                                success: false,
                                error: Some(e),
                                summary: None,
                            },
                        );
                        return;
                    }
                };
                Arc::new(build_runtime_index(contents, entries, BTreeMap::new()))
            }
            RuntimeSource::ZipPath(zip_path) => {
                let mut cb = |p: BuildProgress| {
                    let _ = update_build_status(&key_clone, |st| {
                        st.phase = p.phase;
                        st.current = p.current;
                        st.total = p.total;
                        st.message = p.message;
                    });
                };
                match parse_runtime_from_zip_with_progress(zip_path, Some(&mut cb)) {
                    Ok(v) => Arc::new(v),
                    Err(e) => {
                        let _ = set_build_status(
                            &key_clone,
                            BuildStatus {
                                phase: "error".to_string(),
                                current: 0,
                                total: 1,
                                message: "Failed parsing zip/chm".to_string(),
                                done: true,
                                success: false,
                                error: Some(e),
                                summary: None,
                            },
                        );
                        return;
                    }
                }
            }
        };
        let summary = MasterFeatureSummary {
            debug_root: source_label(&source_clone),
            content_count: runtime.contents.len(),
            index_count: runtime.entries.len(),
        };
        let cache_result = cache_put(&source_clone, runtime);
        match cache_result {
            Ok(_) => {
                let _ = set_build_status(
                    &key_clone,
                    BuildStatus {
                        phase: "done".to_string(),
                        current: summary.index_count,
                        total: summary.index_count,
                        message: "Build complete".to_string(),
                        done: true,
                        success: true,
                        error: None,
                        summary: Some(summary),
                    },
                );
            }
            Err(e) => {
                let _ = set_build_status(
                    &key_clone,
                    BuildStatus {
                        phase: "error".to_string(),
                        current: 0,
                        total: 1,
                        message: "Cache write failed".to_string(),
                        done: true,
                        success: false,
                        error: Some(e),
                        summary: None,
                    },
                );
            }
        }
    });

    Ok(key)
}

pub(crate) fn get_master_build_status_impl(
    debug_root: Option<String>,
) -> Result<BuildStatus, String> {
    let source = resolve_runtime_source(debug_root)?;
    let key = status_key(&source);
    if let Some(st) = get_build_status_internal(&key)? {
        return Ok(st);
    }
    Ok(BuildStatus {
        phase: "idle".to_string(),
        current: 0,
        total: 1,
        message: "No build started".to_string(),
        done: true,
        success: false,
        error: None,
        summary: None,
    })
}

pub(crate) fn get_master_contents_impl(debug_root: Option<String>) -> Result<Vec<ContentItem>, String> {
    let source = resolve_runtime_source(debug_root)?;
    Ok(get_runtime(&source)?.contents.clone())
}

pub(crate) fn get_index_entries_impl(
    prefix: Option<String>,
    limit: Option<usize>,
    debug_root: Option<String>,
) -> Result<Vec<DictionaryIndexEntry>, String> {
    let source = resolve_runtime_source(debug_root)?;
    let runtime = get_runtime(&source)?;
    let p = prefix.unwrap_or_default();
    let p_key = normalize_search_key(&p);
    let p_loose = normalize_search_key_loose(&p);
    let limit = if p.is_empty() {
        limit.unwrap_or(runtime.entries.len()).clamp(1, runtime.entries.len().max(1))
    } else {
        limit.unwrap_or(200).clamp(1, 5_000)
    };

    let mut out = Vec::new();
    for (e, k) in runtime.entries.iter().zip(runtime.entry_keys.iter()) {
        if p.is_empty()
            || starts_with_search_key_precomputed(&k.headword, &k.headword_loose, &p_key, &p_loose)
        {
            out.push(DictionaryIndexEntry {
                id: e.id,
                headword: e.headword.clone(),
                aliases: e.aliases.clone(),
                source_path: e.source_path.clone(),
            });
            if out.len() >= limit {
                break;
            }
        }
    }
    Ok(out)
}

pub(crate) fn search_entries_impl(
    query: String,
    limit: Option<usize>,
    debug_root: Option<String>,
) -> Result<Vec<SearchHit>, String> {
    let q = compact_ws(&query);
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let source = resolve_runtime_source(debug_root)?;
    let runtime = get_runtime(&source)?;
    let terms = q
        .split_whitespace()
        .map(|x| (normalize_search_key(x), normalize_search_key_loose(x)))
        .collect::<Vec<_>>();
    let limit = limit.unwrap_or(50).clamp(1, 200);

    let mut hits = Vec::<SearchHit>::new();
    for (e, k) in runtime.entries.iter().zip(runtime.entry_keys.iter()) {
        let mut score = 0usize;
        let mut ok = true;
        for (t_key, t_loose) in &terms {
            let in_head = contains_search_key_precomputed(&k.headword, &k.headword_loose, t_key, t_loose)
                || k.aliases
                    .iter()
                    .zip(k.aliases_loose.iter())
                    .any(|(a, al)| contains_search_key_precomputed(a, al, t_key, t_loose));
            let in_body =
                contains_search_key_precomputed(&k.body, &k.body_loose, t_key, t_loose);
            if !(in_head || in_body) {
                ok = false;
                break;
            }
            if in_head {
                score += 5;
            }
            if in_body {
                score += 2;
            }
        }
        if ok {
            let snippet = e.definition_text.chars().take(180).collect::<String>();
            hits.push(SearchHit {
                id: e.id,
                headword: e.headword.clone(),
                source_path: e.source_path.clone(),
                score,
                snippet,
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.headword.cmp(&b.headword)));
    hits.truncate(limit);
    Ok(hits)
}

pub(crate) fn get_entry_detail_impl(id: usize, debug_root: Option<String>) -> Result<EntryDetail, String> {
    let source = resolve_runtime_source(debug_root)?;
    let runtime = get_runtime(&source)?;
    let entry = runtime
        .entries
        .iter()
        .find(|e| e.id == id)
        .cloned()
        .ok_or_else(|| format!("entry not found: {id}"))?;

    match source {
        RuntimeSource::DebugRoot(_) => Ok(entry),
        RuntimeSource::ZipPath(zip_path) => Ok(hydrate_zip_entry_detail(&zip_path, entry)),
    }
}

pub(crate) fn get_content_page_impl(
    local: String,
    source_path: Option<String>,
    debug_root: Option<String>,
) -> Result<ContentPage, String> {
    let source_path = source_path
        .unwrap_or_else(|| "master.chm".to_string())
        .to_ascii_lowercase();
    let source = resolve_runtime_source(debug_root)?;
    match &source {
        RuntimeSource::DebugRoot(root) => read_content_page_from_debug(root, &source_path, &local),
        RuntimeSource::ZipPath(zip_path) => {
            let runtime = get_runtime(&source)?;
            if source_path == "master.chm" {
                if let Some(v) = runtime
                    .content_pages
                    .get(&local)
                    .cloned()
                {
                    return Ok(v);
                }
            }
            read_content_page_from_zip(zip_path, &source_path, &local)
        }
    }
}

fn extract_chm_name(prefix: &str) -> Option<String> {
    let lower = prefix.to_ascii_lowercase();
    let end = lower.find(".chm")? + 4;
    let upto = &prefix[..end];
    let name = upto
        .rsplit([':', '/', '\\'])
        .next()
        .map(str::trim)
        .unwrap_or_default();
    if name.is_empty() {
        return None;
    }
    Some(name.to_ascii_lowercase())
}

fn parse_internal_ref(raw_ref: &str) -> Option<(Option<String>, String, bool)> {
    let raw = raw_ref.trim();
    if raw.is_empty() {
        return None;
    }
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("javascript:")
        || lower.starts_with('#')
        || lower.starts_with("data:")
    {
        return None;
    }

    let mut value = raw;
    let mut source_override = None;
    let mut is_absolute = false;
    if let Some((left, right)) = raw.split_once("::/") {
        source_override = extract_chm_name(left);
        value = right;
        is_absolute = true;
    }

    if value.starts_with('/') {
        is_absolute = true;
    }
    let value = value
        .split(['#', '?'])
        .next()
        .unwrap_or(value)
        .replace('\\', "/");
    let value = value.trim().trim_start_matches('/');
    let value = value.trim_start_matches("./");
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some((source_override, value.to_string(), is_absolute))
}

fn normalize_path(input: &str) -> String {
    let mut out = Vec::<&str>::new();
    for seg in input.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            out.pop();
            continue;
        }
        out.push(seg);
    }
    out.join("/")
}

fn resolve_relative_local(local: &str, current_local: Option<&str>, is_absolute: bool) -> String {
    if is_absolute {
        return normalize_path(local);
    }
    if let Some(base) = current_local {
        if let Some((dir, _)) = base.replace('\\', "/").rsplit_once('/') {
            return normalize_path(&format!("{dir}/{local}"));
        }
    }
    normalize_path(local)
}

fn mime_from_path(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        return "image/jpeg";
    }
    if lower.ends_with(".png") {
        return "image/png";
    }
    if lower.ends_with(".gif") {
        return "image/gif";
    }
    if lower.ends_with(".webp") {
        return "image/webp";
    }
    if lower.ends_with(".bmp") {
        return "image/bmp";
    }
    if lower.ends_with(".svg") {
        return "image/svg+xml";
    }
    if lower.ends_with(".ico") {
        return "image/x-icon";
    }
    "application/octet-stream"
}

fn read_chm_binary_object(chm: &mut chm::ChmArchive, local: &str) -> Option<Vec<u8>> {
    let path = local.trim().trim_start_matches('/');
    if path.is_empty() {
        return None;
    }
    if let Ok(v) = chm.read_object(path) {
        return Some(v);
    }
    let slash = format!("/{path}");
    if let Ok(v) = chm.read_object(&slash) {
        return Some(v);
    }
    let needle = path.to_ascii_lowercase();
    let base = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    let mut matches = chm
        .entries()
        .iter()
        .filter_map(|e| {
            let entry_lower = e.path.trim_start_matches('/').to_ascii_lowercase();
            let entry_base = entry_lower.rsplit('/').next().unwrap_or(&entry_lower);
            if entry_lower == needle || entry_base == base {
                Some(e.path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    for p in matches {
        if let Ok(v) = chm.read_object(&p) {
            return Some(v);
        }
    }
    None
}

fn read_debug_binary_object(
    root: &Path,
    source_path: &str,
    local: &str,
) -> Result<Vec<u8>, String> {
    let stem = path_stem(source_path);
    let mut candidates = Vec::new();
    candidates.push(root.join(&stem).join(local));
    candidates.push(root.join(local));
    candidates.push(root.join("master").join(local));
    for candidate in candidates {
        if candidate.exists() {
            return std::fs::read(&candidate)
                .map_err(|e| format!("failed to read {}: {e}", candidate.display()));
        }
    }
    Err(format!("debug asset not found: {source_path}::{local}"))
}

fn resolve_media_data_url_inner(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    debug_root: Option<String>,
) -> Result<String, String> {
    let Some((source_override, local_raw, is_absolute)) = parse_internal_ref(&href) else {
        return Err(format!("unsupported media href: {href}"));
    };
    let resolved_local = resolve_relative_local(&local_raw, current_local.as_deref(), is_absolute);
    let source_path = source_override
        .or(current_source_path.map(|x| x.to_ascii_lowercase()))
        .unwrap_or_else(|| "master.chm".to_string());
    let bytes = match resolve_runtime_source(debug_root)? {
        RuntimeSource::ZipPath(zip_path) => {
            let chm_bytes = read_named_chm_from_zip(&zip_path, &source_path)?;
            let mut chm = chm::ChmArchive::open(chm_bytes)
                .map_err(|e| format!("failed to open {source_path}: {e}"))?;
            read_chm_binary_object(&mut chm, &resolved_local)
                .ok_or_else(|| format!("asset not found in {source_path}: {resolved_local}"))?
        }
        RuntimeSource::DebugRoot(root) => read_debug_binary_object(&root, &source_path, &resolved_local)?,
    };
    let mime = mime_from_path(&resolved_local);
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{mime};base64,{encoded}"))
}

pub(crate) fn resolve_media_data_url_impl(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    debug_root: Option<String>,
) -> Result<String, String> {
    resolve_media_data_url_inner(href, current_source_path, current_local, debug_root)
}

pub(crate) fn resolve_link_target_impl(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    debug_root: Option<String>,
) -> Result<LinkTarget, String> {
    let Some((source_override, local_raw, is_absolute)) = parse_internal_ref(&href) else {
        return Err(format!("unsupported or empty href: {href}"));
    };
    let source_context = source_override
        .or(current_source_path.map(|x| x.to_ascii_lowercase()))
        .unwrap_or_else(|| "master.chm".to_string());
    let local_path = resolve_relative_local(&local_raw, current_local.as_deref(), is_absolute);
    let source = resolve_runtime_source(debug_root)?;
    let runtime = get_runtime(&source)?;

    let local_lower = local_path.to_ascii_lowercase();
    let local_stem_key = normalize_search_key(&path_stem(&local_path));

    if source_context == "master.chm" {
        if let Some(item) = runtime.contents.iter().find(|item| {
            let item_local = item.local.trim_start_matches('/');
            let item_lower = item_local.to_ascii_lowercase();
            if item_lower == local_lower {
                return true;
            }
            eq_search_key(&path_stem(item_local), &local_stem_key)
        }) {
            return Ok(LinkTarget::Content {
                local: item.local.clone(),
                source_path: "master.chm".to_string(),
            });
        }
    }

    if let Some(entry) = runtime.entries.iter().find(|entry| {
        entry.source_path.to_ascii_lowercase() == source_context
            && (eq_search_key(&entry.headword, &local_stem_key)
                || entry
                    .aliases
                    .iter()
                    .any(|alias| eq_search_key(alias, &local_stem_key)))
    }) {
        return Ok(LinkTarget::Entry { id: entry.id });
    }

    if let Some(entry) = runtime.entries.iter().find(|entry| {
        if eq_search_key(&entry.headword, &local_stem_key) {
            return true;
        }
        entry
            .aliases
            .iter()
            .any(|alias| eq_search_key(alias, &local_stem_key))
    }) {
        return Ok(LinkTarget::Entry { id: entry.id });
    }

    // Fallback: allow direct content open by local path even if it is not listed in master.hhc.
    // This covers many inline links such as "st_verben3.html".
    Ok(LinkTarget::Content {
        local: local_path,
        source_path: source_context,
    })
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
