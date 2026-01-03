import { createGlobalResource, createMutation } from '$lib/stores/resource.svelte';

class QueueStore {
    #resource = createGlobalResource('getQueue');

    get data() { return this.#resource.data; }
    get currentTrack() {
        return this.#resource.data?.tracks[this.#resource.data?.currentIndex ?? -1];
    }
    get loading() { return this.#resource.loading; }
    get error() { return this.#resource.error; }


    add = createMutation('addToQueue', { invalidate: 'getQueue' });
    addNext = createMutation('addNext', { invalidate: 'getQueue' });
    remove = createMutation('removeFromQueue', { invalidate: 'getQueue' });
    clear = createMutation('clearQueue', { invalidate: 'getQueue' });
    play = createMutation('playFromQueue', { invalidate: ['getQueue', 'getPlayerState'] });
    setRepeat = createMutation('setRepeat', { invalidate: 'getQueue' });
    toggleShuffle = createMutation('toggleShuffle', { invalidate: 'getQueue' });
}
export const queue = new QueueStore();