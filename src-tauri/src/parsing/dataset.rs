//! ZIP dataset path resolution and summary/statistics extraction.
#[cfg(test)]
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::path::{Path, PathBuf};

use zip::ZipArchive;

#[cfg(test)]
use crate::app::model::{DatasetSummary, FileTypeCount, MainVolumeCoverage};

#[cfg(test)]
fn basename_lower(name: &str) -> String {
    name.rsplit(['/', '\\'])
        .next()
        .unwrap_or(name)
        .to_ascii_lowercase()
}

/// Open ZIP archive from path with user-facing error mapping.
///
/// # Errors
///
/// Returns an error when the ZIP file cannot be opened or parsed.
pub(crate) fn open_zip_archive(path: &Path) -> Result<ZipArchive<File>, String> {
    let file = File::open(path).map_err(|e| format!("failed to open zip: {e}"))?;
    ZipArchive::new(file).map_err(|e| format!("invalid zip archive: {e}"))
}

/// Compute coverage for merge01..merge36 including split volumes.
#[cfg(test)]
pub(crate) fn build_main_volume_coverage(names: &[String]) -> Vec<MainVolumeCoverage> {
    let base_names = names.iter().map(|name| basename_lower(name)).collect::<Vec<_>>();
    let name_set: BTreeSet<_> = base_names.iter().cloned().collect();
    (1..=36)
        .map(|n| {
            let main_file = format!("merge{n:02}.chm");
            let split_prefix = format!("merge{n:02}-");
            let has_main_file = name_set.contains(&main_file);
            let split_file_count = base_names
                .iter()
                .filter(|name| name.ends_with(".chm") && name.starts_with(&split_prefix))
                .count();
            MainVolumeCoverage {
                volume: n as u8,
                has_main_file,
                split_file_count,
                covered: has_main_file || split_file_count > 0,
            }
        })
        .collect::<Vec<_>>()
}

/// Summarize dataset ZIP structure and extension statistics.
///
/// # Errors
///
/// Returns an error when ZIP iteration fails.
#[cfg(test)]
pub(crate) fn summarize_zip(zip_path: &Path) -> Result<DatasetSummary, String> {
    let mut archive = open_zip_archive(zip_path)?;

    let mut total_files = 0usize;
    let mut total_dirs = 0usize;
    let mut uncompressed_bytes = 0u64;
    let mut compressed_bytes = 0u64;
    let mut names = Vec::with_capacity(archive.len());
    let mut ext_counts: BTreeMap<String, usize> = BTreeMap::new();

    for i in 0..archive.len() {
        let entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry #{i}: {e}"))?;
        let name = entry.name().to_string();

        if entry.is_dir() {
            total_dirs += 1;
        } else {
            total_files += 1;
            uncompressed_bytes += entry.size();
            compressed_bytes += entry.compressed_size();

            let ext = name
                .rsplit_once('.')
                .map(|(_, ext)| ext.to_ascii_lowercase())
                .unwrap_or_default();
            *ext_counts.entry(ext).or_insert(0) += 1;
        }
        names.push(name);
    }

    let base_names = names.iter().map(|name| basename_lower(name)).collect::<Vec<_>>();
    let name_set: BTreeSet<_> = base_names.iter().cloned().collect();
    let missing_main_files_only = (1..=36)
        .map(|n| format!("merge{n:02}.chm"))
        .filter(|name| !name_set.contains(name))
        .collect::<Vec<_>>();

    let main_volume_coverage = build_main_volume_coverage(&names);
    let missing_main_volumes = main_volume_coverage
        .iter()
        .filter(|v| !v.covered)
        .map(|v| format!("merge{:02}.chm", v.volume))
        .collect::<Vec<_>>();

    let sample_chm_files = names
        .iter()
        .filter(|name| name.to_ascii_lowercase().ends_with(".chm"))
        .take(8)
        .cloned()
        .collect::<Vec<_>>();

    let extension_counts = ext_counts
        .into_iter()
        .map(|(extension, count)| FileTypeCount { extension, count })
        .collect::<Vec<_>>();

    Ok(DatasetSummary {
        zip_path: zip_path.to_string_lossy().to_string(),
        total_entries: archive.len(),
        total_files,
        total_dirs,
        chm_count: names
            .iter()
            .filter(|name| name.to_ascii_lowercase().ends_with(".chm"))
            .count(),
        lnk_count: names
            .iter()
            .filter(|name| name.to_ascii_lowercase().ends_with(".lnk"))
            .count(),
        txt_count: names
            .iter()
            .filter(|name| name.to_ascii_lowercase().ends_with(".txt"))
            .count(),
        uncompressed_bytes,
        compressed_bytes,
        compression_ratio: if uncompressed_bytes == 0 {
            0.0
        } else {
            compressed_bytes as f64 / uncompressed_bytes as f64
        },
        has_master_chm: name_set.contains("master.chm"),
        has_readme_txt: name_set.contains("readme.txt"),
        missing_main_volumes,
        missing_main_files_only,
        main_volume_coverage,
        extension_counts,
        sample_chm_files,
    })
}

/// Resolve input ZIP path across cwd and ancestor directories.
///
/// # Errors
///
/// Returns an error when the path cannot be found from cwd/ancestor candidates.
pub(crate) fn resolve_zip_path(input: &str) -> Result<PathBuf, String> {
    let raw = PathBuf::from(input);
    if raw.is_absolute() && raw.exists() {
        return Ok(raw);
    }
    if raw.exists() {
        return Ok(raw);
    }

    let cwd = std::env::current_dir().map_err(|e| format!("failed to read current dir: {e}"))?;
    let mut attempts = Vec::new();
    for ancestor in cwd.ancestors() {
        let candidate = ancestor.join(&raw);
        attempts.push(candidate.display().to_string());
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    Err(format!(
        "zip not found: '{input}'. searched current/ancestor paths from '{}'. attempts: {}",
        cwd.display(),
        attempts.join(", ")
    ))
}

#[cfg(test)]
mod tests {
    use super::build_main_volume_coverage;

    #[test]
    fn split_coverage_handles_prefixed_zip_paths() {
        let names = vec![
            "dictionary_v77/merge01.chm".to_string(),
            "dictionary_v77/merge03-01.chm".to_string(),
        ];
        let coverage = build_main_volume_coverage(&names);
        let vol1 = coverage.iter().find(|x| x.volume == 1).expect("v1");
        let vol3 = coverage.iter().find(|x| x.volume == 3).expect("v3");
        assert!(vol1.covered && vol1.has_main_file);
        assert!(vol3.covered && !vol3.has_main_file && vol3.split_file_count == 1);
    }
}
