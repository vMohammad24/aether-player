<script lang="ts">
  import Image from "$lib/components/Image.svelte";
  import { media } from "$lib/stores/player/media.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { queue } from "$lib/stores/player/queue.svelte";
  import { formatDuration } from "$lib/util";
  import {
    Pause,
    Play,
    Repeat,
    Repeat1,
    Shuffle,
    SkipBack,
    SkipForward,
    Volume1,
    Volume2,
    VolumeX,
  } from "@lucide/svelte";

  let currentTrack = $derived(queue.currentTrack);
  let isPlaying = $derived(!player.state?.paused);
  let duration = $derived(player.state?.duration ?? 0);
  let position = $derived(player.state?.position ?? 0);
  let volume = $derived((player.state?.volume ?? 1) * 100);
  let shuffle = $derived(queue.data?.shuffle ?? false);
  let repeat = $derived(queue.data?.repeat ?? "off");
  let coverArt = $derived(
    currentTrack ? media.album(currentTrack.albumId)?.data?.coverArt : null
  );

  function handleSeek(e: Event) {
    const target = e.target as HTMLInputElement;
    player.seek(Number(target.value));
  }

  function handleVolume(e: Event) {
    const target = e.target as HTMLInputElement;
    player.setVolume(Number(target.value));
  }

  function toggleRepeat() {
    const modes = ["off", "all", "one"] as const;
    const nextIndex = (modes.indexOf(repeat) + 1) % modes.length;
    player.setRepeat.trigger(modes[nextIndex]);
  }
</script>

<div
  class="h-24 bg-background/80 backdrop-blur-xl border-t border-white/5 grid grid-cols-3 items-center px-6 z-50"
>
  <div class="flex items-center gap-4 justify-start min-w-0">
    {#if currentTrack}
      <div
        class="h-14 w-14 bg-white/5 rounded-lg overflow-hidden shrink-0 border border-white/5"
      >
        <Image
          src={coverArt}
          type="cover"
          alt={currentTrack.albumTitle}
          class="h-full w-full object-cover"
        />
      </div>
      <div class="flex flex-col min-w-0 justify-center gap-0.5">
        <span class="text-sm font-medium text-text truncate"
          >{currentTrack.title}</span
        >
        <span
          class="text-xs text-subtext truncate hover:text-white transition-colors cursor-pointer"
          >{currentTrack.artistName}</span
        >
      </div>
    {/if}
  </div>

  <div class="flex flex-col items-center justify-center gap-2">
    <div class="flex items-center gap-6">
      <button
        class="hover:text-white transition-colors disabled:opacity-50 {shuffle
          ? 'text-cyan'
          : 'text-subtext'}"
        onclick={() => player.toggleShuffle.trigger()}
        aria-label="Shuffle"
      >
        <Shuffle size={18} />
      </button>

      <button
        class="text-subtext hover:text-white transition-colors hover:scale-105"
        onclick={() => player.prev.trigger()}
        aria-label="Previous"
      >
        <SkipBack size={22} fill="currentColor" />
      </button>

      <button
        class="h-10 w-10 bg-white rounded-full flex items-center justify-center text-black hover:scale-105 transition-transform"
        onclick={() =>
          isPlaying ? player.pause.trigger() : player.play.trigger()}
        aria-label={isPlaying ? "Pause" : "Play"}
      >
        {#if isPlaying}
          <Pause size={20} fill="currentColor" />
        {:else}
          <Play size={20} fill="currentColor" class="ml-0.5" />
        {/if}
      </button>

      <button
        class="text-subtext hover:text-white transition-colors hover:scale-105"
        onclick={() => player.next.trigger()}
        aria-label="Next"
      >
        <SkipForward size={22} fill="currentColor" />
      </button>

      <button
        class="hover:text-white transition-colors {repeat !== 'off'
          ? 'text-cyan'
          : 'text-subtext'}"
        onclick={toggleRepeat}
        aria-label="Repeat"
      >
        {#if repeat === "one"}
          <Repeat1 size={18} />
        {:else}
          <Repeat size={18} />
        {/if}
      </button>
    </div>

    <div class="flex items-center gap-3 w-full max-w-md group/progress">
      <span class="text-xs text-subtext w-10 text-right font-mono"
        >{formatDuration(position)}</span
      >
      <div class="relative flex-1 h-1 group cursor-pointer">
        <div
          class="absolute top-0 left-0 h-full bg-white/10 rounded-full w-full group-hover:bg-white/20 transition-colors"
        ></div>

        <div
          class="absolute top-0 left-0 h-full bg-white rounded-full group-hover:bg-cyan transition-colors"
          style="width: {(position / (duration || 1)) * 100}%"
        ></div>

        <input
          type="range"
          min="0"
          max={duration || 100}
          value={position}
          oninput={handleSeek}
          class="absolute -top-1.5 left-0 w-full h-4 opacity-0 cursor-pointer z-10"
          aria-label="Seek"
        />

        <div
          class="absolute top-1/2 -translate-y-1/2 h-3 w-3 bg-white rounded-full opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none shadow-lg"
          style="left: {(position / (duration || 1)) * 100}%"
        ></div>
      </div>
      <span class="text-xs text-subtext w-10 font-mono"
        >{formatDuration(duration)}</span
      >
    </div>
  </div>

  <div class="flex items-center justify-end gap-3">
    <button
      class="text-subtext hover:text-white transition-colors"
      onclick={() => player.setVolume(volume === 0 ? 100 : 0)}
      aria-label="Mute"
    >
      {#if volume === 0}
        <VolumeX size={20} />
      {:else if volume < 50}
        <Volume1 size={20} />
      {:else}
        <Volume2 size={20} />
      {/if}
    </button>
    <div class="relative w-24 h-1 group cursor-pointer">
      <div
        class="absolute top-0 left-0 h-full bg-white/10 rounded-full w-full group-hover:bg-white/20 transition-colors"
      ></div>
      <div
        class="absolute top-0 left-0 h-full bg-white rounded-full group-hover:bg-cyan transition-colors"
        style="width: {volume}%"
      ></div>
      <input
        type="range"
        min="0"
        max="100"
        value={volume}
        oninput={handleVolume}
        class="absolute -top-1.5 left-0 w-full h-4 opacity-0 cursor-pointer z-10"
        aria-label="Volume"
      />
      <div
        class="absolute top-1/2 -translate-y-1/2 h-3 w-3 bg-white rounded-full opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none shadow-lg"
        style="left: {volume}%"
      ></div>
    </div>
  </div>
</div>
