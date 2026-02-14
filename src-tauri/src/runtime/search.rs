//! In-memory index and full-text search utilities.
use crate::app::model::{DictionaryIndexEntry, EntryDetail, EntrySearchKey, SearchHit};
use crate::parsing::text::compact_ws;
use crate::runtime::state::get_runtime;
use crate::resolve_runtime_source;

/// Normalize headword/search text with German orthography folding.
pub(crate) fn normalize_search_key(s: &str) -> String {
    s.to_ascii_lowercase()
        .replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ü", "ue")
        .replace("ß", "ss")
}

/// Looser normalization for prefix/contains tolerance (ae->a etc.).
pub(crate) fn normalize_search_key_loose(s: &str) -> String {
    normalize_search_key(s)
        .replace("ae", "a")
        .replace("oe", "o")
        .replace("ue", "u")
}

/// Check prefix match against strict+loose normalized variants.
fn starts_with_search_key_precomputed(
    value_key: &str,
    value_loose: &str,
    prefix_key: &str,
    prefix_loose: &str,
) -> bool {
    value_key.starts_with(prefix_key) || value_loose.starts_with(prefix_loose)
}

/// Check contains match against strict+loose normalized variants.
fn contains_search_key_precomputed(
    value_key: &str,
    value_loose: &str,
    term_key: &str,
    term_loose: &str,
) -> bool {
    value_key.contains(term_key) || value_loose.contains(term_loose)
}

/// Compare two terms under strict and loose normalization.
pub(crate) fn eq_search_key(a: &str, b: &str) -> bool {
    if normalize_search_key(a) == normalize_search_key(b) {
        return true;
    }
    normalize_search_key_loose(a) == normalize_search_key_loose(b)
}

/// Precompute normalized search fields to speed lookups.
pub(crate) fn build_entry_search_keys(entries: &[EntryDetail]) -> Vec<EntrySearchKey> {
    entries
        .iter()
        .map(|e| {
            let aliases = e.aliases.iter().map(|a| normalize_search_key(a)).collect::<Vec<_>>();
            let aliases_loose = e
                .aliases
                .iter()
                .map(|a| normalize_search_key_loose(a))
                .collect::<Vec<_>>();
            EntrySearchKey {
                headword: normalize_search_key(&e.headword),
                headword_loose: normalize_search_key_loose(&e.headword),
                body: normalize_search_key(&e.definition_text),
                body_loose: normalize_search_key_loose(&e.definition_text),
                aliases,
                aliases_loose,
            }
        })
        .collect::<Vec<_>>()
}

/// Return index rows, optionally filtered by prefix.
pub(crate) fn get_index_entries_impl(
    prefix: Option<String>,
    limit: Option<usize>,
    zip_path: Option<String>,
) -> Result<Vec<DictionaryIndexEntry>, String> {
    let source = resolve_runtime_source(zip_path)?;
    let runtime = get_runtime(&source)?;
    let p = prefix.unwrap_or_default();
    let p_key = normalize_search_key(&p);
    let p_loose = normalize_search_key_loose(&p);
    let limit = if p.is_empty() {
        limit.unwrap_or(runtime.entries.len()).clamp(1, runtime.entries.len().max(1))
    } else {
        limit.unwrap_or(200).clamp(1, 5_000)
    };

    let mut out = Vec::new();
    for (e, k) in runtime.entries.iter().zip(runtime.entry_keys.iter()) {
        if p.is_empty()
            || starts_with_search_key_precomputed(&k.headword, &k.headword_loose, &p_key, &p_loose)
        {
            out.push(DictionaryIndexEntry {
                id: e.id,
                headword: e.headword.clone(),
                aliases: e.aliases.clone(),
                source_path: e.source_path.clone(),
            });
            if out.len() >= limit {
                break;
            }
        }
    }
    Ok(out)
}

/// Execute weighted in-memory search over headword/aliases/body.
pub(crate) fn search_entries_impl(
    query: &str,
    limit: Option<usize>,
    zip_path: Option<String>,
) -> Result<Vec<SearchHit>, String> {
    let q = compact_ws(query);
    if q.is_empty() {
        return Ok(Vec::new());
    }
    let source = resolve_runtime_source(zip_path)?;
    let runtime = get_runtime(&source)?;
    let terms = q
        .split_whitespace()
        .map(|x| (normalize_search_key(x), normalize_search_key_loose(x)))
        .collect::<Vec<_>>();
    let limit = limit.unwrap_or(50).clamp(1, 200);

    let mut hits = Vec::<SearchHit>::new();
    for (e, k) in runtime.entries.iter().zip(runtime.entry_keys.iter()) {
        let mut score = 0usize;
        let mut ok = true;
        for (t_key, t_loose) in &terms {
            let in_head = contains_search_key_precomputed(&k.headword, &k.headword_loose, t_key, t_loose)
                || k.aliases
                    .iter()
                    .zip(k.aliases_loose.iter())
                    .any(|(a, al)| contains_search_key_precomputed(a, al, t_key, t_loose));
            let in_body =
                contains_search_key_precomputed(&k.body, &k.body_loose, t_key, t_loose);
            if !(in_head || in_body) {
                ok = false;
                break;
            }
            if in_head {
                score += 5;
            }
            if in_body {
                score += 2;
            }
        }
        if ok {
            let snippet = e.definition_text.chars().take(180).collect::<String>();
            hits.push(SearchHit {
                id: e.id,
                headword: e.headword.clone(),
                source_path: e.source_path.clone(),
                score,
                snippet,
            });
        }
    }
    hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| a.headword.cmp(&b.headword)));
    hits.truncate(limit);
    Ok(hits)
}
