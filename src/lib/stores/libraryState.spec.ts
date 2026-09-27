import { describe, expect, it } from "vitest";
import { createLibraryState } from "./libraryState.svelte";

describe("library source scope", () => {
  it("keeps bookmarks and recent views tied to their dictionary", () => {
    const library = createLibraryState(() => {});
    library.setSourceScope("/managed/first.zip");
    library.toggleFavoriteEntry({ id: 1, headword: "Apfel", sourcePath: "merge01.chm" });
    library.pushRecentView({
      key: "entry:1",
      kind: "entry",
      label: "Apfel",
      id: 1,
      local: null,
      sourcePath: "merge01.chm",
      viewedAt: 1,
    });

    library.setSourceScope("/managed/second.zip");
    expect(library.isFavoriteEntry(1)).toBe(false);
    expect(library.favorites).toEqual([]);
    expect(library.recentViews).toEqual([]);

    library.toggleFavoriteEntry({ id: 1, headword: "Baum", sourcePath: "merge01.chm" });
    expect(library.favorites.map((item) => item.label)).toEqual(["Baum"]);

    library.setSourceScope("/managed/first.zip");
    expect(library.favorites.map((item) => item.label)).toEqual(["Apfel"]);
    expect(library.recentViews.map((item) => item.label)).toEqual(["Apfel"]);
  });

  it("preserves legacy bookmarks without attaching them to an unknown dictionary", () => {
    const library = createLibraryState(() => {});
    library.applySnapshot({
      recentSearches: [],
      recentViews: [],
      favorites: [{
        key: "entry:1",
        kind: "entry",
        label: "Old entry",
        id: 1,
        local: null,
        sourcePath: "merge01.chm",
        folderId: "default",
      }],
      bookmarkFolders: [{ id: "default", name: "기본", createdAt: 0 }],
      activeBookmarkFolderId: "default",
    });
    library.setSourceScope("/managed/new.zip");

    expect(library.legacyFavoriteCount).toBe(1);
    expect(library.favorites).toEqual([]);
    expect(library.toSnapshot().favorites).toHaveLength(1);
  });
});
