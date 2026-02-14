use base64::Engine as _;

use crate::chm;
use crate::app::model::{LinkTarget, RuntimeSource};
use crate::parsing::text::path_stem;
use crate::runtime::search::{eq_search_key, normalize_search_key};
use crate::runtime::state::get_runtime;
use crate::runtime::zip::read_named_chm_from_zip;
use crate::resolve_runtime_source;

fn extract_chm_name(prefix: &str) -> Option<String> {
    let lower = prefix.to_ascii_lowercase();
    let end = lower.find(".chm")? + 4;
    let upto = &prefix[..end];
    let name = upto
        .rsplit([':', '/', '\\'])
        .next()
        .map(str::trim)
        .unwrap_or_default();
    if name.is_empty() {
        return None;
    }
    Some(name.to_ascii_lowercase())
}

pub(crate) fn parse_internal_ref(raw_ref: &str) -> Option<(Option<String>, String, bool)> {
    let raw = raw_ref.trim();
    if raw.is_empty() {
        return None;
    }
    let lower = raw.to_ascii_lowercase();
    if lower.starts_with("http://")
        || lower.starts_with("https://")
        || lower.starts_with("mailto:")
        || lower.starts_with("javascript:")
        || lower.starts_with('#')
        || lower.starts_with("data:")
    {
        return None;
    }

    let mut value = raw;
    let mut source_override = None;
    let mut is_absolute = false;
    if let Some((left, right)) = raw.split_once("::/") {
        source_override = extract_chm_name(left);
        value = right;
        is_absolute = true;
    }

    if value.starts_with('/') {
        is_absolute = true;
    }
    let value = value
        .split(['#', '?'])
        .next()
        .unwrap_or(value)
        .replace('\\', "/");
    let value = value.trim().trim_start_matches('/');
    let value = value.trim_start_matches("./");
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    Some((source_override, value.to_string(), is_absolute))
}

pub(crate) fn normalize_path(input: &str) -> String {
    let mut out = Vec::<&str>::new();
    for seg in input.split('/') {
        if seg.is_empty() || seg == "." {
            continue;
        }
        if seg == ".." {
            out.pop();
            continue;
        }
        out.push(seg);
    }
    out.join("/")
}

fn resolve_relative_local(local: &str, current_local: Option<&str>, is_absolute: bool) -> String {
    if is_absolute {
        return normalize_path(local);
    }
    if let Some(base) = current_local {
        if let Some((dir, _)) = base.replace('\\', "/").rsplit_once('/') {
            return normalize_path(&format!("{dir}/{local}"));
        }
    }
    normalize_path(local)
}

fn mime_from_path(path: &str) -> &'static str {
    let lower = path.to_ascii_lowercase();
    if lower.ends_with(".jpg") || lower.ends_with(".jpeg") {
        return "image/jpeg";
    }
    if lower.ends_with(".png") {
        return "image/png";
    }
    if lower.ends_with(".gif") {
        return "image/gif";
    }
    if lower.ends_with(".webp") {
        return "image/webp";
    }
    if lower.ends_with(".bmp") {
        return "image/bmp";
    }
    if lower.ends_with(".svg") {
        return "image/svg+xml";
    }
    if lower.ends_with(".ico") {
        return "image/x-icon";
    }
    "application/octet-stream"
}

