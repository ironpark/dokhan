import {
  classifyMarkerToken,
  tokenizeMarkerText,
  type MarkerKind,
  type MarkerToken,
} from "$lib/utils/readerMarkerParser";

/**
 * Dictionary HTML preprocessor package.
 * - Rewrites raw dictionary markup into readable structural lists.
 * - Applies optional marker tagging with semantic classes/tooltips.
 * - Runs idempotently via dataset flags/versioning.
 */
const PREPROCESS_VERSION = "2026-02-19.1";

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

function applyTopLevelAlphaSenseList(root: HTMLElement) {
  if (root.dataset.alphaSenseListApplied === "1") return;
  const nodes = Array.from(root.childNodes);
  const markers = collectMarkers(nodes, extractAlphaSenseNo);
  if (!markers.length) return;

  const { preface, list } = buildOrderedList(nodes, markers, {
    listClassName: "dict-sense-list",
    itemClassName: "dict-sense-item",
    type: "a",
  });

  root.replaceChildren(preface, list);
  root.dataset.alphaSenseListApplied = "1";
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

function applyTooltip(span: HTMLSpanElement, tooltip: string | null) {
  if (!tooltip) {
    delete span.dataset.tooltip;
    span.removeAttribute("aria-label");
    return;
  }
  span.dataset.tooltip = tooltip;
  span.setAttribute("aria-label", tooltip);
}

function markerText(kind: MarkerKind, label: string): string {
  if (kind === "round") return `((${label}))`;
  if (kind === "square") return `[${label}]`;
  return `<${label}>`;
}

function createMarkerSpan(token: MarkerToken): HTMLSpanElement {
  const span = document.createElement("span");
  const info = classifyMarkerToken(token);
  span.classList.add("dict-marker", `dict-marker-${token.kind}`);
  span.classList.add(`dict-marker-${info.primaryGroup}`);
  if (info.groups.length > 1) {
    span.dataset.markerGroups = info.groups.join(",");
  }
  span.dataset.markerGroup = info.primaryGroup;
  span.dataset.markerKind = token.kind;
  span.dataset.markerLabel = info.normalizedLabel;
  applyTooltip(span, info.tooltip);
  span.textContent = markerText(token.kind, info.normalizedLabel);
  return span;
}

function annotateInlineMarkers(root: HTMLElement) {
  if (root.dataset.inlineMarkersApplied === "1") return;
  const rootText = root.textContent ?? "";
  if (
    !rootText.includes("((") &&
    !rootText.includes("[") &&
    !rootText.includes("<")
  ) {
    root.dataset.inlineMarkersApplied = "1";
    return;
  }
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

  for (const textNode of textNodes) {
    const text = textNode.nodeValue ?? "";
    if (!text.includes("((") && !text.includes("[") && !text.includes("<")) {
      continue;
    }
    const segments = tokenizeMarkerText(text);
    const hasMarker = segments.some((segment) => segment.type === "marker");
    if (!hasMarker) continue;

    const fragment = document.createDocumentFragment();
    for (const segment of segments) {
      if (segment.type === "text") {
        if (segment.value) {
          fragment.appendChild(document.createTextNode(segment.value));
        }
      } else {
        fragment.appendChild(createMarkerSpan(segment.token));
      }
    }
    textNode.parentNode?.replaceChild(fragment, textNode);
  }

  root.dataset.inlineMarkersApplied = "1";
}

function stripInlineMarkers(root: HTMLElement) {
  const markers = Array.from(root.querySelectorAll("span.dict-marker"));
  for (const marker of markers) {
    marker.replaceWith(document.createTextNode(marker.textContent ?? ""));
  }
  root.normalize();
  delete root.dataset.inlineMarkersApplied;
}

type PreprocessOptions = {
  markerTagging?: boolean;
};

/**
 * Applies structural + inline marker preprocessing to a rendered dictionary node.
 *
 * Safe to call repeatedly for the same root. Work is skipped when the target
 * preprocess version/options match the last applied state.
 */
export function applyDictionaryPreprocess(
  root: HTMLElement,
  options: PreprocessOptions = {},
) {
  const { markerTagging = true } = options;
  const targetVersion = `${PREPROCESS_VERSION}:${markerTagging ? "1" : "0"}`;
  if (root.dataset.preprocessVersion === targetVersion) return;
  if (
    root.dataset.preprocessVersion &&
    root.dataset.preprocessVersion !== targetVersion &&
    !markerTagging &&
    root.querySelector("span.dict-marker")
  ) {
    stripInlineMarkers(root);
  }
  splitCombinedSenseMarkers(root);
  applySenseList(root);
  if (root.dataset.senseListApplied !== "1") {
    applyTopLevelAlphaSenseList(root);
  }
  applyBrSpacing(root);
  if (markerTagging) {
    annotateInlineMarkers(root);
  }
  root.dataset.preprocessVersion = targetVersion;
}
