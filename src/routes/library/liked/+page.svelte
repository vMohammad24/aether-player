<script lang="ts">
  import Button from "$lib/components/Button.svelte";
  import TrackList from "$lib/components/player/TrackList.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { createResource } from "$lib/stores/resource.svelte";
  import { Heart, Play } from "@lucide/svelte";

  const favorites = createResource("getFavorites");

  function playAll() {
    if (favorites.data && favorites.data.length > 0) {
      player.playTracks(favorites.data, 0);
    }
  }
</script>

<div class="flex flex-col h-full gap-8">
  <header class="flex flex-col md:flex-row gap-8 items-end">
    <div
      class="w-48 h-48 md:w-64 md:h-64 rounded-2xl overflow-hidden shrink-0
             bg-mauve flex items-center justify-center border border-border"
    >
      <Heart size={80} class="text-primary fill-primary" />
    </div>

    <div class="flex flex-col gap-2 pb-2 w-full">
      <span class="text-sm font-medium text-subtext uppercase tracking-wider"
        >Playlist</span
      >
      <h1 class="text-4xl md:text-6xl font-bold text-text tracking-tight">
        Liked Songs
      </h1>
      <div class="flex items-center gap-2 text-text text-lg">
        <span class="text-text font-medium">You</span>
        <span class="text-subtext">•</span>
        <span class="text-subtext">
          {#if favorites.data}
            {favorites.data.length} songs
          {:else}
            0 songs
          {/if}
        </span>
      </div>

      <div class="mt-4">
        <Button
          variant="primary"
          size="lg"
          onclick={playAll}
          class="gap-2 px-8 rounded-full"
        >
          <Play size={20} fill="currentColor" />
          Play
        </Button>
      </div>
    </div>
  </header>

  {#if favorites.data}
    <section>
      <TrackList tracks={favorites.data} showAlbum={true} />
    </section>
  {:else if favorites.loading}
    <div class="p-8 text-center text-subtext">Loading favorites...</div>
  {:else if favorites.error}
    <div class="p-8 text-center text-red">Error: {favorites.error}</div>
  {/if}
</div>
