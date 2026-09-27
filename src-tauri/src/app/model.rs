//! Serializable API models and runtime data structures shared across modules.
use serde::{Deserialize, Serialize};
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

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ContentItem {
    pub(crate) title: String,
    pub(crate) local: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TextSpan {
    pub(crate) start: usize,
    pub(crate) end: usize,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub(crate) struct DictionaryIndexEntry {
    pub(crate) id: usize,
    pub(crate) headword: String,
    pub(crate) headword_highlights: Vec<TextSpan>,
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

#[derive(Debug, Serialize, Deserialize, Clone)]
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

impl RuntimeIndex {
    /// Entry IDs are assigned from 1 in sorted order. Fall back to a scan for older caches.
    pub(crate) fn entry_by_id(&self, id: usize) -> Option<&EntryDetail> {
        self.entries
            .get(id.checked_sub(1)?)
            .filter(|entry| entry.id == id)
            .or_else(|| self.entries.iter().find(|entry| entry.id == id))
    }
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

impl RuntimeSource {
    pub(crate) fn cache_key(&self) -> String {
        match self {
            Self::ZipPath(path) => format!(
                "zip:{}",
                path.canonicalize()
                    .unwrap_or_else(|_| path.to_path_buf())
                    .to_string_lossy()
            ),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{EntryDetail, RuntimeIndex};
    use std::collections::BTreeMap;

    fn entry(id: usize) -> EntryDetail {
        EntryDetail {
            id,
            headword: format!("word-{id}"),
            aliases: Vec::new(),
            source_path: String::new(),
            target_local: String::new(),
            definition_text: String::new(),
            definition_html: String::new(),
        }
    }

    #[test]
    fn entry_lookup_uses_position_and_handles_legacy_order() {
        let mut runtime = RuntimeIndex {
            contents: Vec::new(),
            entries: vec![entry(1), entry(2)],
            content_pages: BTreeMap::new(),
            entry_keys: Vec::new(),
        };
        assert_eq!(runtime.entry_by_id(2).map(|e| e.headword.as_str()), Some("word-2"));
        assert!(runtime.entry_by_id(0).is_none());

        runtime.entries.swap(0, 1);
        assert_eq!(runtime.entry_by_id(2).map(|e| e.headword.as_str()), Some("word-2"));
    }
}
