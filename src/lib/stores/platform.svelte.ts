import { platform } from '@tauri-apps/plugin-os';

export class PlatformStore {
    isMobile = $state(false);
    platformName = $state<string | null>(null);

    constructor() {
        this.init();
    }

    async init() {
        try {
            const currentPlatform = await platform();
            this.platformName = currentPlatform;
            this.isMobile = currentPlatform === 'android' || currentPlatform === 'ios';
        } catch {
            // Fallback for browser dev or if plugin fails
            this.checkMedia();
            window.addEventListener('resize', () => this.checkMedia());
        }
    }

    checkMedia() {
        this.isMobile = window.matchMedia('(max-width: 980px)').matches;
    }
}

export const platformStore = new PlatformStore();
