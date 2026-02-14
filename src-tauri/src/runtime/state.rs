//! Runtime cache/state management and async build lifecycle.
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use tauri::AppHandle;

use crate::app::model::{
    BuildProgress, BuildStatus, ContentItem, ContentPage, EntryDetail, MasterFeatureSummary,
    RuntimeIndex, RuntimeSource,
};
use crate::resolve_runtime_source;
use crate::runtime::search::{build_entry_search_keys, warm_search_index};
use crate::runtime::storage::{load_runtime_cache, save_runtime_cache, PersistedRuntime};
use crate::runtime::zip::{
    hydrate_zip_entry_detail, parse_runtime_from_zip_with_progress, read_content_page_from_zip,
};

static RUNTIME_CACHE: OnceLock<Mutex<BTreeMap<String, Arc<RuntimeIndex>>>> = OnceLock::new();
static BUILD_STATUS: OnceLock<Mutex<BTreeMap<String, BuildStatus>>> = OnceLock::new();

/// Build a stable cache key for a runtime source.
fn cache_key(source: &RuntimeSource) -> String {
    match source {
        RuntimeSource::ZipPath(path) => format!(
            "zip:{}",
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .to_string_lossy()
        ),
    }
}

/// Human-readable source label exposed in API summaries.
fn source_label(source: &RuntimeSource) -> String {
    match source {
        RuntimeSource::ZipPath(path) => path.to_string_lossy().to_string(),
    }
}

/// Build status map key for a runtime source.
fn status_key(source: &RuntimeSource) -> String {
    cache_key(source)
}

/// Insert or replace build status entry.
///
/// # Errors
///
/// Returns an error when the build status mutex is poisoned.
fn set_build_status(key: &str, status: BuildStatus) -> Result<(), String> {
    let map = BUILD_STATUS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = map.lock().map_err(|_| "build status lock poisoned".to_string())?;
    guard.insert(key.to_string(), status);
    Ok(())
}

/// Update an existing build status entry in place.
///
/// # Errors
///
/// Returns an error when the build status mutex is poisoned.
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

/// Read build status if present.
///
/// # Errors
///
/// Returns an error when the build status mutex is poisoned.
fn get_build_status_internal(key: &str) -> Result<Option<BuildStatus>, String> {
    let map = BUILD_STATUS.get_or_init(|| Mutex::new(BTreeMap::new()));
    let guard = map.lock().map_err(|_| "build status lock poisoned".to_string())?;
    Ok(guard.get(key).cloned())
}

/// Read runtime cache entry if present.
///
/// # Errors
///
/// Returns an error when the runtime cache mutex is poisoned.
fn cache_get(source: &RuntimeSource) -> Result<Option<Arc<RuntimeIndex>>, String> {
    let key = cache_key(source);
    let cache = RUNTIME_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let guard = cache.lock().map_err(|_| "runtime cache lock poisoned".to_string())?;
    Ok(guard.get(&key).cloned())
}

/// Insert runtime into cache.
///
/// # Errors
///
/// Returns an error when the runtime cache mutex is poisoned.
fn cache_put(source: &RuntimeSource, runtime: Arc<RuntimeIndex>) -> Result<(), String> {
    let key = cache_key(source);
    let cache = RUNTIME_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    let mut guard = cache.lock().map_err(|_| "runtime cache lock poisoned".to_string())?;
    guard.insert(key, runtime);
    Ok(())
}

/// Convert runtime snapshot to API summary payload.
fn summary_from_runtime(source: &RuntimeSource, runtime: &RuntimeIndex) -> MasterFeatureSummary {
    MasterFeatureSummary {
        zip_path: source_label(source),
        content_count: runtime.contents.len(),
        index_count: runtime.entries.len(),
    }
}

/// Store successful terminal build status.
///
/// # Errors
///
/// Returns an error when status storage is unavailable.
fn set_build_done_status(
    key: &str,
    summary: MasterFeatureSummary,
    message: &str,
) -> Result<(), String> {
    set_build_status(
        key,
        BuildStatus {
            phase: "done".to_string(),
            current: summary.index_count,
            total: summary.index_count,
            message: message.to_string(),
            done: true,
            success: true,
            error: None,
            summary: Some(summary),
        },
    )
}

/// Store failed terminal build status.
///
/// # Errors
///
/// Returns an error when status storage is unavailable.
fn set_build_error_status(key: &str, message: &str, error: String) -> Result<(), String> {
    set_build_status(
        key,
        BuildStatus {
            phase: "error".to_string(),
            current: 0,
            total: 1,
            message: message.to_string(),
            done: true,
            success: false,
            error: Some(error),
            summary: None,
        },
    )
}

/// Build runtime for a source while streaming progress updates.
///
/// # Errors
///
/// Returns an error when CHM/ZIP parsing fails or status updates cannot be persisted.
fn build_runtime_for_source(
    app: &AppHandle,
    source: &RuntimeSource,
    key: &str,
) -> Result<Arc<RuntimeIndex>, String> {
    if let Some(persisted) = load_runtime_cache(app, source)? {
        let runtime = Arc::new(build_runtime_index(
            persisted.contents,
            persisted.entries,
            BTreeMap::new(),
        ));
        let _ = update_build_status(key, |st| {
            st.phase = "cache".to_string();
            st.message = "Loaded runtime cache".to_string();
        });
        warm_search_index(app, source, &runtime.entries)?;
        return Ok(runtime);
    }

    let runtime = match source {
        RuntimeSource::ZipPath(zip_path) => {
            let mut cb = |p: BuildProgress| {
                let _ = update_build_status(key, |st| {
                    st.phase = p.phase;
                    st.current = p.current;
                    st.total = p.total;
                    st.message = p.message;
                });
            };
            parse_runtime_from_zip_with_progress(zip_path, Some(&mut cb)).map(Arc::new)?
        }
    };

    let _ = update_build_status(key, |st| {
        st.phase = "search-index".to_string();
        st.message = "Building search index".to_string();
    });
    warm_search_index(app, source, &runtime.entries)?;
    let _ = save_runtime_cache(
        app,
        source,
        &PersistedRuntime {
            contents: runtime.contents.clone(),
            entries: runtime.entries.clone(),
        },
    );
    Ok(runtime)
}

