import { commands } from '$lib/bindings';

const globalCache = new Map<string, ResourceState<any>>();

type Commands = typeof commands;

class ResourceState<T> {
    data = $state<T | null>(null);
    loading = $state(false);
    error = $state<string | null>(null);
    lastUpdated = 0;

    constructor(initialValue: T | null = null) {
        this.data = initialValue;
    }
}

export function createResource<K extends keyof Commands>(
    commandKey: K,
    ...args: Parameters<Commands[K]>
) {
    type T = Awaited<ReturnType<Commands[K]>>;

    const cacheKey = `${String(commandKey)}:${JSON.stringify(args)}`;

    if (!globalCache.has(cacheKey)) {
        globalCache.set(cacheKey, new ResourceState<T>());
    }

    const state = globalCache.get(cacheKey) as ResourceState<T>;

    async function fetch(force = false) {
        if (state.loading && !force) return;

        state.loading = true;
        state.error = null;

        try {
            const result = await commands[commandKey].apply(null, args);
            state.data = result as T;

            state.lastUpdated = Date.now();
        } catch (err) {
            console.error(err);
            state.error = String(err);
        } finally {
            state.loading = false;
        }
    }

    if (state.data !== null) {
        const isStale = (Date.now() - state.lastUpdated) > 5000;
        if (isStale) {
            fetch();
        }
    } else {
        fetch();
    }

    return {
        get data() { return state.data },
        get loading() { return state.data === null && state.loading },
        get isValidating() { return state.loading },
        get error() { return state.error },
        refetch: () => fetch(true),
        mutate: (newData: T) => { state.data = newData; }
    };
}