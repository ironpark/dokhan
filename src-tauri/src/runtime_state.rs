use super::*;
use std::collections::BTreeMap;
use std::sync::{Arc, Mutex, OnceLock};

static RUNTIME_CACHE: OnceLock<Mutex<BTreeMap<String, Arc<RuntimeIndex>>>> = OnceLock::new();
static BUILD_STATUS: OnceLock<Mutex<BTreeMap<String, BuildStatus>>> = OnceLock::new();

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
                if let Some(v) = runtime.content_pages.get(&local).cloned() {
                    return Ok(v);
                }
            }
            read_content_page_from_zip(zip_path, &source_path, &local)
        }
    }
}
