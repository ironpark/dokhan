// @vitest-environment jsdom
import { describe, expect, it } from "vitest";
import { applyDictionaryPreprocess } from "./readerPreprocess";

describe("applyDictionaryPreprocess", () => {
  it("applies marker tags and keeps idempotent preprocess version", () => {
    const root = document.createElement("div");
    root.textContent = "((고어·관)) [구맞춤법: Paß] <준접미사>";

    applyDictionaryPreprocess(root, { markerTagging: true });

    const markers = root.querySelectorAll(".dict-marker");
    expect(markers.length).toBe(3);
    expect(root.dataset.preprocessVersion).toBeTruthy();

    const beforeHtml = root.innerHTML;
    applyDictionaryPreprocess(root, { markerTagging: true });
    expect(root.innerHTML).toBe(beforeHtml);
  });

  it("classifies descriptive square content as meaning info in DOM", () => {
    const root = document.createElement("div");
    root.textContent =
      "-gläu|big <준접미사>: [명사와 결합하여 그 인물이나 사실이 신앙심이 깊은 것을 표현하거나 뜻함]";

    applyDictionaryPreprocess(root, { markerTagging: true });

    const square = root.querySelector(".dict-marker-square") as HTMLSpanElement | null;
    expect(square).not.toBeNull();
    expect(square?.dataset.markerGroup).toBe("meaning");
    expect(square?.dataset.tooltip).toContain("설명");
  });

  it("detects replacement bracket alternatives", () => {
    const root = document.createElement("div");
    root.textContent =
      "an Stelle seines Namens[seiner Unterschrift] hat er ein K. gemacht";

    applyDictionaryPreprocess(root, { markerTagging: true });

    const square = root.querySelector(".dict-marker-square") as HTMLSpanElement | null;
    expect(square?.dataset.tooltip).toContain("대체괄호");
  });

  it("removes marker spans when marker tagging is turned off on same root", () => {
    const root = document.createElement("div");
    root.textContent = "((고어)) [관] <Adj.>";

    applyDictionaryPreprocess(root, { markerTagging: true });
    expect(root.querySelectorAll(".dict-marker").length).toBe(3);

    applyDictionaryPreprocess(root, { markerTagging: false });
    expect(root.querySelectorAll(".dict-marker").length).toBe(0);
    expect(root.textContent).toContain("((고어))");
    expect(root.textContent).toContain("[관]");
    expect(root.textContent).toContain("<Adj.>");
  });
});
