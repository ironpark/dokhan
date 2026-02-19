import type {
  FavoriteItem,
  ReaderFontSize,
  ReaderLineHeight,
  ReaderWidth,
  RecentViewItem,
} from "$lib/types/dictionary";

export const MAX_RECENT_SEARCHES = 10;
export const MAX_RECENT_VIEWS = 20;
export const MAX_FAVORITES = 100;

const PREFS_KEY = "dokhan:user-prefs";
const READER_LINE_HEIGHTS: ReaderLineHeight[] = ["tight", "normal", "loose"];
const READER_WIDTHS: ReaderWidth[] = ["narrow", "normal", "wide"];

type DictionaryPrefsPayload = {
  recentSearches: string[];
  recentViews: RecentViewItem[];
  favorites: FavoriteItem[];
  preprocessEnabled: boolean;
  markerPreprocessEnabled: boolean;
  readerFontSize: ReaderFontSize;
  readerLineHeight: ReaderLineHeight;
  readerWidth: ReaderWidth;
};

const DEFAULT_PREFS: DictionaryPrefsPayload = {
  recentSearches: [],
  recentViews: [],
  favorites: [],
  preprocessEnabled: true,
  markerPreprocessEnabled: true,
  readerFontSize: 100,
  readerLineHeight: "normal",
  readerWidth: "normal",
};

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === "object" && value !== null;
}

function isRecentViewItem(value: unknown): value is RecentViewItem {
  if (!isRecord(value)) return false;
  return (
    typeof value.key === "string"
    && (value.kind === "content" || value.kind === "entry")
    && typeof value.label === "string"
    && (value.id === null || typeof value.id === "number")
    && (value.local === null || typeof value.local === "string")
    && (value.sourcePath === null || typeof value.sourcePath === "string")
    && typeof value.viewedAt === "number"
  );
}

function isFavoriteItem(value: unknown): value is FavoriteItem {
  if (!isRecord(value)) return false;
  return (
    typeof value.key === "string"
    && (value.kind === "content" || value.kind === "entry")
    && typeof value.label === "string"
    && (value.id === null || typeof value.id === "number")
    && (value.local === null || typeof value.local === "string")
    && (value.sourcePath === null || typeof value.sourcePath === "string")
  );
}

export function dedupeRecentViews(rows: RecentViewItem[]): RecentViewItem[] {
  const seen = new Set<string>();
  const out: RecentViewItem[] = [];
  for (const row of rows) {
    if (seen.has(row.key)) continue;
    seen.add(row.key);
    out.push(row);
    if (out.length >= MAX_RECENT_VIEWS) break;
  }
  return out;
}

function sanitizeRecentSearches(value: unknown): string[] {
  if (!Array.isArray(value)) return [];
  return value.filter((item): item is string => typeof item === "string").slice(0, MAX_RECENT_SEARCHES);
}

function sanitizeRecentViews(value: unknown): RecentViewItem[] {
  if (!Array.isArray(value)) return [];
  return dedupeRecentViews(value.filter(isRecentViewItem));
}

function sanitizeFavorites(value: unknown): FavoriteItem[] {
  if (!Array.isArray(value)) return [];
  return value.filter(isFavoriteItem).slice(0, MAX_FAVORITES);
}

function sanitizeFontSize(value: unknown): ReaderFontSize {
  // Backward compatibility: previously persisted as "sm" | "md" | "lg".
  if (value === "sm") return 92;
  if (value === "md") return 100;
  if (value === "lg") return 112;

  if (typeof value !== "number" || !Number.isFinite(value)) return 100;
  const rounded = Math.round(value);
  return Math.min(130, Math.max(80, rounded));
}

function sanitizeLineHeight(value: unknown): ReaderLineHeight {
  return READER_LINE_HEIGHTS.includes(value as ReaderLineHeight)
    ? (value as ReaderLineHeight)
    : "normal";
}

function sanitizeWidth(value: unknown): ReaderWidth {
  return READER_WIDTHS.includes(value as ReaderWidth) ? (value as ReaderWidth) : "normal";
}

export function loadDictionaryPrefs(): DictionaryPrefsPayload {
  if (typeof window === "undefined") return DEFAULT_PREFS;
  try {
    const raw = window.localStorage.getItem(PREFS_KEY);
    if (!raw) return DEFAULT_PREFS;
    const parsed = JSON.parse(raw) as unknown;
    if (!isRecord(parsed)) return DEFAULT_PREFS;

    return {
      recentSearches: sanitizeRecentSearches(parsed.recentSearches),
      recentViews: sanitizeRecentViews(parsed.recentViews),
      favorites: sanitizeFavorites(parsed.favorites),
      preprocessEnabled:
        typeof parsed.preprocessEnabled === "boolean"
          ? parsed.preprocessEnabled
          : DEFAULT_PREFS.preprocessEnabled,
      markerPreprocessEnabled:
        typeof parsed.markerPreprocessEnabled === "boolean"
          ? parsed.markerPreprocessEnabled
          : DEFAULT_PREFS.markerPreprocessEnabled,
      readerFontSize: sanitizeFontSize(parsed.readerFontSize),
      readerLineHeight: sanitizeLineHeight(parsed.readerLineHeight),
      readerWidth: sanitizeWidth(parsed.readerWidth),
    };
  } catch {
    return DEFAULT_PREFS;
  }
}

export function saveDictionaryPrefs(payload: DictionaryPrefsPayload) {
  if (typeof window === "undefined") return;
  try {
    window.localStorage.setItem(PREFS_KEY, JSON.stringify(payload));
  } catch {
    // Ignore persistence failures (private mode, quota, etc.)
  }
}
