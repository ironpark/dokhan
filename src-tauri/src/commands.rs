#[tauri::command]
fn analyze_zip_dataset(zip_path: String) -> Result<super::DatasetSummary, String> {
    super::analyze_zip_dataset_impl(zip_path)
}

#[tauri::command]
fn analyze_default_dataset() -> Result<super::DatasetSummary, String> {
    super::analyze_default_dataset_impl()
}

#[tauri::command]
fn preview_chm_paths(
    zip_path: Option<String>,
    sample_limit: Option<usize>,
) -> Result<Vec<super::ChmPathPreview>, String> {
    super::preview_chm_paths_impl(zip_path, sample_limit)
}

#[tauri::command]
fn extract_headwords_preview(
    zip_path: Option<String>,
    chm_file: Option<String>,
    sample_limit: Option<usize>,
) -> Result<Vec<super::HeadwordPreview>, String> {
    super::extract_headwords_preview_impl(zip_path, chm_file, sample_limit)
}

#[tauri::command]
fn extract_headwords_from_hhk(
    zip_path: Option<String>,
    chm_file: Option<String>,
    sample_limit: Option<usize>,
) -> Result<Vec<super::HhkHeadwordPreview>, String> {
    super::extract_headwords_from_hhk_impl(zip_path, chm_file, sample_limit)
}

#[tauri::command]
fn validate_dataset_pipeline(
    zip_path: Option<String>,
) -> Result<super::DatasetValidationReport, String> {
    super::validate_dataset_pipeline_impl(zip_path)
}

#[tauri::command]
fn build_master_features(
    debug_root: Option<String>,
) -> Result<super::MasterFeatureSummary, String> {
    super::build_master_features_impl(debug_root)
}

#[tauri::command]
fn build_master_features_with_progress(
    window: tauri::Window,
    debug_root: Option<String>,
) -> Result<super::MasterFeatureSummary, String> {
    super::build_master_features_with_progress_impl(window, debug_root)
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
    debug_root: Option<String>,
) -> Result<super::ContentPage, String> {
    super::get_content_page_impl(local, debug_root)
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .invoke_handler(tauri::generate_handler![
            analyze_zip_dataset,
            analyze_default_dataset,
            preview_chm_paths,
            extract_headwords_preview,
            extract_headwords_from_hhk,
            validate_dataset_pipeline,
            build_master_features,
            build_master_features_with_progress,
            start_master_build,
            get_master_build_status,
            get_master_contents,
            get_index_entries,
            search_entries,
            get_entry_detail,
            get_content_page
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
