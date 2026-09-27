//! Persistent storage for managed ZIP files and runtime index caches.
use std::collections::BTreeSet;
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use tauri::Manager;

use crate::app::model::{ContentItem, EntryDetail, RuntimeSource};

const MANAGED_ZIP_DIR: &str = "zips";
const LAST_SUCCESSFUL_ZIP_FILE: &str = "last-successful-zip.txt";
const FAILED_MANAGED_ZIPS_FILE: &str = "failed-managed-zips.txt";
const RUNTIME_CACHE_DIR: &str = "runtime-cache";
const SEARCH_INDEX_DIR: &str = "tantivy-v0.26";
const RUNTIME_CACHE_VERSION: u32 = 1;
const CACHE_MANIFEST_FILE: &str = "manifest.bin";
const CACHE_CONTENTS_FILE: &str = "contents.bin.zst";
const CACHE_ENTRIES_FILE: &str = "entries.bin.zst";
const ZSTD_LEVEL: i32 = 3;
static MANAGED_ZIP_STATE_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct PersistedRuntime {
    pub(crate) contents: Vec<ContentItem>,
    pub(crate) entries: Vec<EntryDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
struct RuntimeCacheManifest {
    version: u32,
    contents_count: usize,
    entries_count: usize,
}

fn fnv1a64(input: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for b in input {
        hash ^= *b as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn sanitize_name(name: &str) -> String {
    let mut out = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() || ch == '-' || ch == '_' {
            out.push(ch);
        } else {
            out.push('_');
        }
    }
    if out.is_empty() {
        "dataset".to_string()
    } else {
        out
    }
}

fn source_fingerprint(path: &Path) -> Result<String, String> {
    let canonical = path
        .canonicalize()
        .unwrap_or_else(|_| path.to_path_buf())
        .to_string_lossy()
        .to_string();
    let meta = fs::metadata(path).map_err(|e| format!("failed to stat source zip: {e}"))?;
    let mtime = meta
        .modified()
        .ok()
        .and_then(|t| t.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let payload = format!("{canonical}:{}:{mtime}", meta.len());
    Ok(format!("{:016x}", fnv1a64(payload.as_bytes())))
}

fn managed_root(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    app.path()
        .app_cache_dir()
        .map_err(|e| format!("failed to resolve app cache dir: {e}"))
}

fn managed_zip_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = managed_root(app)?.join(MANAGED_ZIP_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create managed zip dir: {e}"))?;
    Ok(dir)
}

fn is_managed_zip_file(dir: &Path, path: &Path) -> bool {
    if !path.is_file()
        || !path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
    {
        return false;
    }
    match (dir.canonicalize(), path.canonicalize()) {
        (Ok(root), Ok(file)) => file.parent() == Some(root.as_path()),
        _ => false,
    }
}

fn is_managed_zip_name(name: &str) -> bool {
    !name.is_empty()
        && !name.contains('/')
        && !name.contains('\\')
        && Path::new(name).file_name().and_then(|value| value.to_str()) == Some(name)
        && Path::new(name)
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("zip"))
}

fn failed_managed_zip_names(dir: &Path) -> BTreeSet<String> {
    fs::read_to_string(dir.join(FAILED_MANAGED_ZIPS_FILE))
        .unwrap_or_default()
        .lines()
        .filter(|name| is_managed_zip_name(name))
        .map(str::to_string)
        .collect()
}

fn save_failed_managed_zip_names(dir: &Path, names: &BTreeSet<String>) -> Result<(), String> {
    let contents = names.iter().cloned().collect::<Vec<_>>().join("\n");
    write_atomic(&dir.join(FAILED_MANAGED_ZIPS_FILE), contents.as_bytes())
}

fn remembered_managed_zip_in_dir(dir: &Path) -> Option<PathBuf> {
    let name = fs::read_to_string(dir.join(LAST_SUCCESSFUL_ZIP_FILE)).ok()?;
    let name = name.trim();
    if !is_managed_zip_name(name) || failed_managed_zip_names(dir).contains(name) {
        return None;
    }
    let path = dir.join(name);
    is_managed_zip_file(dir, &path).then_some(path)
}

fn latest_managed_zip_in_dir(dir: &Path) -> Result<Option<PathBuf>, String> {
    let mut best: Option<(std::time::SystemTime, PathBuf)> = None;
    let failed = failed_managed_zip_names(dir);

    let iter = fs::read_dir(dir).map_err(|e| format!("failed to read managed zip dir: {e}"))?;
    for entry in iter {
        let entry = entry.map_err(|e| format!("failed to read managed zip entry: {e}"))?;
        let path = entry.path();
        if !is_managed_zip_file(dir, &path)
            || path
                .file_name()
                .and_then(|name| name.to_str())
                .is_some_and(|name| failed.contains(name))
        {
            continue;
        }
        let mtime = match entry.metadata().and_then(|m| m.modified()) {
            Ok(t) => t,
            Err(_) => std::time::SystemTime::UNIX_EPOCH,
        };
        match &best {
            Some((best_time, _)) if *best_time >= mtime => {}
            _ => best = Some((mtime, path)),
        }
    }

    Ok(best.map(|(_, p)| p))
}

fn preferred_managed_zip_in_dir(dir: &Path) -> Result<Option<PathBuf>, String> {
    if let Some(remembered) = remembered_managed_zip_in_dir(dir) {
        return Ok(Some(remembered));
    }
    latest_managed_zip_in_dir(dir)
}

/// Pick the last successfully built ZIP, falling back to the newest managed ZIP.
///
/// # Errors
///
/// Returns an error when the managed ZIP directory cannot be resolved or read.
pub(crate) fn preferred_managed_zip(app: &tauri::AppHandle) -> Result<Option<PathBuf>, String> {
    let dir = managed_zip_dir(app)?;
    preferred_managed_zip_in_dir(&dir)
}

/// Persist the ZIP selected by a successful runtime build.
///
/// # Errors
///
/// Returns an error when the source is outside the managed ZIP directory or the pointer cannot be written.
pub(crate) fn remember_successful_managed_zip(
    app: &tauri::AppHandle,
    source: &RuntimeSource,
) -> Result<(), String> {
    let dir = managed_zip_dir(app)?;
    let RuntimeSource::ZipPath(path) = source;
    remember_successful_zip_in_dir(&dir, path)
}

fn remember_successful_zip_in_dir(dir: &Path, path: &Path) -> Result<(), String> {
    if !is_managed_zip_file(dir, path) {
        return Err("successful ZIP is not in the managed ZIP directory".to_string());
    }
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "managed ZIP filename is not valid UTF-8".to_string())?;
    let lock = MANAGED_ZIP_STATE_WRITE_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock
        .lock()
        .map_err(|_| "managed ZIP state lock poisoned".to_string())?;
    let mut failed = failed_managed_zip_names(dir);
    if failed.remove(name) {
        save_failed_managed_zip_names(dir, &failed)?;
    }
    write_atomic(&dir.join(LAST_SUCCESSFUL_ZIP_FILE), name.as_bytes())
}

/// Exclude a failed managed ZIP from future automatic selection.
///
/// The same ZIP can still be explicitly selected and retried.
///
/// # Errors
///
/// Returns an error when the source is outside the managed ZIP directory or the failure list cannot be written.
pub(crate) fn remember_failed_managed_zip(
    app: &tauri::AppHandle,
    source: &RuntimeSource,
) -> Result<(), String> {
    let dir = managed_zip_dir(app)?;
    let RuntimeSource::ZipPath(path) = source;
    remember_failed_zip_in_dir(&dir, path)
}

fn remember_failed_zip_in_dir(dir: &Path, path: &Path) -> Result<(), String> {
    if !is_managed_zip_file(dir, path) {
        return Err("failed ZIP is not in the managed ZIP directory".to_string());
    }
    let name = path
        .file_name()
        .and_then(|value| value.to_str())
        .ok_or_else(|| "managed ZIP filename is not valid UTF-8".to_string())?;
    let lock = MANAGED_ZIP_STATE_WRITE_LOCK.get_or_init(|| Mutex::new(()));
    let _guard = lock
        .lock()
        .map_err(|_| "managed ZIP state lock poisoned".to_string())?;
    let mut failed = failed_managed_zip_names(dir);
    if failed.insert(name.to_string()) {
        save_failed_managed_zip_names(dir, &failed)?;
    }
    Ok(())
}

fn runtime_cache_dir(app: &tauri::AppHandle) -> Result<PathBuf, String> {
    let dir = managed_root(app)?.join(RUNTIME_CACHE_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create runtime cache dir: {e}"))?;
    Ok(dir)
}

fn runtime_cache_source_id(source: &RuntimeSource) -> String {
    match source {
        RuntimeSource::ZipPath(path) => {
            let file = path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("zip-runtime");
            sanitize_name(file.trim_end_matches(".zip"))
        }
    }
}

fn runtime_cache_source_dir(
    app: &tauri::AppHandle,
    source: &RuntimeSource,
) -> Result<PathBuf, String> {
    let dir = runtime_cache_dir(app)?.join(runtime_cache_source_id(source));
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create source cache dir: {e}"))?;
    Ok(dir)
}

/// Return per-source Tantivy index directory under runtime cache root.
///
/// # Errors
///
/// Returns an error when source cache directory cannot be created.
pub(crate) fn search_index_dir(
    app: &tauri::AppHandle,
    source: &RuntimeSource,
) -> Result<PathBuf, String> {
    let dir = runtime_cache_source_dir(app, source)?.join(SEARCH_INDEX_DIR);
    fs::create_dir_all(&dir).map_err(|e| format!("failed to create search index dir: {e}"))?;
    Ok(dir)
}

fn encode_bin<T: Serialize>(value: &T) -> Result<Vec<u8>, String> {
    bincode::serialize(value).map_err(|e| format!("bincode encode failed: {e}"))
}

fn decode_bin<T: DeserializeOwned>(bytes: &[u8]) -> Result<T, String> {
    bincode::deserialize(bytes).map_err(|e| format!("bincode decode failed: {e}"))
}

fn compress_zstd(bytes: &[u8]) -> Result<Vec<u8>, String> {
    zstd::stream::encode_all(bytes, ZSTD_LEVEL).map_err(|e| format!("zstd encode failed: {e}"))
}

fn decompress_zstd(bytes: &[u8]) -> Result<Vec<u8>, String> {
    zstd::stream::decode_all(bytes).map_err(|e| format!("zstd decode failed: {e}"))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), String> {
    let tmp = path.with_extension("tmp");
    let mut out = fs::File::create(&tmp).map_err(|e| format!("failed to create temp file: {e}"))?;
    out.write_all(bytes)
        .map_err(|e| format!("failed to write temp file: {e}"))?;
    out.flush()
        .map_err(|e| format!("failed to flush temp file: {e}"))?;
    fs::rename(&tmp, path).map_err(|e| format!("failed to rename temp file: {e}"))?;
    Ok(())
}

/// Copy source ZIP into app-managed cache directory and return managed path.
///
/// Existing files with matching fingerprint are reused.
///
/// # Errors
///
/// Returns an error when source metadata/copy or directory resolution fails.
pub(crate) fn ensure_managed_zip_copy(
    app: &tauri::AppHandle,
    source_zip: &Path,
) -> Result<PathBuf, String> {
    let dir = managed_zip_dir(app)?;
    if is_managed_zip_file(&dir, source_zip) {
        return Ok(source_zip.to_path_buf());
    }

    let stem = source_zip
        .file_stem()
        .and_then(|s| s.to_str())
        .map(sanitize_name)
        .unwrap_or_else(|| "dictionary".to_string());
    let fp = source_fingerprint(source_zip)?;
    let managed = dir.join(format!("{stem}-{fp}.zip"));
    if managed.exists() {
        return Ok(managed);
    }
    fs::copy(source_zip, &managed).map_err(|e| format!("failed to copy zip to managed dir: {e}"))?;
    Ok(managed)
}

/// Load persisted runtime cache, if present.
///
/// # Errors
///
/// Returns an error when cache directory resolution fails.
pub(crate) fn load_runtime_cache(
    app: &tauri::AppHandle,
    source: &RuntimeSource,
) -> Result<Option<PersistedRuntime>, String> {
    let source_dir = runtime_cache_source_dir(app, source)?;
    let manifest_file = source_dir.join(CACHE_MANIFEST_FILE);
    let contents_file = source_dir.join(CACHE_CONTENTS_FILE);
    let entries_file = source_dir.join(CACHE_ENTRIES_FILE);

    if !manifest_file.exists() || !contents_file.exists() || !entries_file.exists() {
        return Ok(None);
    }

    let fallback_none = || {
        let _ = fs::remove_dir_all(&source_dir);
        Ok(None)
    };

    let manifest_bytes = match fs::read(&manifest_file) {
        Ok(v) => v,
        Err(_) => return fallback_none(),
    };
    let manifest: RuntimeCacheManifest = match decode_bin(&manifest_bytes) {
        Ok(v) => v,
        Err(_) => return fallback_none(),
    };
    if manifest.version != RUNTIME_CACHE_VERSION {
        return fallback_none();
    }

    let contents_encoded = match fs::read(&contents_file) {
        Ok(v) => v,
        Err(_) => return fallback_none(),
    };
    let entries_encoded = match fs::read(&entries_file) {
        Ok(v) => v,
        Err(_) => return fallback_none(),
    };
    let contents_bytes = match decompress_zstd(&contents_encoded) {
        Ok(v) => v,
        Err(_) => return fallback_none(),
    };
    let entries_bytes = match decompress_zstd(&entries_encoded) {
        Ok(v) => v,
        Err(_) => return fallback_none(),
    };

    let contents: Vec<ContentItem> = match decode_bin(&contents_bytes) {
        Ok(v) => v,
        Err(_) => return fallback_none(),
    };
    let entries: Vec<EntryDetail> = match decode_bin(&entries_bytes) {
        Ok(v) => v,
        Err(_) => return fallback_none(),
    };

    if contents.len() != manifest.contents_count || entries.len() != manifest.entries_count {
        return fallback_none();
    }

    Ok(Some(PersistedRuntime { contents, entries }))
}

/// Save runtime cache atomically.
///
/// # Errors
///
/// Returns an error when cache serialization/write fails.
pub(crate) fn save_runtime_cache(
    app: &tauri::AppHandle,
    source: &RuntimeSource,
    persisted: &PersistedRuntime,
) -> Result<(), String> {
    let source_dir = runtime_cache_source_dir(app, source)?;
    let manifest_file = source_dir.join(CACHE_MANIFEST_FILE);
    let contents_file = source_dir.join(CACHE_CONTENTS_FILE);
    let entries_file = source_dir.join(CACHE_ENTRIES_FILE);

    let contents_raw = encode_bin(&persisted.contents)?;
    let entries_raw = encode_bin(&persisted.entries)?;
    let contents_comp = compress_zstd(&contents_raw)?;
    let entries_comp = compress_zstd(&entries_raw)?;

    write_atomic(&contents_file, &contents_comp)?;
    write_atomic(&entries_file, &entries_comp)?;

    let manifest = RuntimeCacheManifest {
        version: RUNTIME_CACHE_VERSION,
        contents_count: persisted.contents.len(),
        entries_count: persisted.entries.len(),
    };
    let manifest_bytes = encode_bin(&manifest)?;
    write_atomic(&manifest_file, &manifest_bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::tempdir;

    use super::{
        failed_managed_zip_names, latest_managed_zip_in_dir, preferred_managed_zip_in_dir,
        remember_failed_zip_in_dir, remember_successful_zip_in_dir, LAST_SUCCESSFUL_ZIP_FILE,
    };

    #[test]
    fn successful_zip_is_preferred_over_a_later_failed_candidate() {
        let dir = tempdir().expect("temporary directory");
        let successful = dir.path().join("successful.zip");
        let failed = dir.path().join("failed.zip");
        fs::write(&successful, b"valid build marker").expect("successful candidate");
        fs::write(&failed, b"failed build marker").expect("failed candidate");

        remember_successful_zip_in_dir(dir.path(), &successful).expect("remember successful zip");

        assert_eq!(preferred_managed_zip_in_dir(dir.path()).expect("preferred zip"), Some(successful));
    }

    #[test]
    fn missing_or_invalid_pointer_falls_back_to_existing_zip() {
        let dir = tempdir().expect("temporary directory");
        let candidate = dir.path().join("dictionary.zip");
        fs::write(&candidate, b"zip candidate").expect("zip candidate");

        assert_eq!(preferred_managed_zip_in_dir(dir.path()).expect("fallback zip"), Some(candidate.clone()));
        fs::write(dir.path().join(LAST_SUCCESSFUL_ZIP_FILE), "../outside.zip")
            .expect("invalid pointer");
        assert_eq!(preferred_managed_zip_in_dir(dir.path()).expect("fallback zip"), Some(candidate.clone()));

        fs::write(dir.path().join(LAST_SUCCESSFUL_ZIP_FILE), "missing.zip")
            .expect("missing pointer target");
        assert_eq!(preferred_managed_zip_in_dir(dir.path()).expect("fallback zip"), Some(candidate));
    }

    #[test]
    fn only_managed_zip_files_can_be_remembered() {
        let dir = tempdir().expect("temporary directory");
        let outside = tempdir().expect("outside directory");
        let outside_zip = outside.path().join("other.zip");
        fs::write(&outside_zip, b"outside").expect("outside zip");

        assert!(remember_successful_zip_in_dir(dir.path(), &outside_zip).is_err());
        assert!(latest_managed_zip_in_dir(dir.path()).expect("empty managed dir").is_none());
    }

    #[test]
    fn failed_zip_is_skipped_automatically_but_can_succeed_on_retry() {
        let dir = tempdir().expect("temporary directory");
        let candidate = dir.path().join("candidate.zip");
        fs::write(&candidate, b"candidate").expect("zip candidate");

        remember_failed_zip_in_dir(dir.path(), &candidate).expect("mark failed zip");
        assert!(preferred_managed_zip_in_dir(dir.path()).expect("preferred zip").is_none());
        assert!(candidate.is_file(), "explicit retry must remain possible");

        remember_successful_zip_in_dir(dir.path(), &candidate).expect("retry succeeds");
        assert_eq!(preferred_managed_zip_in_dir(dir.path()).expect("preferred zip"), Some(candidate));
        assert!(failed_managed_zip_names(dir.path()).is_empty());
    }

    #[test]
    fn failed_zip_does_not_replace_prior_successful_selection() {
        let dir = tempdir().expect("temporary directory");
        let successful = dir.path().join("successful.zip");
        let failed = dir.path().join("failed.zip");
        fs::write(&successful, b"successful").expect("successful zip");
        fs::write(&failed, b"failed").expect("failed zip");

        remember_successful_zip_in_dir(dir.path(), &successful).expect("remember successful zip");
        remember_failed_zip_in_dir(dir.path(), &failed).expect("remember failed zip");

        assert_eq!(preferred_managed_zip_in_dir(dir.path()).expect("preferred zip"), Some(successful));
    }

    #[test]
    fn pointer_write_failure_does_not_leave_a_recovered_zip_excluded() {
        let dir = tempdir().expect("temporary directory");
        let candidate = dir.path().join("candidate.zip");
        fs::write(&candidate, b"candidate").expect("zip candidate");
        remember_failed_zip_in_dir(dir.path(), &candidate).expect("mark failed zip");
        fs::create_dir(dir.path().join(LAST_SUCCESSFUL_ZIP_FILE)).expect("block pointer file");

        assert!(remember_successful_zip_in_dir(dir.path(), &candidate).is_err());
        assert!(failed_managed_zip_names(dir.path()).is_empty());
        assert_eq!(preferred_managed_zip_in_dir(dir.path()).expect("fallback zip"), Some(candidate));
    }
}
