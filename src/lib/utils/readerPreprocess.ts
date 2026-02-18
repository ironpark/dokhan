import {
  ANGLE_MARKER_ALIASES,
  ANGLE_MARKER_META,
  ROUND_MARKER_META,
  ROUND_MARKER_GROUPS,
  SQUARE_MARKER_META,
  SQUARE_MARKER_GROUPS,
  type MarkerGroup,
} from "$lib/constants/dictionaryMarkers";

function extractSenseNo(node: Node): number | null {
  if (!(node instanceof HTMLSpanElement)) return null;
  const text = node.textContent?.trim() ?? "";
  const match = text.match(/^(\d+)\.$/);
  if (!match) return null;
  const no = Number.parseInt(match[1], 10);
  if (!Number.isFinite(no) || no <= 0) return null;
  return no;
}

function extractAlphaSenseNo(node: Node): number | null {
  if (!(node instanceof HTMLSpanElement)) return null;
  const text = node.textContent?.trim() ?? "";
  const match = text.match(/^([a-z])\)$/i);
  if (!match) return null;
  const code = match[1].toLowerCase().charCodeAt(0) - 96;
  if (!Number.isFinite(code) || code <= 0) return null;
  return code;
}

function splitCombinedSenseMarkers(root: HTMLElement) {
  if (root.dataset.combinedSenseSplit === "1") return;
  const nodes = Array.from(root.childNodes);
  for (const node of nodes) {
    if (!(node instanceof HTMLSpanElement)) continue;
    const text = (node.textContent ?? "").trim();
    const match = text.match(/^(\d+)\.\s*([a-z])\)$/i);
    if (!match) continue;
    const numSpan = document.createElement("span");
    numSpan.textContent = `${match[1]}.`;
    const alphaSpan = document.createElement("span");
    alphaSpan.textContent = `${match[2].toLowerCase()})`;
    node.replaceWith(numSpan, document.createTextNode(" "), alphaSpan);
  }
  root.dataset.combinedSenseSplit = "1";
}

function collectMarkers(
  nodes: Node[],
  extract: (node: Node) => number | null,
  minimum = 2,
): Array<{ idx: number; no: number }> {
  const markers = nodes
    .map((node, idx) => ({ idx, no: extract(node) }))
    .filter((row): row is { idx: number; no: number } => row.no !== null);
  if (markers.length < minimum) return [];
  for (let i = 1; i < markers.length; i += 1) {
    if (markers[i].no <= markers[i - 1].no) return [];
  }
  return markers;
}

function buildOrderedList(
  nodes: Node[],
  markers: Array<{ idx: number; no: number }>,
  options: {
    listClassName: string;
    itemClassName: string;
    type?: "a";
  },
): { preface: DocumentFragment; list: HTMLOListElement } {
  const preface = document.createDocumentFragment();
  const first = markers[0];
  for (let i = 0; i < first.idx; i += 1) {
    preface.appendChild(nodes[i]);
  }

  const list = document.createElement("ol");
  list.className = options.listClassName;
  if (options.type) {
    list.setAttribute("type", options.type);
  }
  if (first.no > 1) {
    list.setAttribute("start", String(first.no));
  }

  for (let i = 0; i < markers.length; i += 1) {
    const marker = markers[i];
    const end = i + 1 < markers.length ? markers[i + 1].idx : nodes.length;
    const li = document.createElement("li");
    li.className = options.itemClassName;
    for (let j = marker.idx + 1; j < end; j += 1) {
      li.appendChild(nodes[j]);
    }
    list.appendChild(li);
  }

  return { preface, list };
}

function applyAlphaSubSenseList(listItem: HTMLElement) {
  if (listItem.dataset.alphaSenseListApplied === "1") return;
  const nodes = Array.from(listItem.childNodes);
  const markers = collectMarkers(nodes, extractAlphaSenseNo);
  if (!markers.length) return;
  const { preface, list } = buildOrderedList(nodes, markers, {
    listClassName: "dict-subsense-list",
    itemClassName: "dict-subsense-item",
    type: "a",
  });
  listItem.replaceChildren(preface, list);
  listItem.dataset.alphaSenseListApplied = "1";
}

function applySenseList(root: HTMLElement) {
  if (root.dataset.senseListApplied === "1") return;
  const nodes = Array.from(root.childNodes);
  const markers = collectMarkers(nodes, extractSenseNo);
  if (!markers.length) return;

  const { preface, list } = buildOrderedList(nodes, markers, {
    listClassName: "dict-sense-list",
    itemClassName: "dict-sense-item",
  });

  root.replaceChildren(preface, list);

  const topLevelItems = Array.from(
    list.querySelectorAll(":scope > li.dict-sense-item"),
  ) as HTMLElement[];
  for (const item of topLevelItems) {
    applyAlphaSubSenseList(item);
  }

  root.dataset.senseListApplied = "1";
}