/// Spawn detached worker that builds runtime and updates status.
fn spawn_build_worker(app: AppHandle, source: RuntimeSource, key: String) {
    std::thread::spawn(move || {
        let runtime = match build_runtime_for_source(&app, &source, &key) {
            Ok(runtime) => runtime,
            Err(err) => {
                let _ = set_build_error_status(&key, "Failed parsing zip/chm", err);
                return;
            }
        };

        let summary = summary_from_runtime(&source, &runtime);
        if let Err(err) = cache_put(&source, runtime) {
            let _ = set_build_error_status(&key, "Cache write failed", err);
            return;
        }
        let _ = set_build_done_status(&key, summary, "Build complete");
    });
}

/// Build immutable runtime index with precomputed search keys.
pub(crate) fn build_runtime_index(
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

/// Return cached runtime or build it synchronously.
///
/// # Errors
///
/// Returns an error when parsing fails or cache/status storage is unavailable.
pub(crate) fn get_runtime(app: &AppHandle, source: &RuntimeSource) -> Result<Arc<RuntimeIndex>, String> {
    if let Some(v) = cache_get(source)? {
        return Ok(v);
    }
    if let Some(persisted) = load_runtime_cache(app, source)? {
        let runtime = Arc::new(build_runtime_index(
            persisted.contents,
            persisted.entries,
            BTreeMap::new(),
        ));
        warm_search_index(app, source, &runtime.entries)?;
        cache_put(source, runtime.clone())?;
        return Ok(runtime);
    }
    let runtime = match source {
        RuntimeSource::ZipPath(zip_path) => Arc::new(parse_runtime_from_zip_with_progress(zip_path, None)?),
    };
    warm_search_index(app, source, &runtime.entries)?;
    let _ = save_runtime_cache(
        app,
        source,
        &PersistedRuntime {
            contents: runtime.contents.clone(),
            entries: runtime.entries.clone(),
        },
    );
    cache_put(source, runtime.clone())?;
    Ok(runtime)
}

/// Start async build and return status polling key.
///
/// If a build is already running for the same source, this function returns the existing key.
///
/// # Errors
///
/// Returns an error when source resolution fails or status/cache storage is unavailable.
pub(crate) fn start_master_build_impl(app: &AppHandle, zip_path: Option<String>) -> Result<String, String> {
    let source = resolve_runtime_source(app, zip_path)?;
    let key = status_key(&source);

    if let Some(runtime) = cache_get(&source)? {
        let summary = summary_from_runtime(&source, &runtime);
        set_build_done_status(&key, summary, "Loaded from cache")?;
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

    spawn_build_worker(app.clone(), source, key.clone());

    Ok(key)
}

/// Get current build status, returning idle if not started.
///
/// # Errors
///
/// Returns an error when source resolution fails or status storage is unavailable.
pub(crate) fn get_master_build_status_impl(
    app: &AppHandle,
    zip_path: Option<String>,
) -> Result<BuildStatus, String> {
    let source = resolve_runtime_source(app, zip_path)?;
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

/// Return parsed content tree for the selected ZIP runtime.
///
/// # Errors
///
/// Returns an error when source resolution or runtime loading fails.
pub(crate) fn get_master_contents_impl(app: &AppHandle, zip_path: Option<String>) -> Result<Vec<ContentItem>, String> {
    let source = resolve_runtime_source(app, zip_path)?;
    Ok(get_runtime(app, &source)?.contents.clone())
}

/// Return entry detail and hydrate body text/html when needed.
///
/// # Errors
///
/// Returns an error when source resolution/runtime loading fails or the entry id does not exist.
pub(crate) fn get_entry_detail_impl(
    app: &AppHandle,
    id: usize,
    zip_path: Option<String>,
) -> Result<EntryDetail, String> {
    let source = resolve_runtime_source(app, zip_path)?;
    let runtime = get_runtime(app, &source)?;
    let entry = runtime
        .entries
        .iter()
        .find(|e| e.id == id)
        .cloned()
        .ok_or_else(|| format!("entry not found: {id}"))?;

    match source {
        RuntimeSource::ZipPath(zip_path) => Ok(hydrate_zip_entry_detail(&zip_path, entry)),
    }
}

/// Return content page HTML/text from runtime cache or CHM object.
///
/// # Errors
///
/// Returns an error when source resolution/runtime loading fails or the content page is missing.
pub(crate) fn get_content_page_impl(
    app: &AppHandle,
    local: &str,
    source_path: Option<&str>,
    zip_path: Option<String>,
) -> Result<ContentPage, String> {
    let source_path = source_path
        .unwrap_or("master.chm")
        .to_ascii_lowercase();
    let source = resolve_runtime_source(app, zip_path)?;
    match &source {
        RuntimeSource::ZipPath(zip_path) => {
            let runtime = get_runtime(app, &source)?;
            if source_path == "master.chm" {
                if let Some(v) = runtime.content_pages.get(local).cloned() {
                    return Ok(v);
                }
            }
            read_content_page_from_zip(zip_path, &source_path, local)
        }
    }
}
