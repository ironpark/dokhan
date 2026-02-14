//! In-memory index and full-text search utilities.
use std::collections::{BTreeMap, HashMap};
use std::sync::{Arc, Mutex, OnceLock};

use tantivy::collector::TopDocs;
use tantivy::doc;
use tantivy::query::QueryParser;
use tantivy::schema::{Field, Schema, TantivyDocument, Value, INDEXED, STORED, TEXT};
use tantivy::{Index, IndexReader, ReloadPolicy};

use crate::app::model::{DictionaryIndexEntry, EntryDetail, EntrySearchKey, SearchHit};
use crate::parsing::text::compact_ws;
use crate::app::model::RuntimeSource;
use crate::runtime::state::get_runtime;
use crate::resolve_runtime_source;

static SEARCH_CACHE: OnceLock<Mutex<BTreeMap<String, Arc<TantivySearchIndex>>>> = OnceLock::new();
static NORMALIZE_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
static NORMALIZE_LOOSE_CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
const NORMALIZE_CACHE_MAX: usize = 65_536;

#[derive(Clone)]
struct TantivySearchIndex {
    index: Index,
    reader: IndexReader,
    id_field: Field,
    headword_field: Field,
    aliases_field: Field,
    body_field: Field,
}

fn source_key(source: &RuntimeSource) -> String {
    match source {
        RuntimeSource::ZipPath(path) => format!(
            "zip:{}",
            path.canonicalize()
                .unwrap_or_else(|_| path.to_path_buf())
                .to_string_lossy()
        ),
    }
}

fn build_tantivy_index(entries: &[EntryDetail]) -> Result<TantivySearchIndex, String> {
    let mut schema_builder = Schema::builder();
    let id_field = schema_builder.add_u64_field("id", INDEXED | STORED);
    let headword_field = schema_builder.add_text_field("headword", TEXT);
    let aliases_field = schema_builder.add_text_field("aliases", TEXT);
    let body_field = schema_builder.add_text_field("body", TEXT);
    let schema = schema_builder.build();

    let index = Index::create_in_ram(schema);
    let mut writer = index
        .writer(50_000_000)
        .map_err(|e| format!("tantivy writer init failed: {e}"))?;
    for entry in entries {
        let aliases = entry.aliases.join(" ");
        writer.add_document(doc!(
            id_field => entry.id as u64,
            headword_field => entry.headword.clone(),
            aliases_field => aliases,
            body_field => entry.definition_text.clone()
        )).map_err(|e| format!("tantivy add doc failed: {e}"))?;
    }
    writer
        .commit()
        .map_err(|e| format!("tantivy commit failed: {e}"))?;
    let reader = index
        .reader_builder()
        .reload_policy(ReloadPolicy::Manual)
        .try_into()
        .map_err(|e| format!("tantivy reader init failed: {e}"))?;
    reader
        .reload()
        .map_err(|e| format!("tantivy reader reload failed: {e}"))?;

    Ok(TantivySearchIndex {
        index,
        reader,
        id_field,
        headword_field,
        aliases_field,
        body_field,
    })
}

fn get_or_build_tantivy_index(
    source: &RuntimeSource,
    entries: &[EntryDetail],
) -> Result<Arc<TantivySearchIndex>, String> {
    let key = source_key(source);
    let cache = SEARCH_CACHE.get_or_init(|| Mutex::new(BTreeMap::new()));
    {
        let guard = cache
            .lock()
            .map_err(|_| "search cache lock poisoned".to_string())?;
        if let Some(found) = guard.get(&key) {
            return Ok(found.clone());
        }
    }

    let built = Arc::new(build_tantivy_index(entries)?);
    let mut guard = cache
        .lock()
        .map_err(|_| "search cache lock poisoned".to_string())?;
    guard.insert(key, built.clone());
    Ok(built)
}

/// Build and cache Tantivy index for a runtime source eagerly.
///
/// # Errors
///
/// Returns an error when Tantivy index creation fails.
pub(crate) fn warm_search_index(
    source: &RuntimeSource,
    entries: &[EntryDetail],
) -> Result<(), String> {
    let _ = get_or_build_tantivy_index(source, entries)?;
    Ok(())
}