function applyBrSpacing(root: HTMLElement) {
  const breaks = Array.from(root.querySelectorAll("br"));
  for (const br of breaks) {
    if ((br as HTMLElement).dataset.spaced === "1") continue;
    const spacer = document.createElement("span");
    spacer.className = "dict-br-spacer";
    spacer.setAttribute("aria-hidden", "true");
    br.insertAdjacentElement("afterend", spacer);
    (br as HTMLElement).dataset.spaced = "1";
  }
}

function classifyRoundMarker(raw: string): MarkerGroup {
  return ROUND_MARKER_META[raw]?.group
    ?? ROUND_MARKER_GROUPS[raw]
    ?? SQUARE_MARKER_GROUPS[raw]
    ?? "misc";
}

function classifySquareMarker(raw: string): MarkerGroup {
  return SQUARE_MARKER_META[raw]?.group ?? SQUARE_MARKER_GROUPS[raw] ?? "misc";
}

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

function normalizeAngleMarker(raw: string): string {
  return ANGLE_MARKER_ALIASES[raw] ?? raw;
}

function buildRoundTooltip(label: string): string | null {
  const meta = ROUND_MARKER_META[label];
  if (meta) return meta.tooltip;
  return markerGroupHint(classifyRoundMarker(label));
}

function buildSquareTooltip(label: string): string | null {
  const meta = SQUARE_MARKER_META[label];
  if (meta) return meta.tooltip;
  return markerGroupHint(classifySquareMarker(label));
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

function applyTooltip(span: HTMLSpanElement, tooltip: string | null) {
  if (!tooltip) {
    delete span.dataset.tooltip;
    span.removeAttribute("aria-label");
    return;
  }
  span.dataset.tooltip = tooltip;
  span.setAttribute("aria-label", tooltip);
}

function isLikelyReplacementBracket(
  label: string,
  prevChar: string,
  nextChar: string,
): boolean {
  const trimmed = label.trim();
  const attachedToPrevWord = /[0-9A-Za-z가-힣)]/.test(prevChar);
  const attachedToNextWord = /[0-9A-Za-z가-힣(]/.test(nextChar);

  // 설명형/정의형 대괄호는 대체괄호로 보지 않는다.
  if (trimmed.includes(":")) return false;
  if (
    /(표현|뜻함|의미|용법|결합|관련|경우|드물게|또한|새 맞춤법|구맞춤법)/.test(
      trimmed,
    )
  ) {
    return false;
  }

  // 대체괄호는 대체로 앞/뒤 단어에 붙어서 등장한다.
  if (!(attachedToPrevWord || attachedToNextWord)) return false;

  // 치환 후보는 짧은 명사구/표현인 경우가 많다.
  if (trimmed.length > 40) return false;
  return true;
}

function resolveSquareMarkerMeta(
  label: string,
  prevChar: string,
  nextChar: string,
): { group: MarkerGroup; tooltip: string } {
  const exact = SQUARE_MARKER_META[label];
  if (exact) {
    return { group: exact.group, tooltip: exact.tooltip };
  }

  const normalizedLabel = label.replace(/\(\([^)]+\)\)\s*/g, "").trim();
  const orthographyMatch = normalizedLabel.match(/^(새 맞춤법|구맞춤법)\s*:/);
  if (orthographyMatch) {
    const kind = orthographyMatch[1];
    return {
      group: "orthography",
      tooltip:
        kind === "구맞춤법"
          ? "맞춤법 병기 정보: 구맞춤법 표기 (과거 문헌 참조용)"
          : "맞춤법 병기 정보: 새 맞춤법 표기",
    };
  }
  if (/(새 맞춤법|구맞춤법)/.test(normalizedLabel)) {
    return {
      group: "orthography",
      tooltip: "맞춤법 병기 정보: 새/구 맞춤법 병행 표기",
    };
  }

  if (isLikelyReplacementBracket(label, prevChar, nextChar)) {
    return {
      group: "meaning",
      tooltip: "대체괄호: 앞 어구를 괄호 안 표현으로 바꿔 쓸 수 있음",
    };
  }
  const fallbackGroup = classifySquareMarker(label);
  return {
    group: fallbackGroup,
    tooltip: markerGroupHint(fallbackGroup),
  };
}

