import {
  ANGLE_MARKER_ALIASES,
  ANGLE_MARKER_META,
  ROUND_MARKER_GROUPS,
  ROUND_MARKER_META,
  SQUARE_MARKER_GROUPS,
  SQUARE_MARKER_META,
  type MarkerGroup,
} from "$lib/constants/dictionaryMarkers";

/**
 * Marker parser package.
 * - Step 1: tokenize raw text into plain text + marker tokens.
 * - Step 2: classify each marker token into semantic groups/tooltips.
 *
 * This module is intentionally DOM-free so it can be tested independently
 * and reused by multiple rendering paths.
 */
export type MarkerKind = "round" | "square" | "angle";

export type MarkerToken = {
  kind: MarkerKind;
  raw: string;
  start: number;
  end: number;
  prevChar: string;
  nextChar: string;
};

export type TextSegment =
  | { type: "text"; value: string }
  | { type: "marker"; token: MarkerToken };

export type MarkerClassification = {
  groups: MarkerGroup[];
  primaryGroup: MarkerGroup;
  tooltip: string;
  normalizedLabel: string;
};

function markerGroupHint(group: MarkerGroup): string {
  if (group === "register" || group === "usage") return "문체/용법 정보";
  if (group === "region") return "사용 지역 정보";
  if (group === "time") return "시대 구분 정보";
  if (group === "domain") return "전문 분야 정보";
  if (group === "meaning") return "의미/용법 정보";
  if (group === "orthography") return "맞춤법 병기 정보";
  if (group === "grammar") return "문법 정보";
  return "표기 정보";
}

function uniqueGroups(groups: MarkerGroup[]): MarkerGroup[] {
  return Array.from(new Set(groups));
}

function pickPrimaryGroup(groups: MarkerGroup[]): MarkerGroup {
  return groups[0] ?? "misc";
}

function normalizeAngleMarker(raw: string): string {
  return ANGLE_MARKER_ALIASES[raw] ?? raw;
}

function isPartOfSpeechMarker(label: string): boolean {
  return /^(adj|adv|verb|subst|pron|praep|prep|konj|interj|part|num)\.?$/i.test(label);
}

function isNumberInfoMarker(label: string): boolean {
  return /(복수|단수|pl\.|sg\.)/i.test(label);
}

function isVerbInflectionMarker(label: string): boolean {
  return /(;|,|\bhat\b|\bist\b|\bsind\b|\bhaben\b|\bdu\b|\ber\b|\bsie\b|\bwir\b|\bihr\b)/i.test(
    label,
  );
}

function buildAngleTooltip(label: string): string {
  const direct = ANGLE_MARKER_META[label];
  if (direct) return direct;
  if (isPartOfSpeechMarker(label)) {
    return `품사 표시: ${label}`;
  }
  if (isNumberInfoMarker(label)) {
    return `문법 수(數) 정보: ${label}`;
  }
  if (isVerbInflectionMarker(label)) {
    return "동사 활용/완료조동사 정보";
  }
  return markerGroupHint("grammar");
}

function classifyRoundSingle(label: string): { group: MarkerGroup; tooltip: string } {
  const direct = ROUND_MARKER_META[label];
  if (direct) return { group: direct.group, tooltip: direct.tooltip };

  const adverbMatch = label.match(/^(때때로)\s+(.+)$/);
  if (adverbMatch) {
    const [, adverb, core] = adverbMatch;
    const coreDirect = ROUND_MARKER_META[core];
    if (coreDirect) {
      return {
        group: coreDirect.group,
        tooltip: `${adverb} ${coreDirect.tooltip}`,
      };
    }
    const coreGroup = ROUND_MARKER_GROUPS[core] ?? SQUARE_MARKER_GROUPS[core];
    if (coreGroup) {
      return {
        group: coreGroup,
        tooltip: `${adverb} ${markerGroupHint(coreGroup)}`,
      };
    }
  }

  const group =
    ROUND_MARKER_META[label]?.group
    ?? ROUND_MARKER_GROUPS[label]
    ?? SQUARE_MARKER_GROUPS[label]
    ?? "misc";
  return { group, tooltip: markerGroupHint(group) };
}

