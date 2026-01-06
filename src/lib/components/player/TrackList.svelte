<script lang="ts">
  import { goto } from "$app/navigation";
  import type { Track } from "$lib/bindings";
  import { contextMenu } from "$lib/hooks";
  import { player } from "$lib/stores/player/player.svelte";
  import { queue } from "$lib/stores/player/queue.svelte";
  import { formatDuration } from "$lib/util";
  import { CirclePlus, Disc, ListPlus, Mic, Pause, Play } from "@lucide/svelte";

  let {
    tracks,
    showAlbum = true,
    showCover = false,
  }: {
    tracks: Track[];
    showAlbum?: boolean;
    showCover?: boolean;
  } = $props();

  function handlePlay(index: number) {
    player.playTracks(tracks, index);
  }
</script>

<div class="w-full text-sm">
  <div
    class="grid gap-4 px-4 py-2 border-b border-border text-subtext font-medium select-none uppercase text-xs tracking-wider
            {showAlbum
      ? 'grid-cols-[auto_1fr_1fr_auto]'
      : 'grid-cols-[auto_1fr_auto]'}"
  >
    <span class="w-8 text-center">#</span>
    <span>Title</span>
    {#if showAlbum}<span>Album</span>{/if}
    <span class="text-right">Time</span>
  </div>
  <div class="flex flex-col pb-4 mt-2">
    {#each tracks as track, i (track.id)}
      {@const isPlaying = queue.currentTrack?.id === track.id}
      <button
        class="group grid gap-4 px-4 py-2.5 hover:bg-secondary/50 rounded-md items-center text-left transition-colors border border-transparent hover:border-border/50
                {showAlbum
          ? 'grid-cols-[auto_1fr_1fr_auto]'
          : 'grid-cols-[auto_1fr_auto]'}"
        onclick={() => handlePlay(i)}
        use:contextMenu={[
          {
            type: "label",
            label: track.title,
          },
          {
            type: "action",
            label: "Play",
            leftIcon: Play,
            onClick: () => handlePlay(i),
          },
          {
            type: "separator",
          },
          {
            type: "action",
            label: "Add to Queue",
            leftIcon: ListPlus,
            onClick: () => queue.add.trigger(track.id),
          },
          {
            type: "action",
            label: "Play Next",
            leftIcon: CirclePlus,
            onClick: () => queue.addNext.trigger(track.id),
          },
          {
            type: "separator",
          },
          {
            type: "action",
            label: "Go to Artist",
            leftIcon: Mic,
            onClick: () => goto(`/library/artists/${track.artistId}`),
          },
          {
            type: "action",
            label: "Go to Album",
            leftIcon: Disc,
            onClick: () => goto(`/library/albums/${track.albumId}`),
          },
        ]}
      >
        <span
          class="w-8 text-center text-subtext group-hover:text-text relative flex items-center justify-center font-mono text-xs"
        >
          {#if isPlaying}
            {@const Icon = player.state?.paused ? Pause : Play}
            <Icon size={12} class="fill-current" />
          {:else}
            <span class="group-hover:hidden">{track.trackNumber || i + 1}</span>
            <Play
              size={12}
              class="hidden group-hover:inline-block fill-current"
            />
          {/if}
        </span>

        <div class="flex items-center gap-3 overflow-hidden">
          {#if showCover && track.albumId}{/if}
          <div class="flex flex-col overflow-hidden gap-0.5">
            <span class="text-text font-medium truncate leading-none"
              >{track.title}</span
            >
            <a
              href="/library/artists/{track.artistId}"
              class="text-subtext text-xs truncate group-hover:text-text/80 transition-colors"
              >{track.artistName}</a
            >
          </div>
        </div>

        {#if showAlbum}
          <a
            href="/library/albums/{track.albumId}"
            class="text-subtext truncate group-hover:text-text transition-colors text-xs"
            >{track.albumTitle}</a
          >
        {/if}
        <span class="text-subtext text-right font-mono text-xs"
          >{formatDuration(track.durationSec)}</span
        >
      </button>
    {/each}
  </div>
</div>
