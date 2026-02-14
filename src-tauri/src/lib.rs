//! Tauri backend crate entry for dictionary runtime/parsing layers.
mod chm;
mod app;
mod parsing;
mod runtime;

pub use app::commands::run;
use crate::app::model::RuntimeSource;

/// Resolve runtime source from optional ZIP path argument.
///
/// # Errors
///
/// Returns an error when `zip_path` is missing or does not resolve to an existing ZIP file.
fn resolve_runtime_source(input: Option<String>) -> Result<RuntimeSource, String> {
    match input {
        Some(raw) => Ok(RuntimeSource::ZipPath(parsing::dataset::resolve_zip_path(&raw)?)),
        None => Err("zip path is required".to_string()),
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::*;

    #[test]
    fn split_volume_counts_as_covered() {
        let names = vec![
            "merge01.chm".to_string(),
            "merge03-01.chm".to_string(),
            "merge03-02.chm".to_string(),
        ];
        let coverage = parsing::dataset::build_main_volume_coverage(&names);
        let v1 = coverage.iter().find(|x| x.volume == 1).expect("v1");
        let v3 = coverage.iter().find(|x| x.volume == 3).expect("v3");
        let v4 = coverage.iter().find(|x| x.volume == 4).expect("v4");
        assert!(v1.covered && v1.has_main_file);
        assert!(v3.covered && !v3.has_main_file && v3.split_file_count == 2);
        assert!(!v4.covered);
    }

    #[test]
    fn dataset_smoke_validation_if_present() {
        let candidates = [
            std::path::PathBuf::from("../asset/dictionary_v77.zip"),
            std::path::PathBuf::from("asset/dictionary_v77.zip"),
        ];
        let Some(path) = candidates.into_iter().find(|p| p.exists()) else {
            return;
        };

        let summary = parsing::dataset::summarize_zip(&path).expect("summary");
        assert_eq!(summary.chm_count, 120);
        assert_eq!(summary.lnk_count, 2);
        assert_eq!(summary.txt_count, 1);
        assert!(summary.has_master_chm);
        assert!(summary.has_readme_txt);
        assert!(summary.missing_main_volumes.is_empty());
    }

    #[test]
    fn dataset_hhk_smoke_if_present() {
        let candidates = [
            std::path::PathBuf::from("../asset/dictionary_v77.zip"),
            std::path::PathBuf::from("asset/dictionary_v77.zip"),
        ];
        let Some(path) = candidates.into_iter().find(|p| p.exists()) else {
            return;
        };

        let mut archive = parsing::dataset::open_zip_archive(Path::new(&path)).expect("zip");
        let mut detected = 0usize;
        for i in 0..archive.len() {
            let mut entry = archive.by_index(i).expect("entry");
            if entry.is_dir() {
                continue;
            }
            let name = entry.name().to_ascii_lowercase();
            if !name.ends_with(".chm") {
                continue;
            }
            let mut bytes = Vec::new();
            std::io::copy(&mut entry, &mut bytes).expect("copy");
            let (_count, _sample, hhk_paths) =
                crate::parsing::index::extract_headwords_from_hhk_bytes(&bytes, 3);
            if hhk_paths > 0 {
                detected += 1;
            }
            if detected >= 3 {
                break;
            }
        }
        assert!(detected > 0, "expected at least one CHM to expose .hhk hints");
    }
}