function createMarkerSpan(
  kind: "round" | "square" | "angle",
  rawLabel: string,
  context?: {
    prevChar?: string;
    nextChar?: string;
  },
): HTMLSpanElement {
  const span = document.createElement("span");
  span.classList.add("dict-marker", `dict-marker-${kind}`);
  const trimmed = rawLabel.trim();

  if (kind === "round") {
    const group = classifyRoundMarker(trimmed);
    span.classList.add(`dict-marker-${group}`);
    span.dataset.markerGroup = group;
    span.dataset.markerKind = "round";
    span.dataset.markerLabel = trimmed;
    applyTooltip(span, buildRoundTooltip(trimmed));
    span.textContent = `((${trimmed}))`;
    return span;
  }

  if (kind === "square") {
    const { group, tooltip } = resolveSquareMarkerMeta(
      trimmed,
      context?.prevChar ?? "",
      context?.nextChar ?? "",
    );
    span.classList.add(`dict-marker-${group}`);
    span.dataset.markerGroup = group;
    span.dataset.markerKind = "square";
    span.dataset.markerLabel = trimmed;
    applyTooltip(span, tooltip);
    span.textContent = `[${trimmed}]`;
    return span;
  }

  const normalized = normalizeAngleMarker(trimmed);
  span.classList.add("dict-marker-grammar");
  span.dataset.markerGroup = "grammar";
  span.dataset.markerKind = "angle";
  span.dataset.markerLabel = normalized;
  applyTooltip(span, buildAngleTooltip(normalized));
  span.textContent = `<${normalized}>`;
  return span;
}

function annotateInlineMarkers(root: HTMLElement) {
  if (root.dataset.inlineMarkersApplied === "1") return;
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT);
  const textNodes: Text[] = [];
  let current = walker.nextNode();
  while (current) {
    const textNode = current as Text;
    const parent = textNode.parentElement;
    if (
      parent &&
      !["SCRIPT", "STYLE", "MARK"].includes(parent.tagName) &&
      !parent.classList.contains("dict-marker")
    ) {
      textNodes.push(textNode);
    }
    current = walker.nextNode();
  }

  const markerPattern = /\(\(([^()]+?)\)\)|\[([^[\]]+?)[\]}]|<([^<>]+?)>/g;
  for (const textNode of textNodes) {
    const text = textNode.nodeValue ?? "";
    markerPattern.lastIndex = 0;
    if (!markerPattern.test(text)) continue;
    markerPattern.lastIndex = 0;

    const fragment = document.createDocumentFragment();
    let lastIndex = 0;
    let match: RegExpExecArray | null;
    while ((match = markerPattern.exec(text)) !== null) {
      const matchIndex = match.index;
      if (matchIndex > lastIndex) {
        fragment.appendChild(
          document.createTextNode(text.slice(lastIndex, matchIndex)),
        );
      }

      const roundLabel = match[1]?.trim();
      const squareLabel = match[2]?.trim();
      const angleLabel = match[3]?.trim();
      const afterIndex = matchIndex + match[0].length;
      const prevChar = matchIndex > 0 ? text[matchIndex - 1] : "";
      const nextChar = afterIndex < text.length ? text[afterIndex] : "";
      if (roundLabel) {
        fragment.appendChild(createMarkerSpan("round", roundLabel));
      } else if (squareLabel) {
        fragment.appendChild(
          createMarkerSpan("square", squareLabel, { prevChar, nextChar }),
        );
      } else if (angleLabel) {
        fragment.appendChild(createMarkerSpan("angle", angleLabel));
      } else {
        fragment.appendChild(document.createTextNode(match[0]));
      }

      lastIndex = matchIndex + match[0].length;
      if (markerPattern.lastIndex === matchIndex) {
        markerPattern.lastIndex += 1;
      }
    }
    if (lastIndex < text.length) {
      fragment.appendChild(document.createTextNode(text.slice(lastIndex)));
    }

    textNode.parentNode?.replaceChild(fragment, textNode);
  }

  root.dataset.inlineMarkersApplied = "1";
}

type PreprocessOptions = {
  markerTagging?: boolean;
};

export function applyDictionaryPreprocess(
  root: HTMLElement,
  options: PreprocessOptions = {},
) {
  const { markerTagging = true } = options;
  splitCombinedSenseMarkers(root);
  applySenseList(root);
  applyBrSpacing(root);
  if (markerTagging) {
    annotateInlineMarkers(root);
  }
}
