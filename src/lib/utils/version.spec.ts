import { describe, expect, it } from "vitest";
import { isNewerVersion, parseVersion } from "./version";

describe("isNewerVersion", () => {
  it("compares numerically, not as text", () => {
    expect(isNewerVersion("0.10.0", "0.9.9")).toBe(true);
    expect(isNewerVersion("v1.0.0", "0.99.0")).toBe(true);
    expect(isNewerVersion("0.2.0", "0.2.0")).toBe(false);
    expect(isNewerVersion("0.1.9", "0.2.0")).toBe(false);
  });

  it("ranks a release above its prereleases", () => {
    expect(isNewerVersion("1.0.0", "1.0.0-beta.2")).toBe(true);
    expect(isNewerVersion("1.0.0-beta.10", "1.0.0-beta.2")).toBe(true);
    expect(isNewerVersion("1.0.0-beta.1", "1.0.0")).toBe(false);
  });

  it("ignores tags that aren't versions", () => {
    expect(isNewerVersion("nightly", "0.1.0")).toBe(false);
    expect(parseVersion("v0.3.1")?.core).toEqual([0, 3, 1]);
  });
});
