<script lang="ts">
  import Image from "$lib/components/Image.svelte";
  import { tooltip } from "$lib/hooks";
  import { media } from "$lib/stores/player/media.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { queue } from "$lib/stores/player/queue.svelte";
  import { formatDuration } from "$lib/util";
  import {
    HeadphoneOff,
    Headphones,
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
  let exclusiveMode = $derived(player.state?.exclusive ?? false);
  let canNext = $derived(
    queue.data && queue.data?.currentIndex < queue.data?.tracks.length - 1
  );
  let canBack = $derived(queue.data && queue.data?.currentIndex > 0);
  let coverArt = $derived(
    currentTrack ? media.album(currentTrack.albumId)?.data?.coverArt : null
  );

  function handleSeek(e: Event) {
    const target = e.target as HTMLInputElement;
    player.seek(Number(target.value));
  }

  function handleVolume(e: Event) {
    const target = e.target as HTMLInputElement;
    if (e instanceof WheelEvent) {
      e.preventDefault();
      const delta = -e.deltaY || -e.detail;
      let newVolume = Number(target.value) + (delta > 0 ? 5 : -5);
      newVolume = Math.max(0, Math.min(100, newVolume));
      player.setVolume(newVolume);
      return;
    }
    player.setVolume(Number(target.value));
  }

  function toggleRepeat() {
    const modes = ["off", "all", "one"] as const;
    const nextIndex = (modes.indexOf(repeat) + 1) % modes.length;
    player.setRepeat.trigger(modes[nextIndex]);
  }
</script>

<div
  class="h-24 bg-background border-t border-border grid grid-cols-[3fr_4fr_3fr] items-center px-6 z-50 select-none"
>
  <div class="flex items-center gap-4 justify-start min-w-0">
    {#if currentTrack}
      <div
        class="h-14 w-14 bg-secondary rounded-md overflow-hidden shrink-0 border border-border"
      >
        <Image
          src={coverArt}
          type="cover"
          alt={currentTrack.albumTitle}
          class="h-full w-full object-cover"
        />
      </div>
      <div class="flex flex-col min-w-0 justify-center gap-1">
        <a
          href="/library/albums/{currentTrack.albumId}"
          class="text-sm font-medium text-text truncate leading-none"
          >{currentTrack.title}</a
        >
        <div class="flex items-center gap-2">
          <a
            href="/library/artists/{currentTrack.artistId}"
            class="text-xs text-subtext truncate hover:text-text transition-colors cursor-pointer"
            >{currentTrack.artistName}</a
          >
          {#if currentTrack.bitrate}
            <span
              class="text-[10px] text-subtext border border-border px-1.5 py-0.5 rounded-sm font-mono tracking-tight leading-none"
            >
              {currentTrack.bitrate}kbps
            </span>
          {/if}
        </div>
      </div>
    {/if}
  </div>

  <div class="flex flex-col items-center justify-center gap-2">
    <div class="flex items-center gap-4">
      <button
        class="p-2 rounded-md hover:bg-secondary/50 text-subtext hover:text-text transition-all disabled:opacity-50 {shuffle
          ? 'text-blue bg-blue/10'
          : ''}"
        onclick={() => player.toggleShuffle.trigger()}
        aria-label="Shuffle"
        use:tooltip={"Shuffle"}
      >
        <Shuffle size={16} />
      </button>

      <button
        class="p-2 rounded-md hover:bg-secondary/50 text-text transition-all hover:scale-105 active:scale-95 {!canBack
          ? 'opacity-50 cursor-not-allowed'
          : ''}"
        onclick={() => player.prev.trigger()}
        aria-label="Previous"
        use:tooltip={"Previous"}
        disabled={!canBack}
      >
        <SkipBack size={20} fill="currentColor" />
      </button>

      <button
        class="h-10 w-10 bg-text rounded-full flex items-center justify-center text-primary hover:scale-105 hover:bg-white transition-all shadow-sm"
        onclick={() =>
          isPlaying ? player.pause.trigger() : player.play.trigger()}
        aria-label={isPlaying ? "Pause" : "Play"}
        use:tooltip={isPlaying ? "Pause" : "Play"}
      >
        {#if isPlaying}
          <Pause size={20} fill="currentColor" />
        {:else}
          <Play size={20} fill="currentColor" class="ml-0.5" />
        {/if}
      </button>

      <button
        class="p-2 rounded-md hover:bg-secondary/50 text-text transition-all hover:scale-105 active:scale-95 {!canNext
          ? 'opacity-50 cursor-not-allowed'
          : ''}"
        disabled={!canNext}
        onclick={() => player.next.trigger()}
        aria-label="Next"
        use:tooltip={"Next"}
      >
        <SkipForward size={20} fill="currentColor" />
      </button>

      <button
        class="p-2 rounded-md hover:bg-secondary/50 text-subtext hover:text-text transition-all {repeat !==
        'off'
          ? 'text-blue bg-blue/10'
          : ''}"
        onclick={toggleRepeat}
        aria-label="Repeat"
        use:tooltip={`Repeat: ${repeat}`}
      >
        {#if repeat === "one"}
          <Repeat1 size={16} />
        {:else}
          <Repeat size={16} />
        {/if}
      </button>
    </div>

    <div class="flex items-center gap-3 w-full max-w-lg group/progress">
      <span class="text-xs text-subtext w-10 text-right font-mono tabular-nums"
        >{formatDuration(position)}</span
      >
      <div class="relative flex-1 h-4 flex items-center group cursor-pointer">
        <div
          class="absolute w-full h-1 bg-secondary rounded-full overflow-hidden"
        >
          <div
            class="h-full bg-text rounded-full group-hover:bg-blue transition-colors duration-100"
            style="width: {(position / (duration || 1)) * 100}%"
          ></div>
        </div>

        <input
          type="range"
          min="0"
          max={duration || 100}
          value={position}
          oninput={handleSeek}
          class="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
          aria-label="Seek"
        />

        <div
          class="absolute h-3 w-3 bg-text rounded-full opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none shadow-sm"
          style="left: {(position / (duration || 1)) *
            100}%; transform: translateX(-50%)"
        ></div>
      </div>
      <span class="text-xs text-subtext w-10 font-mono tabular-nums"
        >{formatDuration(duration)}</span
      >
    </div>
  </div>

  <div class="flex items-center justify-end gap-3">
    <button
      class="text-subtext hover:text-text transition-colors p-2 rounded-md hover:bg-secondary/50"
      onclick={() => player.toggleExclusiveMode.trigger(!exclusiveMode)}
      aria-label="Toggle Exclusive mode"
      use:tooltip={`Exclusive Mode: ${exclusiveMode ? "On" : "Off"}`}
    >
      {#if exclusiveMode}
        <Headphones size={18} />
      {:else}
        <HeadphoneOff size={18} />
      {/if}
    </button>
    <button
      class="text-subtext hover:text-text transition-colors p-2 rounded-md hover:bg-secondary/50"
      onclick={() => (player.muted ? player.unmute() : player.mute())}
      aria-label="Mute"
    >
      {#if volume === 0}
        <VolumeX size={18} />
      {:else if volume < 50}
        <Volume1 size={18} />
      {:else}
        <Volume2 size={18} />
      {/if}
    </button>

    <div class="relative w-24 h-4 flex items-center group cursor-pointer">
      <div
        class="absolute w-full h-1 bg-secondary rounded-full overflow-hidden"
      >
        <div
          class="h-full bg-text rounded-full group-hover:bg-blue transition-colors duration-100"
          style="width: {volume}%"
        ></div>
      </div>

      <input
        type="range"
        min="0"
        max="100"
        value={volume}
        oninput={handleVolume}
        onwheel={handleVolume}
        use:tooltip={`Volume: ${Math.round(volume)}%`}
        class="absolute inset-0 w-full h-full opacity-0 cursor-pointer z-10"
        aria-label="Volume"
      />

      <div
        class="absolute h-3 w-3 bg-text rounded-full opacity-0 group-hover:opacity-100 transition-opacity pointer-events-none shadow-sm"
        style="left: {volume}%; transform: translateX(-50%)"
      ></div>
    </div>
  </div>
</div>
