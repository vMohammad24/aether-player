import { events, type PlayerEvent } from '$lib/bindings';
import { createGlobalResource, createMutation, invalidate, updateCache } from '$lib/stores/resource.svelte';

class PlayerStore {
    #resource = createGlobalResource('getPlayerState');

    constructor() {

        events.playerEvent.listen((event) => {
            this.handleEvent(event.payload);
        });
    }

    get state() { return this.#resource.data; }
    get loading() { return this.#resource.loading; }

    handleEvent(event: PlayerEvent) {

        updateCache('getPlayerState', [], (current) => {
            if (!current) return current;


            const next = { ...current };

            if (event.type === 'TimeUpdate') {
                next.position = event.data;
            } else if (event.type === 'DurationChange') {
                next.duration = event.data;
            } else if (event.type === 'Paused') {
                next.paused = true;
            } else if (event.type === 'Playing') {
                next.paused = false;
                invalidate('getQueue');
            } else if (event.type === 'Ended') {
                next.paused = true;
                invalidate('getQueue');
            }

            return next;
        });
    }


    play = createMutation('play', { invalidate: 'getPlayerState' });
    pause = createMutation('pause', { invalidate: 'getPlayerState' });
    stop = createMutation('stop', { invalidate: 'getPlayerState' });
    next = createMutation('next', { invalidate: ['getPlayerState', 'getQueue'] });
    prev = createMutation('prev', { invalidate: ['getPlayerState', 'getQueue'] });

    playTrack = createMutation('playTrack', { invalidate: ['getPlayerState', 'getQueue'] });

    private _seek = createMutation('seek');
    async seek(seconds: number) {

        updateCache('getPlayerState', [], s => s ? ({ ...s, position: seconds }) : s);
        return this._seek.trigger(seconds);
    }

    private _setVolume = createMutation('setVolume');
    async setVolume(vol: number) {
        if (vol > 1) vol = vol / 100;
        updateCache('getPlayerState', [], s => s ? ({ ...s, volume: vol }) : s);
        return this._setVolume.trigger(vol);
    }

    toggleShuffle = createMutation('toggleShuffle', { invalidate: ['getPlayerState', 'getQueue'] });
    setRepeat = createMutation('setRepeat', { invalidate: ['getPlayerState', 'getQueue'] });
}

export const player = new PlayerStore();
