import { commands, events, type PlayerEvent, type PlayerState, type Track } from '$lib/bindings';
import { createMutation, invalidate } from '$lib/stores/resource.svelte';
import { queue } from './queue.svelte';

class PlayerStore {
    state = $state<PlayerState | null>(null);
    loading = $state(true);
    oldVolume = 0;
    muted = $state(false);
    constructor() {
        this.init();
        events.playerEvent.listen((event) => {
            this.handleEvent(event.payload);
        });
    }

    async init() {
        try {
            const res = await commands.getPlayerState();
            if (res.status === 'ok') {
                this.state = res.data;
            }
        } catch (e) {
            console.error('Failed to initialize player state', e);
        } finally {
            this.loading = false;
        }
    }

    handleEvent(event: PlayerEvent) {
        if (!this.state) return;
        if (event.type === 'TimeUpdate') {
            this.state.position = event.data;
        } else if (event.type === 'DurationChange') {
            this.state.duration = event.data;
        } else if (event.type === 'Paused') {
            this.state.paused = true;
        } else if (event.type === 'Playing') {
            this.state.paused = false;
            invalidate('getQueue');
        } else if (event.type === 'Ended') {
            this.state.paused = true;
            invalidate('getQueue');
        }
    }

    play = createMutation('play', {
        invalidate: 'getQueue',
        onSuccess: () => { if (this.state) this.state.paused = false; }
    });

    pause = createMutation('pause', {
        onSuccess: () => { if (this.state) this.state.paused = true; }
    });

    stop = createMutation('stop', {
        onSuccess: () => {
            if (this.state) {
                this.state.paused = true;
                this.state.position = 0;
            }
        }
    });

    next = createMutation('next', { invalidate: 'getQueue' });
    prev = createMutation('prev', { invalidate: 'getQueue' });

    playTrack = createMutation('playTrack', { invalidate: 'getQueue' });

    private _seek = createMutation('seek');
    async seek(seconds: number) {
        if (this.state) this.state.position = seconds;
        return this._seek.trigger(seconds);
    }

    private _setVolume = createMutation('setVolume');
    async setVolume(vol: number) {
        if (vol > 1) vol = vol / 100;
        if (this.state) this.state.volume = vol;
        if (vol > 0) this.muted = false;
        return this._setVolume.trigger(vol);
    }

    async mute() {
        this.oldVolume = this.state ? this.state.volume : 1;
        this.muted = true;
        return this.setVolume(0);
    }

    async unmute() {
        this.muted = false;
        return this.setVolume(this.oldVolume);
    }

    async playTracks(trackIds: string[] | Track[], index: number = 0) {
        const ids = trackIds.map(t => typeof t === 'string' ? t : t.id);
        player.stop.trigger();
        player.seek(0);
        await queue.clear.trigger();
        await queue.addMultiple.trigger(ids);
        return queue.play.trigger(index);
    }

    toggleShuffle = createMutation('toggleShuffle', { invalidate: 'getQueue' });
    setRepeat = createMutation('setRepeat', { invalidate: 'getQueue' });
}

export const player = new PlayerStore();