pub(crate) fn read_chm_binary_object(chm: &mut chm::ChmArchive, local: &str) -> Option<Vec<u8>> {
    let path = local.trim().trim_start_matches('/');
    if path.is_empty() {
        return None;
    }
    if let Ok(v) = chm.read_object(path) {
        return Some(v);
    }
    let slash = format!("/{path}");
    if let Ok(v) = chm.read_object(&slash) {
        return Some(v);
    }
    let needle = path.to_ascii_lowercase();
    let base = path.rsplit('/').next().unwrap_or(path).to_ascii_lowercase();
    let mut matches = chm
        .entries()
        .iter()
        .filter_map(|e| {
            let entry_lower = e.path.trim_start_matches('/').to_ascii_lowercase();
            let entry_base = entry_lower.rsplit('/').next().unwrap_or(&entry_lower);
            if entry_lower == needle || entry_base == base {
                Some(e.path.clone())
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    matches.sort();
    matches.dedup();
    for p in matches {
        if let Ok(v) = chm.read_object(&p) {
            return Some(v);
        }
    }
    None
}

fn resolve_media_data_url_inner(
    href: &str,
    current_source_path: Option<&str>,
    current_local: Option<&str>,
    zip_path: Option<String>,
) -> Result<String, String> {
    let Some((source_override, local_raw, is_absolute)) = parse_internal_ref(href) else {
        return Err(format!("unsupported media href: {href}"));
    };
    let resolved_local = resolve_relative_local(&local_raw, current_local, is_absolute);
    let source_path = source_override
        .or(current_source_path.map(|x| x.to_ascii_lowercase()))
        .unwrap_or_else(|| "master.chm".to_string());
    let bytes = match resolve_runtime_source(zip_path)? {
        RuntimeSource::ZipPath(zip_path) => {
            let chm_bytes = read_named_chm_from_zip(&zip_path, &source_path)?;
            let mut chm = chm::ChmArchive::open(chm_bytes)
                .map_err(|e| format!("failed to open {source_path}: {e}"))?;
            read_chm_binary_object(&mut chm, &resolved_local)
                .ok_or_else(|| format!("asset not found in {source_path}: {resolved_local}"))?
        }
    };
    let mime = mime_from_path(&resolved_local);
    let encoded = base64::engine::general_purpose::STANDARD.encode(bytes);
    Ok(format!("data:{mime};base64,{encoded}"))
}

pub(crate) fn resolve_media_data_url_impl(
    href: &str,
    current_source_path: Option<&str>,
    current_local: Option<&str>,
    zip_path: Option<String>,
) -> Result<String, String> {
    resolve_media_data_url_inner(href, current_source_path, current_local, zip_path)
}

pub(crate) fn resolve_link_target_impl(
    href: &str,
    current_source_path: Option<&str>,
    current_local: Option<&str>,
    zip_path: Option<String>,
) -> Result<LinkTarget, String> {
    let Some((source_override, local_raw, is_absolute)) = parse_internal_ref(href) else {
        return Err(format!("unsupported or empty href: {href}"));
    };
    let source_context = source_override
        .or(current_source_path.map(|x| x.to_ascii_lowercase()))
        .unwrap_or_else(|| "master.chm".to_string());
    let local_path = resolve_relative_local(&local_raw, current_local, is_absolute);
    let source = resolve_runtime_source(zip_path)?;
    let runtime = get_runtime(&source)?;

    let local_lower = local_path.to_ascii_lowercase();
    let local_stem_key = normalize_search_key(&path_stem(&local_path));

    if source_context == "master.chm" {
        if let Some(item) = runtime.contents.iter().find(|item| {
            let item_local = item.local.trim_start_matches('/');
            let item_lower = item_local.to_ascii_lowercase();
            if item_lower == local_lower {
                return true;
            }
            let item_stem_key = normalize_search_key(&path_stem(item_local));
            eq_search_key(&item_stem_key, &local_stem_key)
        }) {
            return Ok(LinkTarget::Content {
                local: item.local.clone(),
                source_path: "master.chm".to_string(),
            });
        }
    }

    if let Some(entry) = runtime.entries.iter().find(|entry| {
        entry.source_path.to_ascii_lowercase() == source_context
            && (eq_search_key(&entry.headword, &local_stem_key)
                || entry
                    .aliases
                    .iter()
                    .any(|alias| eq_search_key(alias, &local_stem_key)))
    }) {
        return Ok(LinkTarget::Entry { id: entry.id });
    }

    if let Some(entry) = runtime.entries.iter().find(|entry| {
        if eq_search_key(&entry.headword, &local_stem_key) {
            return true;
        }
        entry
            .aliases
            .iter()
            .any(|alias| eq_search_key(alias, &local_stem_key))
    }) {
        return Ok(LinkTarget::Entry { id: entry.id });
    }

    Ok(LinkTarget::Content {
        local: local_path,
        source_path: source_context,
    })
}
