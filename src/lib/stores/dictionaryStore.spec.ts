import { beforeEach, describe, expect, it, vi } from "vitest";
import type { EntryDetail, SearchHit } from "$lib/types/dictionary";

const api = vi.hoisted(() => ({
  startMasterBuild: vi.fn(),
  getMasterBuildStatus: vi.fn(),
  getMasterContents: vi.fn(),
  getIndexEntries: vi.fn(),
  searchEntries: vi.fn(),
  getEntryDetail: vi.fn(),
}));

vi.mock("$lib/api/dictionary", () => ({
  ...api,
  getContentPage: vi.fn(),
  prepareZipSource: vi.fn(),
  resolveLinkTarget: vi.fn(),
  resolveMediaDataUrl: vi.fn(),
}));
vi.mock("@tauri-apps/plugin-dialog", () => ({ open: vi.fn() }));

import { createDictionaryStore } from "./dictionaryStore.svelte";

function deferred<T>() {
  let resolve!: (value: T) => void;
  const promise = new Promise<T>((done) => { resolve = done; });
  return { promise, resolve };
}

describe("dictionary source changes", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    api.startMasterBuild.mockResolvedValue("build-key");
    api.getMasterBuildStatus.mockImplementation(async (path: string) => {
      if (path === "/bad.zip") {
        return { done: true, success: false, error: "Invalid ZIP" };
      }
      return {
        done: true,
        success: true,
        summary: { zipPath: "/managed/first.zip", contentCount: 1, indexCount: 1 },
      };
    });
    api.getMasterContents.mockResolvedValue([{ local: "intro.htm", title: "Introduction" }]);
    api.getIndexEntries.mockResolvedValue([{ id: 1, headword: "Apfel", headwordHighlights: [] }]);
  });

  it("keeps the previous dictionary active when a replacement ZIP fails", async () => {
    const store = createDictionaryStore();
    store.setAutoOpenFirstContent(false);
    await store.useZipPath("/first.zip");
    expect(store.zipPath).toBe("/managed/first.zip");

    await store.useZipPath("/bad.zip");
    expect(store.zipPath).toBe("/managed/first.zip");
    expect(store.contents[0].title).toBe("Introduction");
    expect(store.error).toContain("Invalid ZIP");
    store.dispose();
  });

  it("keeps the previous dictionary when a built ZIP cannot load its contents", async () => {
    api.getMasterBuildStatus.mockImplementation(async (path: string) => ({
      done: true,
      success: true,
      summary: { zipPath: `/managed${path}`, contentCount: 1, indexCount: 1 },
    }));
    const store = createDictionaryStore();
    store.setAutoOpenFirstContent(false);
    await store.useZipPath("/first.zip");
    api.getMasterContents.mockRejectedValueOnce(new Error("Contents unavailable"));

    await store.useZipPath("/replacement.zip");

    expect(store.zipPath).toBe("/managed/first.zip");
    expect(store.contents[0].title).toBe("Introduction");
    expect(store.error).toContain("Contents unavailable");
    expect(store.showProgress).toBe(false);
    store.dispose();
  });

  it("ignores an earlier ZIP load that finishes after a newer selection", async () => {
    const store = createDictionaryStore();
    store.setAutoOpenFirstContent(false);
    const slowContents = deferred<Array<{ local: string; title: string }>>();
    api.getMasterBuildStatus.mockImplementation(async (path: string) => ({
      done: true,
      success: true,
      summary: { zipPath: `/managed${path}`, contentCount: 1, indexCount: 1 },
    }));
    api.getMasterContents.mockImplementation((path: string) =>
      path.includes("slow")
        ? slowContents.promise
        : Promise.resolve([{ local: "fast.htm", title: "New dictionary" }]),
    );

    const earlier = store.useZipPath("/slow.zip");
    await vi.waitFor(() => expect(api.getMasterContents).toHaveBeenCalledWith("/managed/slow.zip"));
    await store.useZipPath("/fast.zip");
    slowContents.resolve([{ local: "slow.htm", title: "Old dictionary" }]);
    await earlier;

    expect(store.zipPath).toBe("/managed/fast.zip");
    expect(store.contents[0].title).toBe("New dictionary");
    store.dispose();
  });

  it("discards search and detail responses after the user leaves them", async () => {
    const store = createDictionaryStore();
    store.setAutoOpenFirstContent(false);
    await store.useZipPath("/first.zip");

    const search = deferred<SearchHit[]>();
    api.searchEntries.mockReturnValue(search.promise);
    store.setSearchQuery("Apfel");
    const pendingSearch = store.submitSearch();
    store.setSearchQuery("");
    search.resolve([{ id: 1, headword: "Apfel", sourcePath: "merge01.chm", snippet: "fruit", score: 1 }]);
    await pendingSearch;
    expect(store.searchRows).toEqual([]);

    const detail = deferred<EntryDetail>();
    api.getEntryDetail.mockReturnValue(detail.promise);
    const pendingDetail = store.openEntry(1);
    store.closeDetail();
    detail.resolve({
      id: 1,
      headword: "Apfel",
      aliases: [],
      sourcePath: "merge01.chm",
      definitionText: "fruit",
      definitionHtml: "",
    });
    await pendingDetail;
    expect(store.selectedEntry).toBeNull();
    expect(store.selectedEntryId).toBeNull();
    store.dispose();
  });
});
