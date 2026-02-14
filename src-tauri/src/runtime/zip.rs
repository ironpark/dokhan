//! ZIP-backed CHM reading and runtime index construction.
use std::collections::BTreeMap;
use std::path::Path;

use crate::chm;
use crate::app::model::{BuildProgress, ContentItem, ContentPage, EntryDetail, RuntimeIndex};
use crate::parsing::dataset::open_zip_archive;
use crate::parsing::index::{extract_index_entries_from_chm_bytes, parse_master_hhc_text};
use crate::parsing::text::{
    body_html, compact_ws, decode_euc_kr, extract_first_bold_text, find_all_tag_values,
    first_paragraph_html, strip_html_tags,
};
use crate::runtime::link_media::read_chm_binary_object;
use crate::runtime::search::normalize_search_key;
use crate::runtime::state::build_runtime_index;

/// Decode CHM page bytes into normalized content payload.
fn decode_content_page(local: String, source_path: String, bytes: &[u8]) -> ContentPage {
    let text = decode_euc_kr(bytes);
    let title = find_all_tag_values(&text, "title")
        .into_iter()
        .map(|x| compact_ws(&strip_html_tags(&x)))
        .find(|x| !x.is_empty())
        .unwrap_or_else(|| local.clone());
    let b_html = body_html(&text).unwrap_or_default();
    let b_text = compact_ws(&strip_html_tags(&b_html));
    ContentPage {
        local,
        source_path,
        title,
        body_text: b_text,
        body_html: b_html,
    }
}

/// Build candidate local paths for CHM object resolution.
fn resolve_local_candidates(local: &str) -> Vec<String> {
    let raw = local.trim().trim_start_matches('/').to_string();
    let mut out = Vec::new();
    if !raw.is_empty() {
        out.push(raw.clone());
    }
    let raw_lower = raw.to_ascii_lowercase();
    if !raw_lower.contains('.') {
        out.push(format!("{raw}.html"));
        out.push(format!("{raw}.htm"));
    }
    if raw_lower == "master" {
        out.push("master.html".to_string());
        out.push("master.htm".to_string());
    }
    out.sort();
    out.dedup();
    out
}

