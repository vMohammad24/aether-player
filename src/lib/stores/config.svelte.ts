import { commands, type AppConfig } from '$lib/bindings';
import { untrack } from 'svelte';


class ConfigManager {
    #state = $state<AppConfig>({ theme: 'default', audioOutputDevice: null, sources: [], lastfmSession: null, discordRpc: { enabled: false } });
    #isReady = $state(false);
    #isSaving = $state(false);
    #lastError = $state<string | null>(null);
    #saveTimeout: ReturnType<typeof setTimeout> | null = null;

    constructor() {
        this.#init();

        $effect.root(() => {
            $effect(() => {

                JSON.stringify(this.#state);

                if (this.#isReady) {
                    untrack(() => this.save());
                }
            });
        });
    }

    async #init() {
        try {
            const result = await commands.getAppConfig();
            if (result.status === 'ok') {
                Object.assign(this.#state, result.data);
                console.log('[Config] Hydrated from Rust');
            } else {
                throw new Error(result.error);
            }
            this.#isReady = true;
        } catch (err) {
            console.error('[Config] Failed to load:', err);
            this.#lastError = String(err);
            this.#isReady = true;
        }
    }

    async save() {
        if (this.#saveTimeout) {
            clearTimeout(this.#saveTimeout);
        }

        this.#saveTimeout = setTimeout(async () => {
            this.#isSaving = true;
            try {
                const configToSave = $state.snapshot(this.#state);
                const result = await commands.saveAppConfig(configToSave);
                if (result.status === 'error') {
                    throw new Error(result.error);
                }
                console.log('[Config] Saved to Rust');
            } catch (err) {
                console.error('[Config] Failed to save:', err);
                this.#lastError = String(err);
            } finally {
                this.#isSaving = false;
            }
        }, 500);
    }

    async reset() {
        Object.assign(this.#state, await commands.getDefaultConfig());
    }

    async forceSync() {
        await this.#init();
    }

    get isReady() { return this.#isReady; }
    get isSaving() { return this.#isSaving; }
    get error() { return this.#lastError; }
    get _state() { return this.#state; }
}

const manager = new ConfigManager();

export const config = new Proxy(manager, {
    get(target, prop, receiver) {
        if (prop in target) {
            const value = Reflect.get(target, prop, receiver);
            if (typeof value === 'function') {
                return value.bind(target);
            }
            return value;
        }
        return target._state[prop as keyof AppConfig];
    },

    set(target, prop, value, receiver) {
        if (prop in target) {
            return false;
        }
        target._state[prop as keyof AppConfig] = value;
        return true;
    }
}) as ConfigManager & AppConfig;