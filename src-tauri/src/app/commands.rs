//! Tauri command handlers exposed to the frontend.
#[cfg(target_os = "android")]
use std::fs;
#[cfg(target_os = "android")]
use std::path::PathBuf;
#[cfg(target_os = "android")]
use std::time::{SystemTime, UNIX_EPOCH};

use crate::app::model::{
    BuildStatus, ContentItem, ContentPage, DictionaryIndexEntry, EntryDetail,
    LinkTarget, SearchHit,
};
use crate::runtime::link_media::{resolve_link_target_impl, resolve_media_data_url_impl};
use crate::runtime::search::{get_index_entries_impl, search_entries_impl};
use crate::runtime::state::{
    get_content_page_impl, get_entry_detail_impl, get_master_build_status_impl,
    get_master_contents_impl, start_master_build_impl,
};
#[cfg(target_os = "android")]
use std::io;
#[cfg(target_os = "android")]
use tauri_plugin_fs::FsExt;

/// Normalize selected ZIP path/URI to a local file-system path.
///
/// On Android, `content://` URIs are copied to app temp dir via `app.fs().open(...)`.
///
/// # Errors
///
/// Returns an error when source URI cannot be opened or temp file write fails.
#[tauri::command]
fn prepare_zip_source(path: String, app: tauri::AppHandle) -> Result<String, String> {
    let raw = path.trim();
    if raw.is_empty() {
        return Err("zip path is empty".to_string());
    }

    if raw.starts_with("content://") {
        #[cfg(target_os = "android")]
        {
            let mut dir = std::env::temp_dir();
            dir.push("dokhan-zips");
            fs::create_dir_all(&dir).map_err(|e| format!("failed to create temp zip dir: {e}"))?;

            let stamp = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .map_err(|e| format!("failed to read system time: {e}"))?
                .as_millis();
            let file_name = format!("picked-{stamp}-{}.zip", std::process::id());
            let mut out = PathBuf::from(&dir);
            out.push(file_name);

            let uri = url::Url::parse(raw).map_err(|e| format!("invalid content uri: {e}"))?;
            let mut open_opts = tauri_plugin_fs::OpenOptions::new();
            open_opts.read(true);
            let mut src = app
                .fs()
                .open(tauri_plugin_fs::FilePath::Url(uri), open_opts)
                .map_err(|e| format!("failed to open content uri: {e}"))?;
            let mut dst =
                fs::File::create(&out).map_err(|e| format!("failed to create temp zip: {e}"))?;
            io::copy(&mut src, &mut dst)
                .map_err(|e| format!("failed to copy content uri to temp zip: {e}"))?;
            return Ok(out.to_string_lossy().to_string());
        }
        #[cfg(not(target_os = "android"))]
        {
            let _ = app;
            return Err("content uri is only supported on Android".to_string());
        }
    }

    if raw.starts_with("file://") {
        let url = url::Url::parse(raw).map_err(|e| format!("invalid file url: {e}"))?;
        let path = url
            .to_file_path()
            .map_err(|_| "file url is not a valid path".to_string())?;
        return Ok(path.to_string_lossy().to_string());
    }

    Ok(raw.to_string())
}

/// Start asynchronous runtime build for a ZIP dataset.
///
/// If a build is already running for the same ZIP source, the existing polling key is returned.
///
/// # Errors
///
/// Returns an error when `zip_path` is missing/invalid or status storage cannot be updated.
#[tauri::command]
fn start_master_build(zip_path: Option<String>, app: tauri::AppHandle) -> Result<String, String> {
    start_master_build_impl(&app, zip_path)
}

/// Get current asynchronous build status for the given ZIP source.
///
/// # Errors
///
/// Returns an error when `zip_path` is missing/invalid or status storage is unavailable.
#[tauri::command]
fn get_master_build_status(
    zip_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<BuildStatus, String> {
    get_master_build_status_impl(&app, zip_path)
}

/// Return parsed content tree entries.
///
/// # Errors
///
/// Returns an error when runtime source resolution or runtime loading fails.
#[tauri::command]
fn get_master_contents(
    zip_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<Vec<ContentItem>, String> {
    get_master_contents_impl(&app, zip_path)
}

/// Return index rows with optional prefix filtering.
///
/// # Errors
///
/// Returns an error when runtime source resolution or runtime loading fails.
#[tauri::command]
fn get_index_entries(
    prefix: Option<String>,
    limit: Option<usize>,
    zip_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<Vec<DictionaryIndexEntry>, String> {
    get_index_entries_impl(&app, prefix, limit, zip_path)
}

/// Run full-text search against in-memory runtime index.
///
/// # Errors
///
/// Returns an error when runtime source resolution or runtime loading fails.
#[tauri::command]
fn search_entries(
    query: String,
    limit: Option<usize>,
    zip_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<Vec<SearchHit>, String> {
    search_entries_impl(&app, &query, limit, zip_path)
}

/// Load a dictionary entry detail by stable runtime id.
///
/// # Errors
///
/// Returns an error when the entry id is missing, runtime loading fails, or CHM hydration fails.
#[tauri::command]
fn get_entry_detail(
    id: usize,
    zip_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<EntryDetail, String> {
    get_entry_detail_impl(&app, id, zip_path)
}

/// Read a content page by local path and optional source CHM.
///
/// # Errors
///
/// Returns an error when runtime loading fails or the target content page cannot be read.
#[tauri::command]
fn get_content_page(
    local: String,
    source_path: Option<String>,
    zip_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<ContentPage, String> {
    get_content_page_impl(&app, &local, source_path.as_deref(), zip_path)
}

/// Resolve an internal CHM hyperlink to either content or entry target.
///
/// # Errors
///
/// Returns an error for unsupported links or when runtime/source resolution fails.
#[tauri::command]
fn resolve_link_target(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    zip_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<LinkTarget, String> {
    resolve_link_target_impl(
        &app,
        &href,
        current_source_path.as_deref(),
        current_local.as_deref(),
        zip_path,
    )
}

/// Resolve media href (image asset) into a data URL for webview rendering.
///
/// # Errors
///
/// Returns an error for unsupported media links or when the binary object cannot be loaded.
#[tauri::command]
fn resolve_media_data_url(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    zip_path: Option<String>,
    app: tauri::AppHandle,
) -> Result<String, String> {
    resolve_media_data_url_impl(
        &app,
        &href,
        current_source_path.as_deref(),
        current_local.as_deref(),
        zip_path,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
/// Start the Tauri application and register all frontend-invokable commands.
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_fs::init())
        .plugin(tauri_plugin_os::init())
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            prepare_zip_source,
            start_master_build,
            get_master_build_status,
            get_master_contents,
            get_index_entries,
            search_entries,
            get_entry_detail,
            get_content_page,
            resolve_link_target,
            resolve_media_data_url
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
