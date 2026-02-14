import { invoke } from '@tauri-apps/api/core';
import type {
  BuildStatus,
  ContentItem,
  ContentPage,
  DictionaryIndexEntry,
  DictionaryLinkTarget,
  EntryDetail,
  MasterFeatureSummary,
  SearchHit
} from '$lib/types/dictionary';

export function analyzeZipDataset(zipPath: string): Promise<MasterFeatureSummary> {
  return invoke<MasterFeatureSummary>('analyze_zip_dataset', { zipPath });
}

export function startMasterBuild(zipPath: string): Promise<string> {
  return invoke<string>('start_master_build', { zipPath });
}

export function getMasterBuildStatus(zipPath: string): Promise<BuildStatus> {
  return invoke<BuildStatus>('get_master_build_status', { zipPath });
}

export function getMasterContents(zipPath: string): Promise<ContentItem[]> {
  return invoke<ContentItem[]>('get_master_contents', { zipPath });
}

export function getIndexEntries(
  zipPath: string,
  prefix: string,
  limit: number | null = null
): Promise<DictionaryIndexEntry[]> {
  return invoke<DictionaryIndexEntry[]>('get_index_entries', { prefix, limit, zipPath });
}

export function searchEntries(
  zipPath: string,
  query: string,
  limit = 200
): Promise<SearchHit[]> {
  return invoke<SearchHit[]>('search_entries', { query, limit, zipPath });
}

export function getEntryDetail(zipPath: string, id: number): Promise<EntryDetail> {
  return invoke<EntryDetail>('get_entry_detail', { id, zipPath });
}

export function getContentPage(
  zipPath: string,
  local: string,
  sourcePath: string | null = null
): Promise<ContentPage> {
  return invoke<ContentPage>('get_content_page', { local, sourcePath, zipPath });
}

export function resolveLinkTarget(
  zipPath: string,
  href: string,
  currentSourcePath: string | null,
  currentLocal: string | null
): Promise<DictionaryLinkTarget> {
  return invoke<DictionaryLinkTarget>('resolve_link_target', {
    href,
    currentSourcePath,
    currentLocal,
    zipPath
  });
}

export function resolveMediaDataUrl(
  zipPath: string,
  href: string,
  currentSourcePath: string | null,
  currentLocal: string | null
): Promise<string> {
  return invoke<string>('resolve_media_data_url', {
    href,
    currentSourcePath,
    currentLocal,
    zipPath
  });
}