function splitRoundComposite(label: string): string[] {
  if (!label.includes("·")) return [label.trim()];
  const parts = label
    .split("·")
    .map((part) => part.trim())
    .filter(Boolean);
  return parts.length ? parts : [label.trim()];
}

function classifyRound(label: string): MarkerClassification {
  const parts = splitRoundComposite(label);
  const infos = parts.map((part) => classifyRoundSingle(part));
  const groups = uniqueGroups(infos.map((info) => info.group));
  const tooltip = infos
    .map((info) => info.tooltip)
    .filter(Boolean)
    .filter((value, index, arr) => arr.indexOf(value) === index)
    .join(" / ");

  return {
    groups,
    primaryGroup: pickPrimaryGroup(groups),
    tooltip: tooltip || markerGroupHint("misc"),
    normalizedLabel: label,
  };
}

function isLikelyReplacementBracket(label: string, prevChar: string, nextChar: string): boolean {
  const trimmed = label.trim();
  if (!trimmed) return false;

  const attachedToPrevWord = /[0-9A-Za-z가-힣)]/.test(prevChar);
  const attachedToNextWord = /[0-9A-Za-z가-힣(]/.test(nextChar);

  if (!(attachedToPrevWord || attachedToNextWord)) return false;
  if (trimmed.length > 40) return false;
  if (trimmed.includes(":")) return false;

  if (/(표현|뜻함|의미|용법|결합|관련|경우|드물게|또한|새 맞춤법|구맞춤법|준접미사)/.test(trimmed)) {
    return false;
  }

  // 전문영역 축약 코드는 대체괄호가 아니다.
  if (/^[가-힣]{1,4}$/.test(trimmed) && SQUARE_MARKER_GROUPS[trimmed]) {
    return false;
  }

  return true;
}

function normalizeSquareLabel(raw: string): string {
  return raw.replace(/\(\([^)]+\)\)\s*/g, "").trim();
}

function isDescriptiveSquareContent(label: string): boolean {
  return /(표현|뜻함|의미|용법|결합|관련|설명|규정|표기|지시|준접미사|준접두사|형태|문체)/.test(label);
}

function classifySquare(
  rawLabel: string,
  prevChar: string,
  nextChar: string,
): MarkerClassification {
  const label = rawLabel.trim();

  const exact = SQUARE_MARKER_META[label];
  if (exact) {
    return {
      groups: [exact.group],
      primaryGroup: exact.group,
      tooltip: exact.tooltip,
      normalizedLabel: label,
    };
  }

  const normalized = normalizeSquareLabel(label);
  const normalizedExact = SQUARE_MARKER_META[normalized];
  if (normalizedExact) {
    return {
      groups: [normalizedExact.group],
      primaryGroup: normalizedExact.group,
      tooltip: normalizedExact.tooltip,
      normalizedLabel: normalized,
    };
  }

  if (/^(새 맞춤법|구맞춤법)\s*:/.test(normalized)) {
    const kind = RegExp.$1;
    return {
      groups: ["orthography"],
      primaryGroup: "orthography",
      tooltip:
        kind === "구맞춤법"
          ? "맞춤법 병기 정보: 구맞춤법 표기 (과거 문헌 참조용)"
          : "맞춤법 병기 정보: 새 맞춤법 표기",
      normalizedLabel: normalized,
    };
  }

  if (/(새 맞춤법|구맞춤법)/.test(normalized)) {
    return {
      groups: ["orthography"],
      primaryGroup: "orthography",
      tooltip: "맞춤법 병기 정보: 새/구 맞춤법 병행 표기",
      normalizedLabel: normalized,
    };
  }

  if (isLikelyReplacementBracket(normalized, prevChar, nextChar)) {
    return {
      groups: ["meaning"],
      primaryGroup: "meaning",
      tooltip: "대체괄호: 앞 어구를 괄호 안 표현으로 바꿔 쓸 수 있음",
      normalizedLabel: normalized,
    };
  }

  if (isDescriptiveSquareContent(normalized) || normalized.includes(":")) {
    return {
      groups: ["meaning"],
      primaryGroup: "meaning",
      tooltip: "의미/용법 설명 정보",
      normalizedLabel: normalized,
    };
  }

  const fallback = SQUARE_MARKER_GROUPS[normalized] ?? "misc";
  return {
    groups: [fallback],
    primaryGroup: fallback,
    tooltip: markerGroupHint(fallback),
    normalizedLabel: normalized,
  };
}

