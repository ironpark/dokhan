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
    compact_html_text, compact_ws, decode_euc_kr, extract_first_bold_text,
    extract_first_bold_text_from_paragraph, extract_html_fragments,
    sanitize_html_fragment, strip_html_tags,
};
use crate::runtime::cache::{keyed_slot, BoundedCache, KeyedSlots};
use crate::runtime::link_media::read_chm_binary_object;
use crate::runtime::search::normalize_search_key;
use crate::runtime::state::build_runtime_index;

type ChmBytes = Arc<[u8]>;
type ZipBytes = Arc<[u8]>;

static CHM_BYTES_CACHE: OnceLock<Mutex<BoundedCache<ChmBytes>>> = OnceLock::new();
static ZIP_BYTES_CACHE: OnceLock<Mutex<BoundedCache<ZipBytes>>> = OnceLock::new();
static ZIP_LOAD_LOCKS: KeyedSlots<()> = OnceLock::new();
type SharedChmArchive = Arc<Mutex<chm::ChmArchive>>;
static CHM_ARCHIVE_CACHE: OnceLock<Mutex<BoundedCache<SharedChmArchive>>> = OnceLock::new();
static CHM_OPEN_LOCKS: KeyedSlots<()> = OnceLock::new();
const MAX_CHM_BYTES_CACHE_ITEMS: usize = 24;
const MAX_CHM_ARCHIVE_CACHE_ITEMS: usize = 16;
const MAX_ZIP_BYTES_CACHE_ITEMS: usize = 2;

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
    let cache = CHM_BYTES_CACHE
        .get_or_init(|| Mutex::new(BoundedCache::new(MAX_CHM_BYTES_CACHE_ITEMS)));
    let mut guard = cache.lock().map_err(|_| "chm cache lock poisoned".to_string())?;
    Ok(guard.get(&chm_cache_key(zip_path, chm_name)).cloned())
}

fn cache_chm_bytes(zip_path: &Path, chm_name: &str, bytes: ChmBytes) -> Result<(), String> {
    let cache = CHM_BYTES_CACHE
        .get_or_init(|| Mutex::new(BoundedCache::new(MAX_CHM_BYTES_CACHE_ITEMS)));
    let mut guard = cache.lock().map_err(|_| "chm cache lock poisoned".to_string())?;
    let key = chm_cache_key(zip_path, chm_name);
    guard.insert(key, bytes);
    Ok(())
}

fn get_cached_chm_archive(
    zip_path: &Path,
    chm_name: &str,
) -> Result<Option<SharedChmArchive>, String> {
    let cache = CHM_ARCHIVE_CACHE
        .get_or_init(|| Mutex::new(BoundedCache::new(MAX_CHM_ARCHIVE_CACHE_ITEMS)));
    let mut guard = cache
        .lock()
        .map_err(|_| "chm archive cache lock poisoned".to_string())?;
    Ok(guard.get(&chm_cache_key(zip_path, chm_name)).cloned())
}

fn cache_chm_archive(
    zip_path: &Path,
    chm_name: &str,
    archive: SharedChmArchive,
) -> Result<(), String> {
    let cache = CHM_ARCHIVE_CACHE
        .get_or_init(|| Mutex::new(BoundedCache::new(MAX_CHM_ARCHIVE_CACHE_ITEMS)));
    let mut guard = cache
        .lock()
        .map_err(|_| "chm archive cache lock poisoned".to_string())?;
    let key = chm_cache_key(zip_path, chm_name);
    guard.insert(key, archive);
    Ok(())
}

fn cached_zip_bytes(key: &str) -> Result<Option<ZipBytes>, String> {
    let cache = ZIP_BYTES_CACHE
        .get_or_init(|| Mutex::new(BoundedCache::new(MAX_ZIP_BYTES_CACHE_ITEMS)));
    let mut guard = cache.lock().map_err(|_| "zip cache lock poisoned".to_string())?;
    Ok(guard.get(key).map(Arc::clone))
}

