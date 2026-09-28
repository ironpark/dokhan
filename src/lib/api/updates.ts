import { getVersion } from '@tauri-apps/api/app';
import { check, type Update } from '@tauri-apps/plugin-updater';
import { isNewerVersion } from '$lib/utils/version';

const LATEST_RELEASE_API = 'https://api.github.com/repos/ironpark/dokhan/releases/latest';

export type AndroidRelease = { version: string; url: string };

/** Desktop: the updater plugin reads latest.json from the newest (non-prerelease) GitHub release. */
export function checkDesktopUpdate(): Promise<Update | null> {
  return check();
}

/**
 * Android has no updater plugin, so compare against the latest GitHub release and
 * link to its universal APK (or the release page when that asset is missing).
 */
export async function checkAndroidRelease(): Promise<AndroidRelease | null> {
  const response = await fetch(LATEST_RELEASE_API, {
    headers: { Accept: 'application/vnd.github+json' }
  });
  // No published release yet.
  if (response.status === 404) return null;
  if (!response.ok) throw new Error(`GitHub 릴리스 확인 실패 (${response.status})`);
  const release = (await response.json()) as {
    tag_name: string;
    html_url: string;
    assets?: Array<{ name: string; browser_download_url: string }>;
  };
  const current = await getVersion();
  if (!isNewerVersion(release.tag_name, current)) return null;
  const apk = release.assets?.find((asset) => /-universal-release\.apk$/.test(asset.name));
  return {
    version: release.tag_name.replace(/^v/, ''),
    url: apk?.browser_download_url ?? release.html_url
  };
}
