export type MasterFeatureSummary = {
  zipPath: string;
  contentCount: number;
  indexCount: number;
};

export type ContentItem = {
  title: string;
  local: string;
};

export type ContentPage = {
  local: string;
  sourcePath: string;
  title: string;
  bodyText: string;
  bodyHtml: string;
};

export type DictionaryIndexEntry = {
  id: number;
  headword: string;
  headwordHighlights: Array<{ start: number; end: number }>;
  aliases: string[];
  sourcePath: string;
};

export type FavoriteItem = {
  key: string;
  kind: "entry" | "content";
  label: string;
  id: number | null;
  local: string | null;
  sourcePath: string | null;
};

export type RecentViewItem = {
  key: string;
  kind: "entry" | "content";
  label: string;
  id: number | null;
  local: string | null;
  sourcePath: string | null;
  viewedAt: number;
};

export type SearchHit = {
  id: number;
  headword: string;
  sourcePath: string;
  score: number;
  snippet: string;
};

export type EntryDetail = {
  id: number;
  headword: string;
  aliases: string[];
  sourcePath: string;
  definitionText: string;
  definitionHtml: string;
};

export type ReaderFontSize = 'sm' | 'md' | 'lg';
export type ReaderLineHeight = 'tight' | 'normal' | 'loose';
export type ReaderWidth = 'narrow' | 'normal' | 'wide';

export type DictionaryLinkTarget =
  | { kind: 'content'; local: string; sourcePath: string }
  | { kind: 'entry'; id: number };

export type BuildProgress = {
  phase: string;
  current: number;
  total: number;
  message: string;
};

export type BuildStatus = {
  phase: string;
  current: number;
  total: number;
  message: string;
  done: boolean;
  success: boolean;
  error: string | null;
  summary: MasterFeatureSummary | null;
};

export type Tab = 'content' | 'index' | 'search' | 'favorites';
export type DetailMode = 'content' | 'entry' | 'none';
