//! Tauri backend crate entry for dictionary runtime/parsing layers.
mod chm;
mod app;
mod parsing;
mod runtime;

pub use app::commands::run;
use crate::app::model::RuntimeSource;
use tauri::AppHandle;

/// Resolve runtime source from optional ZIP path argument.
///
/// # Errors
///
/// Returns an error when `zip_path` is missing or does not resolve to an existing ZIP file.
fn resolve_runtime_source(app: &AppHandle, input: Option<String>) -> Result<RuntimeSource, String> {
    match input {
        Some(raw) => {
            let resolved = parsing::dataset::resolve_zip_path(&raw)?;
            let managed = runtime::storage::ensure_managed_zip_copy(app, &resolved)?;
            Ok(RuntimeSource::ZipPath(managed))
        }
        None => {
            if let Some(found) = runtime::storage::preferred_managed_zip(app)? {
                return Ok(RuntimeSource::ZipPath(found));
            }
            Err("zip path is required (no managed zip cache found)".to_string())
        }
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

    #[test]
    fn configured_zip_runtime_smoke() {
        let Ok(path) = std::env::var("DOKHAN_TEST_ZIP") else {
            return;
        };
        let runtime = runtime::zip::parse_runtime_from_zip_with_progress(Path::new(&path), None)
            .expect("configured dictionary zip should parse");
        assert!(!runtime.contents.is_empty(), "expected table of contents");
        assert!(!runtime.entries.is_empty(), "expected dictionary entries");
    }

    /// Run with DOKHAN_TEST_ZIP and --ignored --nocapture to compare CHM changes.
    #[test]
    #[ignore]
    fn bench_chm_runtime_build() {
        use std::collections::HashSet;

        let path = std::env::var("DOKHAN_TEST_ZIP").expect("set DOKHAN_TEST_ZIP");
        let start = std::time::Instant::now();
        let mut parse_start = None;
        let mut parse_end = None;
        let mut on_progress = |p: app::model::BuildProgress| {
            if p.phase == "parse" && p.current == 0 {
                parse_start = Some(start.elapsed());
            } else if p.message == "Completed multithreaded parse" {
                parse_end = Some(start.elapsed());
            }
        };
        let runtime = runtime::zip::parse_runtime_from_zip_with_progress(
            Path::new(&path),
            Some(&mut on_progress),
        )
        .expect("benchmark dictionary should parse");
        let elapsed = start.elapsed();
        let targets = runtime
            .entries
            .iter()
            .filter(|entry| !entry.target_local.is_empty())
            .map(|entry| (&entry.source_path, &entry.target_local))
            .collect::<HashSet<_>>();
        let empty_targets = runtime
            .entries
            .iter()
            .filter(|entry| entry.target_local.is_empty())
            .count();
        let hydrated = runtime
            .entries
            .iter()
            .filter(|entry| !entry.definition_text.is_empty())
            .count();
        eprintln!(
            "CHM runtime build: {elapsed:?}, {} entries, {} unique targets, {empty_targets} empty targets, {hydrated} hydrated",
            runtime.entries.len(),
            targets.len()
        );
        if let (Some(scan), Some(parsed)) = (parse_start, parse_end) {
            eprintln!(
                "CHM build phases: scan={scan:?}, parallel_parse={:?}, finalize_and_keys={:?}",
                parsed - scan,
                elapsed - parsed
            );
        }
        let sample = runtime.entries.iter()
            .filter(|entry| entry.source_path == "merge17.chm" && !entry.definition_text.is_empty())
            .take(100)
            .cloned()
            .collect::<Vec<_>>();
        let start = std::time::Instant::now();
        let rendered = sample.into_iter()
            .map(|entry| runtime::zip::hydrate_zip_entry_detail(Path::new(&path), entry))
            .filter(|entry| !entry.definition_html.is_empty())
            .count();
        eprintln!("CHM detail HTML: {:?}, {rendered}/100 rendered", start.elapsed());
    }

    #[test]
    #[ignore]
    fn bench_chm_page_reads() {
        use std::io::Read;
        use std::sync::Arc;
        use std::time::Instant;

        let path = std::env::var("DOKHAN_TEST_ZIP").expect("set DOKHAN_TEST_ZIP");
        let file = std::fs::File::open(&path).expect("open benchmark zip");
        let mut zip = zip::ZipArchive::new(file).expect("parse benchmark zip");
        let mut bytes = Vec::new();
        zip.by_name("merge17.chm")
            .expect("merge17.chm")
            .read_to_end(&mut bytes)
            .expect("read CHM bytes");
        let shared: Arc<[u8]> = Arc::from(bytes);
        let start = Instant::now();
        let template = chm::ChmArchive::open(shared).expect("open CHM");
        let open_time = start.elapsed();
        let paths = template
            .entries()
            .iter()
            .filter(|e| e.path.ends_with(".html") || e.path.ends_with(".htm"))
            .take(100)
            .map(|e| e.path.clone())
            .collect::<Vec<_>>();
        assert!(!paths.is_empty());

        let start = Instant::now();
        let mut reusable = template.clone();
        let mut total_bytes = 0;
        for path in &paths {
            total_bytes += reusable.read_object(path).expect("read page").len();
        }
        let shared_time = start.elapsed();

        let start = Instant::now();
        for path in &paths {
            let mut fresh = template.clone();
            total_bytes += fresh.read_object(path).expect("read page").len();
        }
        let clone_time = start.elapsed();

        let start = Instant::now();
        for path_local in &paths {
            let archive = runtime::zip::open_named_chm_from_zip(Path::new(&path), "merge17.chm")
                .expect("open cached CHM");
            let mut archive = archive.lock().expect("lock cached CHM");
            total_bytes += archive.read_object(path_local).expect("read cached page").len();
        }
        let cached_time = start.elapsed();
        eprintln!(
            "CHM page reads: open={open_time:?}, reusable={shared_time:?}, fresh-clone={clone_time:?}, runtime-cache={cached_time:?}, pages={}, bytes={total_bytes}",
            paths.len()
        );
    }
}
