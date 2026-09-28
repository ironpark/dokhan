import { openUrl } from '@tauri-apps/plugin-opener';
import { relaunch } from '@tauri-apps/plugin-process';
import type { Update } from '@tauri-apps/plugin-updater';
import { checkAndroidRelease, checkDesktopUpdate } from '$lib/api/updates';

export type UpdatePhase =
  | { phase: 'idle' }
  | { phase: 'available'; version: string }
  | { phase: 'downloading'; version: string; progress: number | null }
  | { phase: 'installing'; version: string }
  | { phase: 'error'; version: string; message: string };

function toMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error);
}

export function createUpdateState() {
  let status = $state<UpdatePhase>({ phase: 'idle' });
  let desktopUpdate: Update | null = null;
  let androidUrl: string | null = null;
  let checked = false;

  /** Once per launch. Failures (offline, rate limits) stay silent: nothing to act on. */
  async function checkForUpdate(mode: 'desktop' | 'android') {
    if (checked) return;
    checked = true;
    try {
      if (mode === 'desktop') {
        desktopUpdate = await checkDesktopUpdate();
        if (desktopUpdate) status = { phase: 'available', version: desktopUpdate.version };
        return;
      }
      const release = await checkAndroidRelease();
      if (release) {
        androidUrl = release.url;
        status = { phase: 'available', version: release.version };
      }
    } catch (error) {
      console.warn('업데이트를 확인하지 못했습니다.', error);
    }
  }

  async function install() {
    if (status.phase !== 'available' && status.phase !== 'error') return;
    const version = status.version;
    if (androidUrl) {
      // The APK installs through the system installer; the banner has done its job.
      status = { phase: 'idle' };
      await openUrl(androidUrl).catch((error) => {
        status = { phase: 'error', version, message: toMessage(error) };
      });
      return;
    }
    if (!desktopUpdate) return;
    let total = 0;
    let received = 0;
    status = { phase: 'downloading', version, progress: null };
    try {
      await desktopUpdate.downloadAndInstall((event) => {
        if (event.event === 'Started') {
          total = event.data.contentLength ?? 0;
        } else if (event.event === 'Progress') {
          received += event.data.chunkLength;
          status = {
            phase: 'downloading',
            version,
            progress: total > 0 ? Math.min(100, Math.round((received / total) * 100)) : null
          };
        } else {
          status = { phase: 'installing', version };
        }
      });
      // Windows' installer exits the app itself; macOS swaps the bundle and needs a restart.
      status = { phase: 'installing', version };
      await relaunch();
    } catch (error) {
      status = { phase: 'error', version, message: toMessage(error) };
    }
  }

  function dismiss() {
    if (status.phase === 'downloading' || status.phase === 'installing') return;
    status = { phase: 'idle' };
  }

  return {
    get status() {
      return status;
    },
    get isDownloadOnly() {
      return androidUrl !== null;
    },
    checkForUpdate,
    install,
    dismiss
  };
}

export type UpdateState = ReturnType<typeof createUpdateState>;
