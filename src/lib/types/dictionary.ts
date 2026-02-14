export type MasterFeatureSummary = {
  debugRoot: string;
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
  aliases: string[];
  sourcePath: string;
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

export type Tab = 'content' | 'index' | 'search';
export type DetailMode = 'content' | 'entry' | 'none';
