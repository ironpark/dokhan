/** Parse `v1.2.3` / `1.2.3-beta.1` into numeric parts; null when it isn't a version. */
export function parseVersion(value: string): { core: [number, number, number]; prerelease: string } | null {
  const match = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?(?:\+[0-9A-Za-z.-]+)?$/.exec(value.trim());
  if (!match) return null;
  return {
    core: [Number(match[1]), Number(match[2]), Number(match[3])],
    prerelease: match[4] ?? ''
  };
}

/** True when `candidate` is a later release than `current`. A release outranks its own prereleases. */
export function isNewerVersion(candidate: string, current: string): boolean {
  const next = parseVersion(candidate);
  const now = parseVersion(current);
  if (!next || !now) return false;
  for (let i = 0; i < 3; i += 1) {
    if (next.core[i] !== now.core[i]) return next.core[i] > now.core[i];
  }
  if (next.prerelease === now.prerelease) return false;
  if (!next.prerelease) return true;
  if (!now.prerelease) return false;
  return next.prerelease.localeCompare(now.prerelease, 'en', { numeric: true }) > 0;
}