/**
 * Classifies a single marker token into semantic metadata used by UI.
 * The result drives marker color, tooltip text, and grouped marker hints.
 */
export function classifyMarkerToken(token: MarkerToken): MarkerClassification {
  if (token.kind === "round") {
    return classifyRound(token.raw.trim());
  }
  if (token.kind === "square") {
    return classifySquare(token.raw.trim(), token.prevChar, token.nextChar);
  }

  const normalized = normalizeAngleMarker(token.raw.trim());
  return {
    groups: ["grammar"],
    primaryGroup: "grammar",
    tooltip: buildAngleTooltip(normalized),
    normalizedLabel: normalized,
  };
}

function findSquareClose(text: string, startAt: number): number {
  const closeBracket = text.indexOf("]", startAt);
  const closeBrace = text.indexOf("}", startAt);
  if (closeBracket === -1) return closeBrace;
  if (closeBrace === -1) return closeBracket;
  return Math.min(closeBracket, closeBrace);
}

/**
 * Tokenizes text with tolerant delimiter handling.
 * Supports malformed square closers (`}`) found in source data.
 * Unmatched delimiters are emitted as plain text instead of throwing.
 */
export function tokenizeMarkerText(text: string): TextSegment[] {
  if (!text || (!text.includes("((") && !text.includes("[") && !text.includes("<"))) {
    return [{ type: "text", value: text }];
  }

  const segments: TextSegment[] = [];
  let cursor = 0;

  function pushText(end: number) {
    if (end > cursor) {
      segments.push({ type: "text", value: text.slice(cursor, end) });
    }
  }

  while (cursor < text.length) {
    let next = -1;
    let kind: MarkerKind | null = null;

    const roundIdx = text.indexOf("((", cursor);
    const squareIdx = text.indexOf("[", cursor);
    const angleIdx = text.indexOf("<", cursor);

    for (const [idx, candidateKind] of [
      [roundIdx, "round"],
      [squareIdx, "square"],
      [angleIdx, "angle"],
    ] as const) {
      if (idx >= 0 && (next < 0 || idx < next)) {
        next = idx;
        kind = candidateKind;
      }
    }

    if (next < 0 || !kind) {
      pushText(text.length);
      cursor = text.length;
      break;
    }

    pushText(next);

    let close = -1;
    let raw = "";
    let end = -1;

    if (kind === "round") {
      close = text.indexOf("))", next + 2);
      if (close >= 0) {
        raw = text.slice(next + 2, close);
        end = close + 2;
      }
    } else if (kind === "square") {
      close = findSquareClose(text, next + 1);
      if (close >= 0) {
        raw = text.slice(next + 1, close);
        end = close + 1;
      }
    } else {
      close = text.indexOf(">", next + 1);
      if (close >= 0) {
        raw = text.slice(next + 1, close);
        end = close + 1;
      }
    }

    if (close < 0 || !raw.trim()) {
      // Treat unmatched delimiters as plain text and continue.
      segments.push({ type: "text", value: text[next] });
      cursor = next + 1;
      continue;
    }

    const prevChar = next > 0 ? text[next - 1] : "";
    const nextChar = end < text.length ? text[end] : "";
    segments.push({
      type: "marker",
      token: {
        kind,
        raw,
        start: next,
        end,
        prevChar,
        nextChar,
      },
    });
    cursor = end;
  }

  return segments.length ? segments : [{ type: "text", value: text }];
}
