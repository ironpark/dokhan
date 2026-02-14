//! ZIP-backed CHM reading and runtime index construction.
use std::collections::BTreeMap;
use std::fs;
use std::io::Cursor;
use std::path::Path;
use std::sync::mpsc;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, Instant};

use rayon::prelude::*;
use zip::ZipArchive;

use crate::chm;
use crate::app::model::{BuildProgress, ContentItem, ContentPage, EntryDetail, RuntimeIndex};
use crate::parsing::index::{extract_index_entries_from_open_chm, parse_master_hhc_text};
use crate::parsing::text::{
    compact_ws, decode_euc_kr, extract_first_bold_text, extract_html_fragments,
    sanitize_html_fragment, strip_html_tags,
};
use crate::runtime::link_media::read_chm_binary_object;
use crate::runtime::search::normalize_search_key;
use crate::runtime::state::build_runtime_index;

type ChmBytes = Arc<[u8]>;
type ZipBytes = Arc<[u8]>;

static CHM_BYTES_CACHE: OnceLock<Mutex<BTreeMap<String, ChmBytes>>> = OnceLock::new();
static ZIP_BYTES_CACHE: OnceLock<Mutex<BTreeMap<String, ZipBytes>>> = OnceLock::new();
static CHM_ARCHIVE_CACHE: OnceLock<Mutex<BTreeMap<String, Arc<chm::ChmArchive>>>> = OnceLock::new();

/// Decode CHM page bytes into normalized content payload.
fn decode_content_page(local: String, source_path: String, bytes: &[u8]) -> ContentPage {
    let text = decode_euc_kr(bytes);
    let fragments = extract_html_fragments(&text);
    let title = fragments
        .title
        .as_ref()
        .map(|x| compact_ws(&strip_html_tags(x)))
        .filter(|x| !x.is_empty())
        .unwrap_or_else(|| local.clone());
    let b_html = fragments.body_html.unwrap_or_default();
    let b_html = sanitize_html_fragment(&b_html);
    let b_text = compact_ws(&strip_html_tags(&b_html));
    ContentPage {
        local,
        source_path,
        title,
        body_text: b_text,
        body_html: b_html,
    }
}

/// Build candidate local paths for CHM object resolution.
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

/// Read CHM object with filename and basename fallback candidates.
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

fn zip_cache_prefix(zip_path: &Path) -> String {
    zip_path
        .canonicalize()
        .unwrap_or_else(|_| zip_path.to_path_buf())
        .to_string_lossy()
        .to_string()
}

fn chm_basename_lower(name: &str) -> String {
    name.rsplit(['/', '\\'])
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

fn chm_cache_key(zip_path: &Path, chm_name: &str) -> String {
    format!("{}::{}", zip_cache_prefix(zip_path), chm_basename_lower(chm_name))
}

fn get_cached_chm_bytes(zip_path: &Path, chm_name: &str) -> Result<Option<ChmBytes>, String> {
    let cache = CHM_BYTES_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let guard = cache.lock().map_err(|_| "chm cache lock poisoned".to_string())?;
    Ok(guard.get(&chm_cache_key(zip_path, chm_name)).cloned())
}

fn cache_chm_bytes(zip_path: &Path, chm_name: &str, bytes: ChmBytes) -> Result<(), String> {
    let cache = CHM_BYTES_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = cache.lock().map_err(|_| "chm cache lock poisoned".to_string())?;
    guard.insert(chm_cache_key(zip_path, chm_name), bytes);
    Ok(())
}

fn get_cached_chm_archive(zip_path: &Path, chm_name: &str) -> Result<Option<chm::ChmArchive>, String> {
    let cache = CHM_ARCHIVE_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let guard = cache
        .lock()
        .map_err(|_| "chm archive cache lock poisoned".to_string())?;
    Ok(guard
        .get(&chm_cache_key(zip_path, chm_name))
        .map(|arch| (**arch).clone()))
}

fn cache_chm_archive(zip_path: &Path, chm_name: &str, archive: chm::ChmArchive) -> Result<(), String> {
    let cache = CHM_ARCHIVE_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = cache
        .lock()
        .map_err(|_| "chm archive cache lock poisoned".to_string())?;
    guard.insert(chm_cache_key(zip_path, chm_name), Arc::new(archive));
    Ok(())
}

fn get_zip_bytes(zip_path: &Path) -> Result<ZipBytes, String> {
    let key = zip_cache_prefix(zip_path);
    let cache = ZIP_BYTES_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    {
        let guard = cache.lock().map_err(|_| "zip cache lock poisoned".to_string())?;
        if let Some(found) = guard.get(&key) {
            return Ok(Arc::clone(found));
        }
    }

    let bytes = fs::read(zip_path).map_err(|e| format!("failed to read zip file: {e}"))?;
    let shared: ZipBytes = Arc::from(bytes.into_boxed_slice());
    let mut guard = cache.lock().map_err(|_| "zip cache lock poisoned".to_string())?;
    guard.insert(key, Arc::clone(&shared));
    Ok(shared)
}

fn open_zip_archive_from_memory(zip_path: &Path) -> Result<ZipArchive<Cursor<ZipBytes>>, String> {
    let zip_bytes = get_zip_bytes(zip_path)?;
    ZipArchive::new(Cursor::new(zip_bytes)).map_err(|e| format!("failed to open zip: {e}"))
}

/// Read raw CHM file bytes from dataset ZIP by filename.
///
/// # Errors
///
/// Returns an error when the ZIP cannot be opened/read or the named CHM does not exist.
pub(crate) fn read_named_chm_from_zip(zip_path: &Path, chm_name: &str) -> Result<ChmBytes, String> {
    if let Some(cached) = get_cached_chm_bytes(zip_path, chm_name)? {
        return Ok(cached);
    }

    let mut archive = open_zip_archive_from_memory(zip_path)?;
    let target = chm_basename_lower(chm_name);
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry #{i}: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let entry_base = chm_basename_lower(entry.name());
        if entry_base != target {
            continue;
        }
        let mut bytes = Vec::new();
        std::io::copy(&mut entry, &mut bytes)
            .map_err(|e| format!("failed to load {chm_name} from zip: {e}"))?;
        let shared: ChmBytes = Arc::from(bytes.into_boxed_slice());
        let _ = cache_chm_bytes(zip_path, &entry_base, Arc::clone(&shared));
        return Ok(shared);
    }

    Err(format!("chm not found in zip: {chm_name}"))
}

