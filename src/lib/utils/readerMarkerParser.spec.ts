import { describe, expect, it } from "vitest";
import { classifyMarkerToken, tokenizeMarkerText, type MarkerToken } from "./readerMarkerParser";

function token(kind: MarkerToken["kind"], raw: string, prevChar = " ", nextChar = " "): MarkerToken {
  return { kind, raw, start: 0, end: raw.length, prevChar, nextChar };
}

describe("readerMarkerParser", () => {
  it("tokenizes mixed marker text including malformed square-close braces", () => {
    const segments = tokenizeMarkerText(
      "-gläu|big <준접미사>: [명사와 결합하여 ... 뜻함}: A[B]",
    );
    const markers = segments.filter((segment) => segment.type === "marker");
    expect(markers).toHaveLength(3);
    expect(markers[0].type === "marker" && markers[0].token.kind).toBe("angle");
    expect(markers[1].type === "marker" && markers[1].token.raw).toContain("명사와 결합");
    expect(markers[2].type === "marker" && markers[2].token.raw).toBe("B");
  });

  it("classifies composite round markers with multiple groups", () => {
    const info = classifyMarkerToken(token("round", "고어·관"));
    expect(info.groups).toContain("time");
    expect(info.groups).toContain("domain");
    expect(info.primaryGroup).toBe("time");
  });

  it("classifies orthography square markers with nested round note", () => {
    const info = classifyMarkerToken(token("square", "((또한)) 새 맞춤법: Serigrafie"));
    expect(info.primaryGroup).toBe("orthography");
    expect(info.tooltip).toContain("맞춤법");
  });

  it("classifies replacement brackets in inline alternatives", () => {
    const info = classifyMarkerToken(token("square", "seiner Unterschrift", "s", " "));
    expect(info.primaryGroup).toBe("meaning");
    expect(info.tooltip).toContain("대체괄호");
  });

  it("classifies descriptive square text as meaning/usage info", () => {
    const info = classifyMarkerToken(
      token("square", "명사와 결합하여 그 인물이나 사실이 신앙심이 깊은 것을 표현"),
    );
    expect(info.primaryGroup).toBe("meaning");
    expect(info.tooltip).toContain("설명");
  });

  it("classifies round markers with adverb prefix like '때때로 폄'", () => {
    const info = classifyMarkerToken(token("round", "때때로 폄"));
    expect(info.primaryGroup).toBe("usage");
    expect(info.tooltip).toContain("때때로");
    expect(info.tooltip).toContain("폄");
  });
});
