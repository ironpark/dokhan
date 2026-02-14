//! Serializable API models and runtime data structures shared across modules.
use serde::Serialize;
use std::collections::BTreeMap;
use std::path::PathBuf;

#[cfg(test)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct FileTypeCount {
    pub(crate) extension: String,
    pub(crate) count: usize,
}

#[cfg(test)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DatasetSummary {
    pub(crate) zip_path: String,
    pub(crate) total_entries: usize,
    pub(crate) total_files: usize,
    pub(crate) total_dirs: usize,
    pub(crate) chm_count: usize,
    pub(crate) lnk_count: usize,
    pub(crate) txt_count: usize,
    pub(crate) uncompressed_bytes: u64,
    pub(crate) compressed_bytes: u64,
    pub(crate) compression_ratio: f64,
    pub(crate) has_master_chm: bool,
    pub(crate) has_readme_txt: bool,
    pub(crate) missing_main_volumes: Vec<String>,
    pub(crate) missing_main_files_only: Vec<String>,
    pub(crate) main_volume_coverage: Vec<MainVolumeCoverage>,
    pub(crate) extension_counts: Vec<FileTypeCount>,
    pub(crate) sample_chm_files: Vec<String>,
}

#[cfg(test)]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MainVolumeCoverage {
    pub(crate) volume: u8,
    pub(crate) has_main_file: bool,
    pub(crate) split_file_count: usize,
    pub(crate) covered: bool,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentItem {
    pub(crate) title: String,
    pub(crate) local: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DictionaryIndexEntry {
    pub(crate) id: usize,
    pub(crate) headword: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) source_path: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SearchHit {
    pub(crate) id: usize,
    pub(crate) headword: String,
    pub(crate) source_path: String,
    pub(crate) score: usize,
    pub(crate) snippet: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct EntryDetail {
    pub(crate) id: usize,
    pub(crate) headword: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) source_path: String,
    pub(crate) target_local: String,
    pub(crate) definition_text: String,
    pub(crate) definition_html: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentPage {
    pub(crate) local: String,
    pub(crate) source_path: String,
    pub(crate) title: String,
    pub(crate) body_text: String,
    pub(crate) body_html: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase", tag = "kind")]
pub(crate) enum LinkTarget {
    Content {
        local: String,
        source_path: String,
    },
    Entry {
        id: usize,
    },
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct MasterFeatureSummary {
    pub(crate) zip_path: String,
    pub(crate) content_count: usize,
    pub(crate) index_count: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildProgress {
    pub(crate) phase: String,
    pub(crate) current: usize,
    pub(crate) total: usize,
    pub(crate) message: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BuildStatus {
    pub(crate) phase: String,
    pub(crate) current: usize,
    pub(crate) total: usize,
    pub(crate) message: String,
    pub(crate) done: bool,
    pub(crate) success: bool,
    pub(crate) error: Option<String>,
    pub(crate) summary: Option<MasterFeatureSummary>,
}

#[derive(Debug, Clone)]
pub(crate) struct RuntimeIndex {
    pub(crate) contents: Vec<ContentItem>,
    pub(crate) entries: Vec<EntryDetail>,
    pub(crate) content_pages: BTreeMap<String, ContentPage>,
    pub(crate) entry_keys: Vec<EntrySearchKey>,
}

#[derive(Debug, Clone)]
pub(crate) struct EntrySearchKey {
    pub(crate) headword: String,
    pub(crate) headword_loose: String,
    pub(crate) body: String,
    pub(crate) body_loose: String,
    pub(crate) aliases: Vec<String>,
    pub(crate) aliases_loose: Vec<String>,
}

#[derive(Debug, Clone)]
pub(crate) enum RuntimeSource {
    ZipPath(PathBuf),
}