/// Read CHM object with filename and basename fallback candidates.
fn read_chm_object_with_candidates(chm: &mut chm::ChmArchive, local: &str) -> Option<Vec<u8>> {
    let candidates = resolve_local_candidates(local);
    for c in &candidates {
        if let Ok(v) = chm.read_object(c) {
            return Some(v);
        }
        let slash = format!("/{c}");
        if let Ok(v) = chm.read_object(&slash) {
            return Some(v);
        }
    }

    let candidate_lowers = candidates
        .iter()
        .map(|x| x.to_ascii_lowercase())
        .collect::<Vec<_>>();
    let matched_paths = chm
        .entries()
        .iter()
        .filter_map(|e| {
            let base = e.path.rsplit('/').next().unwrap_or(&e.path).to_ascii_lowercase();
            if candidate_lowers.iter().any(|c| c == &base) {
                Some(e.path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    for p in matched_paths {
        if let Ok(v) = chm.read_object(&p) {
            return Some(v);
        }
    }
    None
}

/// Read raw CHM file bytes from dataset ZIP by filename.
///
/// # Errors
///
/// Returns an error when the ZIP cannot be opened/read or the named CHM does not exist.
pub(crate) fn read_named_chm_from_zip(zip_path: &Path, chm_name: &str) -> Result<Vec<u8>, String> {
    let mut archive = open_zip_archive(zip_path)?;
    let target = chm_name.to_ascii_lowercase();
    for i in 0..archive.len() {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry #{i}: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let entry_base = entry
            .name()
            .rsplit(['/', '\\'])
            .next()
            .unwrap_or(entry.name())
            .to_ascii_lowercase();
        if entry_base != target {
            continue;
        }
        let mut bytes = Vec::new();
        std::io::copy(&mut entry, &mut bytes)
            .map_err(|e| format!("failed to load {chm_name} from zip: {e}"))?;
        return Ok(bytes);
    }
    Err(format!("chm not found in zip: {chm_name}"))
}

/// Resolve entry HTML bytes using target local or headword fallback.
fn read_entry_html_from_chm(chm: &mut chm::ChmArchive, headword: &str) -> Option<Vec<u8>> {
    for candidate in resolve_local_candidates(headword) {
        if let Ok(v) = chm.read_object(&candidate) {
            return Some(v);
        }
        let slash = format!("/{candidate}");
        if let Ok(v) = chm.read_object(&slash) {
            return Some(v);
        }
    }

    let wanted = normalize_search_key(headword);
    let mut matches = chm
        .entries()
        .iter()
        .filter_map(|e| {
            let base = e.path.rsplit('/').next().unwrap_or(&e.path);
            let stem = base
                .rsplit_once('.')
                .map_or(base, |(stem, _)| stem);
            let ext = base
                .rsplit_once('.')
                .map(|(_, x)| x.to_ascii_lowercase())
                .unwrap_or_default();
            if (ext == "htm" || ext == "html") && normalize_search_key(stem) == wanted {
                Some(e.path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    for path in matches {
        if let Ok(v) = chm.read_object(&path) {
            return Some(v);
        }
    }
    None
}

/// Fill empty entry body fields by reading original CHM HTML.
pub(crate) fn hydrate_zip_entry_detail(zip_path: &Path, mut entry: EntryDetail) -> EntryDetail {
    if !entry.definition_text.is_empty() {
        return entry;
    }
    let Ok(chm_bytes) = read_named_chm_from_zip(zip_path, &entry.source_path) else {
        return entry;
    };
    let Ok(mut chm) = chm::ChmArchive::open(chm_bytes) else {
        return entry;
    };
    let html_bytes = if entry.target_local.is_empty() {
        read_entry_html_from_chm(&mut chm, &entry.headword)
    } else {
        read_chm_binary_object(&mut chm, &entry.target_local)
            .or_else(|| read_entry_html_from_chm(&mut chm, &entry.headword))
    };
    let Some(html_bytes) = html_bytes else {
        return entry;
    };

    let html_text = decode_euc_kr(&html_bytes);
    let paragraph_html = first_paragraph_html(&html_text).unwrap_or_default();
    let paragraph_text = compact_ws(&strip_html_tags(&paragraph_html));
    let body = body_html(&html_text).unwrap_or_default();
    let body_text = compact_ws(&strip_html_tags(&body));
    if !paragraph_html.is_empty() {
        entry.definition_html = paragraph_html.clone();
    } else if !body.is_empty() {
        entry.definition_html = body.clone();
    }
    if !paragraph_text.is_empty() {
        entry.definition_text = paragraph_text;
    } else if !body_text.is_empty() {
        entry.definition_text = body_text;
    }

    let title_aliases = find_all_tag_values(&html_text, "title")
        .into_iter()
        .map(|x| compact_ws(&strip_html_tags(&x)))
        .filter(|x| !x.is_empty())
        .collect::<Vec<_>>();
    for alias in title_aliases {
        if !entry.aliases.contains(&alias) {
            entry.aliases.push(alias);
        }
    }
    if let Some(bold) = extract_first_bold_text(&html_text) {
        let bold = compact_ws(&bold);
        if !bold.is_empty() && !entry.aliases.contains(&bold) {
            entry.aliases.push(bold);
        }
    }
    entry
}

/// Read and decode content page from ZIP-contained CHM.
///
/// # Errors
///
/// Returns an error when the CHM cannot be loaded/opened or the target page cannot be resolved.
pub(crate) fn read_content_page_from_zip(
    zip_path: &Path,
    source_path: &str,
    local: &str,
) -> Result<ContentPage, String> {
    let bytes = read_named_chm_from_zip(zip_path, source_path)?;
    let mut chm = chm::ChmArchive::open(bytes).map_err(|e| format!("failed to open {source_path}: {e}"))?;
    if let Some(v) = read_chm_object_with_candidates(&mut chm, local) {
        return Ok(decode_content_page(local.to_string(), source_path.to_string(), &v));
    }
    Err(format!(
        "content page not found in zip runtime: {source_path}::{local}"
    ))
}

/// Parse full runtime index from ZIP and emit progress events.
///
/// The callback receives best-effort progress snapshots during CHM iteration.
///
/// # Errors
///
/// Returns an error when ZIP/CHM reading fails during runtime construction.
pub(crate) fn parse_runtime_from_zip_with_progress(
    zip_path: &Path,
    mut progress: Option<&mut dyn FnMut(BuildProgress)>,
) -> Result<RuntimeIndex, String> {
    let mut archive = open_zip_archive(zip_path)?;
    let total = archive.len();
    let mut entries = Vec::<EntryDetail>::new();
    let mut contents = Vec::<ContentItem>::new();

    for i in 0..total {
        let mut entry = archive
            .by_index(i)
            .map_err(|e| format!("failed to read zip entry #{i}: {e}"))?;
        if entry.is_dir() {
            continue;
        }
        let name = entry.name().to_string();
        let lower = name.to_ascii_lowercase();
        if !lower.ends_with(".chm") {
            continue;
        }
        let mut bytes = Vec::new();
        std::io::copy(&mut entry, &mut bytes).map_err(|e| format!("failed to load {name}: {e}"))?;

        if lower.ends_with("master.chm") {
            if let Ok(mut chm) = chm::ChmArchive::open(bytes.clone()) {
                if let Some(hhc) = read_chm_object_with_candidates(&mut chm, "master.hhc") {
                    let text = decode_euc_kr(&hhc);
                    let parsed = parse_master_hhc_text(&text);
                    if !parsed.is_empty() {
                        contents = parsed;
                    }
                }
            }
            if contents.is_empty() {
                contents.push(ContentItem {
                    title: "목차".to_string(),
                    local: "master".to_string(),
                });
            }
        }

        if lower.starts_with("merge") {
            entries.extend(extract_index_entries_from_chm_bytes(&name, &bytes));
        }

        if let Some(cb) = progress.as_mut() {
            cb(BuildProgress {
                phase: "parse".to_string(),
                current: i + 1,
                total,
                message: format!("Parsing {name}"),
            });
        }
    }

    entries.sort_by(|a, b| {
        normalize_search_key(&a.headword)
            .cmp(&normalize_search_key(&b.headword))
            .then_with(|| a.source_path.cmp(&b.source_path))
            .then_with(|| a.target_local.cmp(&b.target_local))
    });
    entries.dedup_by(|a, b| {
        normalize_search_key(&a.headword) == normalize_search_key(&b.headword)
            && a.source_path == b.source_path
            && a.target_local == b.target_local
    });
    for (i, e) in entries.iter_mut().enumerate() {
        e.id = i + 1;
    }
    Ok(build_runtime_index(contents, entries, BTreeMap::new()))
}
