use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

use crate::app::model::{
    BuildProgress, BuildStatus, ContentItem, ContentPage, EntryDetail, MasterFeatureSummary,
    RuntimeIndex, RuntimeSource,
};
use crate::resolve_runtime_source;
use crate::runtime::search::build_entry_search_keys;
use crate::runtime::zip::{
    hydrate_zip_entry_detail, parse_runtime_from_zip_with_progress, read_content_page_from_zip,
};

static RUNTIME_CACHE: OnceLock<Mutex<BTreeMap<String, Arc<RuntimeIndex>>>> = OnceLock::new();
static BUILD_STATUS: OnceLock<Mutex<BTreeMap<String, BuildStatus>>> = OnceLock::new();

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

fn source_label(source: &RuntimeSource) -> String {
    match source {
        RuntimeSource::ZipPath(path) => path.to_string_lossy().to_string(),
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

fn summary_from_runtime(source: &RuntimeSource, runtime: &RuntimeIndex) -> MasterFeatureSummary {
    MasterFeatureSummary {
        zip_path: source_label(source),
        content_count: runtime.contents.len(),
        index_count: runtime.entries.len(),
    }
}

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

fn build_runtime_for_source(source: &RuntimeSource, key: &str) -> Result<Arc<RuntimeIndex>, String> {
    match source {
        RuntimeSource::ZipPath(zip_path) => {
            let mut cb = |p: BuildProgress| {
                let _ = update_build_status(key, |st| {
                    st.phase = p.phase;
                    st.current = p.current;
                    st.total = p.total;
                    st.message = p.message;
                });
            };
            parse_runtime_from_zip_with_progress(zip_path, Some(&mut cb)).map(Arc::new)
        }
    }
}

fn spawn_build_worker(source: RuntimeSource, key: String) {
    std::thread::spawn(move || {
        let runtime = match build_runtime_for_source(&source, &key) {
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

pub(crate) fn get_runtime(source: &RuntimeSource) -> Result<Arc<RuntimeIndex>, String> {
    if let Some(v) = cache_get(source)? {
        return Ok(v);
    }
    let runtime = match source {
        RuntimeSource::ZipPath(zip_path) => Arc::new(parse_runtime_from_zip_with_progress(zip_path, None)?),
    };
    cache_put(source, runtime.clone())?;
    Ok(runtime)
}

pub(crate) fn start_master_build_impl(zip_path: Option<String>) -> Result<String, String> {
    let source = resolve_runtime_source(zip_path)?;
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

    spawn_build_worker(source, key.clone());

    Ok(key)
}

pub(crate) fn get_master_build_status_impl(
    zip_path: Option<String>,
) -> Result<BuildStatus, String> {
    let source = resolve_runtime_source(zip_path)?;
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

pub(crate) fn get_master_contents_impl(zip_path: Option<String>) -> Result<Vec<ContentItem>, String> {
    let source = resolve_runtime_source(zip_path)?;
    Ok(get_runtime(&source)?.contents.clone())
}

pub(crate) fn get_entry_detail_impl(id: usize, zip_path: Option<String>) -> Result<EntryDetail, String> {
    let source = resolve_runtime_source(zip_path)?;
    let runtime = get_runtime(&source)?;
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

pub(crate) fn get_content_page_impl(
    local: &str,
    source_path: Option<&str>,
    zip_path: Option<String>,
) -> Result<ContentPage, String> {
    let source_path = source_path
        .unwrap_or("master.chm")
        .to_ascii_lowercase();
    let source = resolve_runtime_source(zip_path)?;
    match &source {
        RuntimeSource::ZipPath(zip_path) => {
            let runtime = get_runtime(&source)?;
            if source_path == "master.chm" {
                if let Some(v) = runtime.content_pages.get(local).cloned() {
                    return Ok(v);
                }
            }
            read_content_page_from_zip(zip_path, &source_path, local)
        }
    }
}
