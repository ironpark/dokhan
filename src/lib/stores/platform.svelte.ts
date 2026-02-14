import { platform } from '@tauri-apps/plugin-os';

export class PlatformStore {
    isMobile = $state(false);
    platformName = $state<string | null>(null);
    #onResize: (() => void) | null = null;

    constructor() {
        this.checkMedia();
        this.init();
    }

    async init() {
        try {
            const currentPlatform = await platform();
            this.platformName = currentPlatform;
            this.isMobile = currentPlatform === 'android' || currentPlatform === 'ios';
        } catch {
            // Fallback for browser dev or if plugin fails
            this.#attachResizeListener();
        }
    }

    checkMedia() {
        if (typeof window === 'undefined') return;
        this.isMobile = window.matchMedia('(max-width: 980px)').matches;
    }

    #attachResizeListener() {
        if (typeof window === 'undefined') return;
        if (this.#onResize) {
            window.removeEventListener('resize', this.#onResize);
        }
        this.#onResize = () => this.checkMedia();
        window.addEventListener('resize', this.#onResize);
    }
}

export const platformStore = new PlatformStore();
