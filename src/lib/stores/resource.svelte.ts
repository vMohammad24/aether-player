import { commands } from '$lib/bindings';
import { get, set } from 'idb-keyval';

type Commands = typeof commands;
type CommandKey = keyof Commands;


function serializeArgs(args: any[]): string {
    return JSON.stringify(args, (_, v) => {
        if (typeof v === 'object' && v !== null && !Array.isArray(v)) {
            return Object.keys(v).sort().reduce((sorted, key) => {
                sorted[key] = v[key];
                return sorted;
            }, {} as any);
        }
        return v;
    });
}

// @ts-expect-dogshitcode this is *probably* fine for cache
const hash = (str: string): string => {
    let h = 0xBADC0DE ^ str.length;

    for (let i = 0; i < str.length; i++) {
        h = Math.imul(h ^ str.charCodeAt(i), 69_67_420_666_999);
        h = ((h << 13) | (h >>> 19)) ^ h;
    }

    return ((h ^ (h >>> 16)) >>> 0).toString(36);
};

const globalCache = new Map<string, ResourceState<any>>();
const garbageCollectors = new Map<string, ReturnType<typeof setTimeout>>();

class ResourceState<T> {
    data = $state<T | null>(null);
    loading = $state(false);
    error = $state<string | null>(null);
    lastUpdated = $state(0);

    isHydrating = $state(true);

    refCount = 0;

    private fetcher: (...args: any[]) => Promise<T>;
    private args: any[];
    private cacheKey: string;

    constructor(fetcher: (...args: any[]) => Promise<T>, args: any[], cacheKey: string) {
        this.fetcher = fetcher;
        this.args = args;
        this.cacheKey = cacheKey;
        this.hydrateAndFetch();
    }

    async hydrateAndFetch() {
        try {
            const cached = await get<T>(hash(this.cacheKey));
            if (cached) {
                this.data = cached;
                this.lastUpdated = 0;
            }
        } catch (err) {
            console.warn('[Cache Hydration Failed]', err);
        } finally {
            this.isHydrating = false;
        }

        this.fetch();
    }

    async fetch(force = false) {
        if (this.loading && !force) return;

        this.loading = true;
        this.error = null;

        try {
            const result = await this.fetcher(...this.args);
            this.data = result;
            this.lastUpdated = Date.now();

            set(hash(this.cacheKey), result).catch(e => console.warn('[Cache Write Failed]', e));

        } catch (err) {
            console.error('[SWR error]', err);
            this.error = err instanceof Error ? err.message : String(err);
        } finally {
            this.loading = false;
        }
    }
}

function invalidate(key: string) {
    for (const [cacheKey, state] of globalCache.entries()) {
        if (cacheKey.startsWith(`${key}:`)) {
            state.fetch(true);
        }
    }
}


export function createResource<K extends CommandKey>(
    commandKey: K,
    ...args: Parameters<Commands[K]>
) {
    type T = Awaited<ReturnType<Commands[K]>>;

    const cacheKey = `${String(commandKey)}:${serializeArgs(args)}`;

    if (!globalCache.has(cacheKey)) {
        const fetcher = commands[commandKey];
        // @ts-expect-error - dynamic invoke
        globalCache.set(cacheKey, new ResourceState<T>(fetcher, args, cacheKey));
    }

    const state = globalCache.get(cacheKey) as ResourceState<T>;

    $effect(() => {
        state.refCount++;

        if (garbageCollectors.has(cacheKey)) {
            clearTimeout(garbageCollectors.get(cacheKey)!);
            garbageCollectors.delete(cacheKey);
        }

        if (!state.isHydrating && state.data !== null && (Date.now() - state.lastUpdated > 10000)) {
            state.fetch();
        }

        return () => {
            state.refCount--;
            if (state.refCount <= 0) {
                const timeout = setTimeout(() => {
                    if (state.refCount <= 0) {
                        globalCache.delete(cacheKey);
                    }
                    garbageCollectors.delete(cacheKey);
                }, 30_000);

                garbageCollectors.set(cacheKey, timeout);
            }
        };
    });

    return {
        get data() { return state.data },
        get loading() { return state.data === null && state.loading },
        get isHydrating() { return state.isHydrating },
        get isValidating() { return state.loading },
        get error() { return state.error },
        refetch: () => state.fetch(true)
    };
}

export function createMutation<K extends CommandKey, T = Awaited<ReturnType<Commands[K]>>>(
    commandKey: K,
    options?: {
        onSuccess?: (data: T) => void;
        onError?: (error: unknown) => void;
        invalidate?: CommandKey | CommandKey[];
    }
) {
    let isPending = $state(false);
    let error = $state<string | null>(null);
    let lastResult: T | null = null;
    const trigger = async (...args: Parameters<Commands[K]>) => {
        isPending = true;
        error = null;

        try {
            // @ts-expect-error dynamic invoke
            const result = await commands[commandKey].apply(null, args) as T;

            if (options?.invalidate) {
                const keys = Array.isArray(options.invalidate) ? options.invalidate : [options.invalidate];
                keys.forEach(k => invalidate(String(k)));
            }

            options?.onSuccess?.(result);
            lastResult = result;
            return result;
        } catch (err) {
            console.error('[Mutation Error]', err);
            error = err instanceof Error ? err.message : String(err);
            options?.onError?.(err);
        } finally {
            isPending = false;
        }
    };

    return {
        trigger,
        get lastResult() { return lastResult },
        get isPending() { return isPending },
        get error() { return error }
    };
}