use crate::app::model::{
    BuildStatus, ContentItem, ContentPage, DatasetSummary, DictionaryIndexEntry, EntryDetail,
    LinkTarget, SearchHit,
};
use crate::runtime::link_media::{resolve_link_target_impl, resolve_media_data_url_impl};
use crate::runtime::search::{get_index_entries_impl, search_entries_impl};
use crate::runtime::state::{
    get_content_page_impl, get_entry_detail_impl, get_master_build_status_impl,
    get_master_contents_impl, start_master_build_impl,
};

#[tauri::command]
fn analyze_zip_dataset(zip_path: String) -> Result<DatasetSummary, String> {
    crate::analyze_zip_dataset_impl(&zip_path)
}

#[tauri::command]
fn start_master_build(zip_path: Option<String>) -> Result<String, String> {
    start_master_build_impl(zip_path)
}

#[tauri::command]
fn get_master_build_status(
    zip_path: Option<String>,
) -> Result<BuildStatus, String> {
    get_master_build_status_impl(zip_path)
}

#[tauri::command]
fn get_master_contents(
    zip_path: Option<String>,
) -> Result<Vec<ContentItem>, String> {
    get_master_contents_impl(zip_path)
}

#[tauri::command]
fn get_index_entries(
    prefix: Option<String>,
    limit: Option<usize>,
    zip_path: Option<String>,
) -> Result<Vec<DictionaryIndexEntry>, String> {
    get_index_entries_impl(prefix, limit, zip_path)
}

#[tauri::command]
fn search_entries(
    query: String,
    limit: Option<usize>,
    zip_path: Option<String>,
) -> Result<Vec<SearchHit>, String> {
    search_entries_impl(&query, limit, zip_path)
}

#[tauri::command]
fn get_entry_detail(
    id: usize,
    zip_path: Option<String>,
) -> Result<EntryDetail, String> {
    get_entry_detail_impl(id, zip_path)
}

#[tauri::command]
fn get_content_page(
    local: String,
    source_path: Option<String>,
    zip_path: Option<String>,
) -> Result<ContentPage, String> {
    get_content_page_impl(&local, source_path.as_deref(), zip_path)
}

#[tauri::command]
fn resolve_link_target(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    zip_path: Option<String>,
) -> Result<LinkTarget, String> {
    resolve_link_target_impl(
        &href,
        current_source_path.as_deref(),
        current_local.as_deref(),
        zip_path,
    )
}

#[tauri::command]
fn resolve_media_data_url(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    zip_path: Option<String>,
) -> Result<String, String> {
    resolve_media_data_url_impl(
        &href,
        current_source_path.as_deref(),
        current_local.as_deref(),
        zip_path,
    )
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            analyze_zip_dataset,
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
