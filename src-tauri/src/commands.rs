#[tauri::command]
fn analyze_zip_dataset(zip_path: String) -> Result<super::DatasetSummary, String> {
    super::analyze_zip_dataset_impl(zip_path)
}

#[tauri::command]
fn start_master_build(debug_root: Option<String>) -> Result<String, String> {
    super::start_master_build_impl(debug_root)
}

#[tauri::command]
fn get_master_build_status(
    debug_root: Option<String>,
) -> Result<super::BuildStatus, String> {
    super::get_master_build_status_impl(debug_root)
}

#[tauri::command]
fn get_master_contents(
    debug_root: Option<String>,
) -> Result<Vec<super::ContentItem>, String> {
    super::get_master_contents_impl(debug_root)
}

#[tauri::command]
fn get_index_entries(
    prefix: Option<String>,
    limit: Option<usize>,
    debug_root: Option<String>,
) -> Result<Vec<super::DictionaryIndexEntry>, String> {
    super::get_index_entries_impl(prefix, limit, debug_root)
}

#[tauri::command]
fn search_entries(
    query: String,
    limit: Option<usize>,
    debug_root: Option<String>,
) -> Result<Vec<super::SearchHit>, String> {
    super::search_entries_impl(query, limit, debug_root)
}

#[tauri::command]
fn get_entry_detail(
    id: usize,
    debug_root: Option<String>,
) -> Result<super::EntryDetail, String> {
    super::get_entry_detail_impl(id, debug_root)
}

#[tauri::command]
fn get_content_page(
    local: String,
    source_path: Option<String>,
    debug_root: Option<String>,
) -> Result<super::ContentPage, String> {
    super::get_content_page_impl(local, source_path, debug_root)
}

#[tauri::command]
fn resolve_link_target(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    debug_root: Option<String>,
) -> Result<super::LinkTarget, String> {
    super::resolve_link_target_impl(href, current_source_path, current_local, debug_root)
}

#[tauri::command]
fn resolve_media_data_url(
    href: String,
    current_source_path: Option<String>,
    current_local: Option<String>,
    debug_root: Option<String>,
) -> Result<String, String> {
    super::resolve_media_data_url_impl(href, current_source_path, current_local, debug_root)
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