/// Normalize headword/search text with German orthography folding.
pub(crate) fn normalize_search_key(s: &str) -> String {
    if let Ok(mut cache) = NORMALIZE_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        if let Some(found) = cache.get(s) {
            return found.clone();
        }
        let normalized = s
            .to_lowercase()
            .replace("ä", "ae")
            .replace("ö", "oe")
            .replace("ü", "ue")
            .replace("ß", "ss");
        if cache.len() >= NORMALIZE_CACHE_MAX {
            cache.clear();
        }
        cache.insert(s.to_string(), normalized.clone());
        return normalized;
    }
    s.to_lowercase()
        .replace("ä", "ae")
        .replace("ö", "oe")
        .replace("ü", "ue")
        .replace("ß", "ss")
}

/// Looser normalization for prefix/contains tolerance (ae->a etc.).
pub(crate) fn normalize_search_key_loose(s: &str) -> String {
    if let Ok(mut cache) = NORMALIZE_LOOSE_CACHE
        .get_or_init(|| Mutex::new(HashMap::new()))
        .lock()
    {
        if let Some(found) = cache.get(s) {
            return found.clone();
        }
        let normalized = normalize_search_key(s)
            .replace("ae", "a")
            .replace("oe", "o")
            .replace("ue", "u");
        if cache.len() >= NORMALIZE_CACHE_MAX {
            cache.clear();
        }
        cache.insert(s.to_string(), normalized.clone());
        return normalized;
    }
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

fn search_entries_linear(query: &str, limit: usize, entries: &[EntryDetail], keys: &[EntrySearchKey]) -> Vec<SearchHit> {
    let terms = query
        .split_whitespace()
        .map(|x| (normalize_search_key(x), normalize_search_key_loose(x)))
        .collect::<Vec<_>>();

    let mut hits = Vec::<SearchHit>::new();
    for (e, k) in entries.iter().zip(keys.iter()) {
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
    hits
}

fn search_entries_tantivy(
    source: &RuntimeSource,
    query: &str,
    limit: usize,
    entries: &[EntryDetail],
) -> Result<Vec<SearchHit>, String> {
    let idx = get_or_build_tantivy_index(source, entries)?;
    let mut parser = QueryParser::for_index(
        &idx.index,
        vec![idx.headword_field, idx.aliases_field, idx.body_field],
    );
    parser.set_conjunction_by_default();
    let parsed = parser
        .parse_query(query)
        .map_err(|e| format!("tantivy query parse failed: {e}"))?;

    let searcher = idx.reader.searcher();
    let top_docs = searcher
        .search(&parsed, &TopDocs::with_limit(limit))
        .map_err(|e| format!("tantivy search failed: {e}"))?;
    let by_id = entries
        .iter()
        .map(|e| (e.id, e))
        .collect::<HashMap<usize, &EntryDetail>>();

    let mut out = Vec::<SearchHit>::new();
    for (score, addr) in top_docs {
        let doc: TantivyDocument = searcher
            .doc(addr)
            .map_err(|e| format!("tantivy doc read failed: {e}"))?;
        let Some(id) = doc
            .get_first(idx.id_field)
            .and_then(|v| v.as_u64())
            .map(|v| v as usize)
        else {
            continue;
        };
        let Some(entry) = by_id.get(&id) else {
            continue;
        };
        out.push(SearchHit {
            id: entry.id,
            headword: entry.headword.clone(),
            source_path: entry.source_path.clone(),
            score: score.max(0.0).round() as usize,
            snippet: entry.definition_text.chars().take(180).collect::<String>(),
        });
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
    let limit = limit.unwrap_or(50).clamp(1, 200);
    match search_entries_tantivy(&source, &q, limit, &runtime.entries) {
        Ok(v) => Ok(v),
        Err(_) => Ok(search_entries_linear(
            &q,
            limit,
            &runtime.entries,
            &runtime.entry_keys,
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::normalize_search_key;

    #[test]
    fn normalize_search_key_handles_upper_umlaut() {
        assert_eq!(normalize_search_key("Äpfel"), "aepfel");
        assert_eq!(normalize_search_key("Öl"), "oel");
        assert_eq!(normalize_search_key("Übung"), "uebung");
    }
}
