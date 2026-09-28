import { describe, expect, it } from "vitest";
import { createReadingHistory } from "./readingHistory.svelte";

const entry = (id: number) => ({ kind: "entry" as const, id });

describe("reading history", () => {
  it("steps back and forward, and a new visit drops the forward entries", () => {
    const history = createReadingHistory();
    history.visit(entry(1));
    history.visit(entry(2));
    history.visit(entry(3));
    expect(history.canGoForward).toBe(false);

    expect(history.step(-1)?.location).toEqual(entry(2));
    expect(history.step(-1)?.location).toEqual(entry(1));
    expect(history.step(-1)).toBeNull();
    expect(history.canGoBack).toBe(false);
    expect(history.canGoForward).toBe(true);

    // Landing on the stepped-to location (the load finishing) is not a new visit.
    history.visit(entry(1));
    expect(history.canGoForward).toBe(true);

    history.visit(entry(4));
    expect(history.canGoForward).toBe(false);
    expect(history.step(-1)?.location).toEqual(entry(1));
  });

  it("keeps scroll offsets only for the item under the cursor", () => {
    const history = createReadingHistory();
    history.visit(entry(1));
    history.recordScroll("entry:1", 420);
    history.visit(entry(2));
    history.recordScroll("entry:1", 0);
    expect(history.step(-1)?.scrollTop).toBe(420);
  });

  it("caps its length", () => {
    const history = createReadingHistory(3);
    for (const id of [1, 2, 3, 4]) history.visit(entry(id));
    expect(history.step(-1)?.location).toEqual(entry(3));
    expect(history.step(-1)?.location).toEqual(entry(2));
    expect(history.step(-1)).toBeNull();
  });
});