fn get_zip_bytes(zip_path: &Path) -> Result<ZipBytes, String> {
    let key = zip_cache_prefix(zip_path);
    if let Some(found) = cached_zip_bytes(&key)? {
        return Ok(found);
    }

    // Serialize loads per ZIP so concurrent misses read the file once,
    // without holding the shared cache lock during the read.
    let load_lock = keyed_slot(&ZIP_LOAD_LOCKS, &key)?;
    let _load_guard = load_lock
        .lock()
        .map_err(|_| "zip load lock poisoned".to_string())?;
    if let Some(found) = cached_zip_bytes(&key)? {
        return Ok(found);
    }

    let bytes = fs::read(zip_path).map_err(|e| format!("failed to read zip file: {e}"))?;
    let shared: ZipBytes = Arc::from(bytes.into_boxed_slice());
    let cache = ZIP_BYTES_CACHE
        .get_or_init(|| Mutex::new(BoundedCache::new(MAX_ZIP_BYTES_CACHE_ITEMS)));
    cache
        .lock()
        .map_err(|_| "zip cache lock poisoned".to_string())?
        .insert(key, Arc::clone(&shared));
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
pub(crate) fn open_named_chm_from_zip(
    zip_path: &Path,
    chm_name: &str,
) -> Result<SharedChmArchive, String> {
    if let Some(arch) = get_cached_chm_archive(zip_path, chm_name)? {
        return Ok(arch);
    }
    let key = chm_cache_key(zip_path, chm_name);
    let open_lock = keyed_slot(&CHM_OPEN_LOCKS, &key)?;
    let _open_guard = open_lock
        .lock()
        .map_err(|_| "chm open lock poisoned".to_string())?;
    if let Some(arch) = get_cached_chm_archive(zip_path, chm_name)? {
        return Ok(arch);
    }
    let bytes = read_named_chm_from_zip(zip_path, chm_name)?;
    let archive = Arc::new(Mutex::new(
        chm::ChmArchive::open(bytes).map_err(|e| format!("failed to open {chm_name}: {e}"))?,
    ));
    cache_chm_archive(zip_path, chm_name, Arc::clone(&archive))?;
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
        let candidate_lower = candidate.to_ascii_lowercase();
        if by_stem.is_none()
            && (candidate_lower.ends_with(".htm") || candidate_lower.ends_with(".html"))
        {
            if let Some(v) = chm.read_object_by_basename(&candidate) {
                return Some(v);
            }
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
    let path_index = build_html_path_index(chm);
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
        let paragraph_text = compact_html_text(&paragraph_html);
        let body = fragments.body_html.unwrap_or_default();

        // Search needs plain text now; HTML is sanitized when a detail is opened.
        if !paragraph_text.is_empty() {
            entry.definition_text = paragraph_text;
        } else if !body.is_empty() {
            let body_text = compact_html_text(&body);
            if !body_text.is_empty() {
                entry.definition_text = body_text;
            }
        }

        if let Some(title_alias) = fragments
            .title
            .as_ref()
            .map(|x| compact_html_text(x))
            .filter(|x| !x.is_empty())
        {
            if !entry.aliases.contains(&title_alias) {
                entry.aliases.push(title_alias);
            }
        }
        if let Some(bold) = extract_first_bold_text_from_paragraph(&paragraph_html) {
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

/// Fill entry HTML on demand by reading the original CHM page.
pub(crate) fn hydrate_zip_entry_detail(zip_path: &Path, mut entry: EntryDetail) -> EntryDetail {
    if !entry.definition_text.is_empty() && !entry.definition_html.is_empty() {
        return entry;
    }
    let Ok(chm) = open_named_chm_from_zip(zip_path, &entry.source_path) else {
        return entry;
    };
    let Ok(mut chm) = chm.lock() else {
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
    drop(chm);

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
    let chm = open_named_chm_from_zip(zip_path, source_path)?;
    let mut chm = chm.lock().map_err(|_| "chm archive lock poisoned".to_string())?;
    let bytes = read_chm_object_with_candidates(&mut chm, local);
    drop(chm);
    if let Some(v) = bytes {
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

#[cfg(test)]
mod tests {
    use super::{extract_index_entries_from_open_chm, hydrate_entries_from_open_chm, hydrate_zip_entry_detail, open_named_chm_from_zip, read_entry_html_from_chm};
    use crate::chm::ChmArchive;
    use crate::parsing::text::{compact_ws, decode_euc_kr, extract_first_bold_text, extract_html_fragments, sanitize_html_fragment, strip_html_tags};
    use std::io::Read;
    use std::path::Path;
    use std::sync::{Arc, Barrier};
    use std::time::Instant;

    #[test]
    fn concurrent_opens_share_one_chm_archive_if_configured() {
        let Ok(path) = std::env::var("DOKHAN_TEST_ZIP") else {
            return;
        };
        let barrier = Arc::new(Barrier::new(8));
        let workers = (0..8)
            .map(|_| {
                let path = path.clone();
                let barrier = Arc::clone(&barrier);
                std::thread::spawn(move || {
                    barrier.wait();
                    open_named_chm_from_zip(Path::new(&path), "merge17.chm").expect("open CHM")
                })
            })
            .collect::<Vec<_>>();
        let archives = workers
            .into_iter()
            .map(|worker| worker.join().expect("worker"))
            .collect::<Vec<_>>();
        assert!(archives.iter().all(|archive| Arc::ptr_eq(archive, &archives[0])));
    }

    #[test]
    fn entry_html_is_loaded_on_demand_if_configured() {
        let Ok(path) = std::env::var("DOKHAN_TEST_ZIP") else {
            return;
        };
        let file = std::fs::File::open(&path).expect("open benchmark zip");
        let mut zip = zip::ZipArchive::new(file).expect("parse benchmark zip");
        let mut bytes = Vec::new();
        zip.by_name("merge17.chm")
            .expect("CHM in benchmark zip")
            .read_to_end(&mut bytes)
            .expect("read CHM bytes");
        let mut chm = ChmArchive::open(bytes).expect("open CHM");
        let mut entries = extract_index_entries_from_open_chm("merge17.chm", &mut chm);
        hydrate_entries_from_open_chm(&mut chm, &mut entries);
        let (entry, expected_html) = entries.into_iter()
            .filter(|entry| !entry.definition_text.is_empty())
            .find_map(|entry| {
                let bytes = read_entry_html_from_chm(&mut chm, &entry.headword, None)?;
                let text = decode_euc_kr(&bytes);
                let fragments = extract_html_fragments(&text);
                let html = fragments.first_paragraph_html.or(fragments.body_html)?;
                let clean = sanitize_html_fragment(&html);
                (!clean.is_empty()).then_some((entry, clean))
            })
            .expect("entry with renderable HTML");
        assert!(entry.definition_html.is_empty());
        let detail = hydrate_zip_entry_detail(Path::new(&path), entry.clone());
        assert_eq!(detail.definition_html, expected_html);
        assert_eq!(detail.definition_text, entry.definition_text);
    }

    #[test]
    #[ignore]
    fn bench_chm_extract_and_hydrate() {
        let path = std::env::var("DOKHAN_TEST_ZIP").expect("set DOKHAN_TEST_ZIP");
        let file = std::fs::File::open(path).expect("open benchmark zip");
        let mut zip = zip::ZipArchive::new(file).expect("parse benchmark zip");
        for name in ["merge17.chm", "merge36.chm", "merge01.chm"] {
            let mut bytes = Vec::new();
            zip.by_name(name)
                .expect("CHM in benchmark zip")
                .read_to_end(&mut bytes)
                .expect("read CHM bytes");
            let start = Instant::now();
            let mut chm = ChmArchive::open(bytes).expect("open CHM");
            let open_time = start.elapsed();
            let start = Instant::now();
            let mut entries = extract_index_entries_from_open_chm(name, &mut chm);
            let extract_time = start.elapsed();
            let start = Instant::now();
            hydrate_entries_from_open_chm(&mut chm, &mut entries);
            let hydrate_time = start.elapsed();
            eprintln!(
                "CHM {name}: open={open_time:?}, extract={extract_time:?}, hydrate={hydrate_time:?}, entries={}",
                entries.len()
            );
        }
    }

    #[test]
    #[ignore]
    fn bench_chm_html_stages() {
        let path = std::env::var("DOKHAN_TEST_ZIP").expect("set DOKHAN_TEST_ZIP");
        let file = std::fs::File::open(path).expect("open benchmark zip");
        let mut zip = zip::ZipArchive::new(file).expect("parse benchmark zip");
        let mut bytes = Vec::new();
        zip.by_name("merge17.chm")
            .expect("CHM in benchmark zip")
            .read_to_end(&mut bytes)
            .expect("read CHM bytes");
        let mut chm = ChmArchive::open(bytes).expect("open CHM");
        let paths = chm.entries().iter()
            .filter(|entry| entry.path.ends_with(".html") || entry.path.ends_with(".htm"))
            .take(100)
            .map(|entry| entry.path.clone())
            .collect::<Vec<_>>();
        let (mut read_time, mut decode_time, mut extract_time, mut sanitize_time, mut text_time, mut bold_time) =
            (std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO,
             std::time::Duration::ZERO, std::time::Duration::ZERO, std::time::Duration::ZERO);
        for path in &paths {
            let start = Instant::now();
            let bytes = chm.read_object(path).expect("read page");
            read_time += start.elapsed();
            let start = Instant::now();
            let text = decode_euc_kr(&bytes);
            decode_time += start.elapsed();
            let start = Instant::now();
            let fragments = extract_html_fragments(&text);
            extract_time += start.elapsed();
            let paragraph = fragments.first_paragraph_html.unwrap_or_default();
            let start = Instant::now();
            let _ = sanitize_html_fragment(&paragraph);
            sanitize_time += start.elapsed();
            let start = Instant::now();
            let _ = compact_ws(&strip_html_tags(&paragraph));
            text_time += start.elapsed();
            let start = Instant::now();
            let _ = extract_first_bold_text(&text);
            bold_time += start.elapsed();
        }
        eprintln!("CHM HTML stages for {} pages: read={read_time:?}, decode={decode_time:?}, extract={extract_time:?}, sanitize={sanitize_time:?}, strip={text_time:?}, bold={bold_time:?}", paths.len());
    }
}