/// Open CHM archive from ZIP with parsed-archive template cache.
///
/// # Errors
///
/// Returns an error when CHM bytes cannot be loaded or archive parsing fails.
pub(crate) fn open_named_chm_from_zip(zip_path: &Path, chm_name: &str) -> Result<chm::ChmArchive, String> {
    if let Some(arch) = get_cached_chm_archive(zip_path, chm_name)? {
        return Ok(arch);
    }
    let bytes = read_named_chm_from_zip(zip_path, chm_name)?;
    let archive = chm::ChmArchive::open(bytes).map_err(|e| format!("failed to open {chm_name}: {e}"))?;
    let _ = cache_chm_archive(zip_path, chm_name, archive.clone());
    Ok(archive)
}

/// Resolve entry HTML bytes using target local or headword fallback.
fn read_entry_html_from_chm(
    chm: &mut chm::ChmArchive,
    headword: &str,
    by_stem: Option<&BTreeMap<String, Vec<String>>>,
) -> Option<Vec<u8>> {
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
    let matches = if let Some(indexed) = by_stem {
        indexed.get(&wanted).cloned().unwrap_or_default()
    } else {
        let mut scanned = chm
            .entries()
            .iter()
            .filter_map(|e| {
                let base = e.path.rsplit('/').next().unwrap_or(&e.path);
                let stem = base.rsplit_once('.').map_or(base, |(stem, _)| stem);
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
        scanned.sort();
        scanned.dedup();
        scanned
    };
    for path in matches {
        if let Ok(v) = chm.read_object(&path) {
            return Some(v);
        }
    }
    None
}

fn build_html_path_index(chm: &chm::ChmArchive) -> BTreeMap<String, Vec<String>> {
    let mut by_stem = BTreeMap::<String, Vec<String>>::new();
    for entry in chm.entries() {
        let base = entry.path.rsplit('/').next().unwrap_or(&entry.path);
        let Some((stem, ext)) = base.rsplit_once('.') else {
            continue;
        };
        let ext = ext.to_ascii_lowercase();
        if ext != "htm" && ext != "html" {
            continue;
        }
        by_stem
            .entry(normalize_search_key(stem))
            .or_default()
            .push(entry.path.clone());
    }
    for paths in by_stem.values_mut() {
        paths.sort();
        paths.dedup();
    }
    by_stem
}

fn hydrate_entries_from_open_chm(chm: &mut chm::ChmArchive, entries: &mut [EntryDetail]) {
    let path_index = build_html_path_index(&chm);
    for entry in entries.iter_mut() {
        let html_bytes = if entry.target_local.is_empty() {
            read_entry_html_from_chm(chm, &entry.headword, Some(&path_index))
        } else {
            read_chm_binary_object(chm, &entry.target_local)
                .or_else(|| read_entry_html_from_chm(chm, &entry.headword, Some(&path_index)))
        };
        let Some(html_bytes) = html_bytes else {
            continue;
        };

        let html_text = decode_euc_kr(&html_bytes);
        let fragments = extract_html_fragments(&html_text);
        let paragraph_html = fragments.first_paragraph_html.unwrap_or_default();
        let paragraph_text = compact_ws(&strip_html_tags(&paragraph_html));
        let body = fragments.body_html.unwrap_or_default();
        let body_text = compact_ws(&strip_html_tags(&body));

        if !paragraph_html.is_empty() {
            entry.definition_html = sanitize_html_fragment(&paragraph_html);
        } else if !body.is_empty() {
            entry.definition_html = sanitize_html_fragment(&body);
        }
        if !paragraph_text.is_empty() {
            entry.definition_text = paragraph_text;
        } else if !body_text.is_empty() {
            entry.definition_text = body_text;
        }

        if let Some(title_alias) = fragments
            .title
            .as_ref()
            .map(|x| compact_ws(&strip_html_tags(x)))
            .filter(|x| !x.is_empty())
        {
            if !entry.aliases.contains(&title_alias) {
                entry.aliases.push(title_alias);
            }
        }
        if let Some(bold) = extract_first_bold_text(&html_text) {
            let bold = compact_ws(&bold);
            if !bold.is_empty() && !entry.aliases.contains(&bold) {
                entry.aliases.push(bold);
            }
        }
    }
}

fn finalize_entries(mut entries: Vec<EntryDetail>) -> Vec<EntryDetail> {
    let mut keyed = entries
        .drain(..)
        .map(|entry| {
            let headword_key = normalize_search_key(&entry.headword);
            let source_key = entry.source_path.clone();
            let local_key = entry.target_local.clone();
            (headword_key, source_key, local_key, entry)
        })
        .collect::<Vec<_>>();

    keyed.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    keyed.dedup_by(|a, b| a.0 == b.0 && a.1 == b.1 && a.2 == b.2);

    let mut out = Vec::with_capacity(keyed.len());
    for (i, (_, _, _, mut entry)) in keyed.into_iter().enumerate() {
        entry.id = i + 1;
        out.push(entry);
    }
    out
}

fn recommended_parse_threads(task_count: usize) -> usize {
    let available = std::thread::available_parallelism()
        .map(|n| n.get())
        .unwrap_or(1);
    let capped = if cfg!(target_os = "android") {
        available.min(4)
    } else {
        available
    };
    capped.min(task_count.max(1))
}

fn emit_progress_throttled(
    progress: &mut Option<&mut dyn FnMut(BuildProgress)>,
    last_emit: &mut Instant,
    interval: Duration,
    payload: BuildProgress,
    force: bool,
) {
    if !force && last_emit.elapsed() < interval {
        return;
    }
    if let Some(cb) = progress.as_mut() {
        cb(payload);
    }
    *last_emit = Instant::now();
}

/// Fill empty entry body fields by reading original CHM HTML.
pub(crate) fn hydrate_zip_entry_detail(zip_path: &Path, mut entry: EntryDetail) -> EntryDetail {
    if !entry.definition_text.is_empty() {
        return entry;
    }
    let Ok(mut chm) = open_named_chm_from_zip(zip_path, &entry.source_path) else {
        return entry;
    };
    let html_bytes = if entry.target_local.is_empty() {
        read_entry_html_from_chm(&mut chm, &entry.headword, None)
    } else {
        read_chm_binary_object(&mut chm, &entry.target_local)
            .or_else(|| read_entry_html_from_chm(&mut chm, &entry.headword, None))
    };
    let Some(html_bytes) = html_bytes else {
        return entry;
    };

    let html_text = decode_euc_kr(&html_bytes);
    let fragments = extract_html_fragments(&html_text);
    let paragraph_html = fragments.first_paragraph_html.unwrap_or_default();
    let paragraph_text = compact_ws(&strip_html_tags(&paragraph_html));
    let body = fragments.body_html.unwrap_or_default();
    let body_text = compact_ws(&strip_html_tags(&body));
    if !paragraph_html.is_empty() {
        entry.definition_html = sanitize_html_fragment(&paragraph_html);
    } else if !body.is_empty() {
        entry.definition_html = sanitize_html_fragment(&body);
    }
    if !paragraph_text.is_empty() {
        entry.definition_text = paragraph_text;
    } else if !body_text.is_empty() {
        entry.definition_text = body_text;
    }

    if let Some(title_alias) = fragments
        .title
        .as_ref()
        .map(|x| compact_ws(&strip_html_tags(x)))
        .filter(|x| !x.is_empty())
    {
        if !entry.aliases.contains(&title_alias) {
            entry.aliases.push(title_alias);
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

/// Read and decode content page from ZIP-contained CHM.
///
/// # Errors
///
/// Returns an error when the CHM cannot be loaded/opened or the target page cannot be resolved.
pub(crate) fn read_content_page_from_zip(
    zip_path: &Path,
    source_path: &str,
    local: &str,
) -> Result<ContentPage, String> {
    let mut chm = open_named_chm_from_zip(zip_path, source_path)?;
    if let Some(v) = read_chm_object_with_candidates(&mut chm, local) {
        return Ok(decode_content_page(local.to_string(), source_path.to_string(), &v));
    }
    Err(format!(
        "content page not found in zip runtime: {source_path}::{local}"
    ))
}

/// Parse full runtime index from ZIP and emit progress events.
///
/// The callback receives best-effort progress snapshots during CHM iteration.
///
/// # Errors
///
/// Returns an error when ZIP/CHM reading fails during runtime construction.
pub(crate) fn parse_runtime_from_zip_with_progress(
    zip_path: &Path,
    mut progress: Option<&mut dyn FnMut(BuildProgress)>,
) -> Result<RuntimeIndex, String> {
    let mut archive = open_zip_archive_from_memory(zip_path)?;
    let total = archive.len();
    let mut contents = Vec::<ContentItem>::new();
    let mut merge_chms = Vec::<(String, ChmBytes)>::new();
    let mut progress_last_emit = Instant::now();
    let progress_interval = Duration::from_millis(120);

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
        let shared: ChmBytes = Arc::from(bytes.into_boxed_slice());
        let entry_base = chm_basename_lower(&name);
        let _ = cache_chm_bytes(zip_path, &entry_base, Arc::clone(&shared));

        if lower.ends_with("master.chm") {
            if let Ok(mut chm) = chm::ChmArchive::open(Arc::clone(&shared)) {
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

        emit_progress_throttled(
            &mut progress,
            &mut progress_last_emit,
            progress_interval,
            BuildProgress {
                phase: "scan".to_string(),
                current: i + 1,
                total,
                message: format!("Scanning {name}"),
            },
            false,
        );

        if lower.starts_with("merge") {
            merge_chms.push((name, shared));
        }
    }
    let parse_total = merge_chms.len();
    let parse_threads = recommended_parse_threads(parse_total);
    emit_progress_throttled(
        &mut progress,
        &mut progress_last_emit,
        progress_interval,
        BuildProgress {
            phase: "parse".to_string(),
            current: 0,
            total: parse_total.max(1),
            message: format!(
                "Parsing {} CHM files (multithreaded, {} threads)",
                parse_total, parse_threads
            ),
        },
        true,
    );

    let (tx, rx) = mpsc::channel::<usize>();
    let worker = std::thread::spawn(move || {
        let pool = rayon::ThreadPoolBuilder::new()
            .num_threads(parse_threads)
            .build()
            .map_err(|e| format!("failed to build parse thread pool: {e}"))?;
        let rows = pool.install(|| {
            merge_chms
                .into_par_iter()
                .map(|(name, bytes)| {
                    let Ok(mut chm) = chm::ChmArchive::open(bytes) else {
                        let _ = tx.send(1);
                        return Vec::new();
                    };
                    let mut parsed = extract_index_entries_from_open_chm(&name, &mut chm);
                    hydrate_entries_from_open_chm(&mut chm, &mut parsed);
                    let _ = tx.send(1);
                    parsed
                })
                .collect::<Vec<_>>()
        });
        Ok::<Vec<Vec<EntryDetail>>, String>(rows)
    });

    let mut parsed_done = 0usize;
    while parsed_done < parse_total {
        match rx.recv_timeout(Duration::from_millis(120)) {
            Ok(done) => {
                parsed_done = (parsed_done + done).min(parse_total);
                emit_progress_throttled(
                    &mut progress,
                    &mut progress_last_emit,
                    progress_interval,
                    BuildProgress {
                        phase: "parse".to_string(),
                        current: parsed_done,
                        total: parse_total.max(1),
                        message: format!("Parsed {parsed_done}/{parse_total} CHM files"),
                    },
                    false,
                );
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {}
            Err(mpsc::RecvTimeoutError::Disconnected) => break,
        }
    }

    let parsed_chunks = worker
        .join()
        .map_err(|_| "multithreaded parse worker panicked".to_string())??;

    let mut entries = Vec::<EntryDetail>::new();
    for mut chunk in parsed_chunks {
        entries.append(&mut chunk);
    }

    emit_progress_throttled(
        &mut progress,
        &mut progress_last_emit,
        progress_interval,
        BuildProgress {
            phase: "parse".to_string(),
            current: parse_total.max(1),
            total: parse_total.max(1),
            message: "Completed multithreaded parse".to_string(),
        },
        true,
    );

    let entries = finalize_entries(entries);
    Ok(build_runtime_index(contents, entries, BTreeMap::new()))
}
