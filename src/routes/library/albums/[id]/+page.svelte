<script lang="ts">
  import { page } from "$app/state";
  import Button from "$lib/components/Button.svelte";
  import Image from "$lib/components/Image.svelte";
  import TrackList from "$lib/components/player/TrackList.svelte";
  import { media } from "$lib/stores/player/media.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { Play } from "@lucide/svelte";

  const id = page.params.id ?? "";

  const album = media.album(id);
  const tracks = media.albumTracks(id);

  async function playAlbum() {
    if (tracks.data && tracks.data.length > 0) {
      player.playTracks(tracks.data, 0);
    }
  }
</script>

<div class="flex flex-col h-full gap-8">
  {#if album.data}
    <header class="flex flex-col md:flex-row gap-8 items-end">
      <div
        class="w-48 h-48 md:w-64 md:h-64 rounded-2xl overflow-hidden shrink-0 bg-secondary border border-border"
      >
        <Image
          src={album.data.coverArt}
          alt={album.data.title}
          type="cover"
          class="w-full h-full object-cover"
        />
      </div>

      <div class="flex flex-col gap-2 pb-2 w-full">
        <span class="text-sm font-medium text-subtext uppercase tracking-wider"
          >Album</span
        >
        <h1 class="text-4xl md:text-5xl font-bold text-text tracking-tight">
          {album.data.title}
        </h1>
        <div class="flex items-center gap-2 text-text text-lg">
          <a
            href="/library/artists/{album.data.artistId}"
            class="hover:underline hover:text-white font-medium"
          >
            {album.data.artistName}
          </a>
          <span class="text-subtext">•</span>
          {#if album.data.year}
            <span class="text-subtext">{album.data.year}</span>
            <span class="text-subtext">•</span>
          {/if}
          <span class="text-subtext">{album.data.trackCount} Songs</span>
        </div>

        <div class="mt-4">
          <Button
            variant="primary"
            size="lg"
            onclick={playAlbum}
            class="gap-2 px-8 rounded-full"
          >
            <Play size={20} fill="currentColor" />
            Play
          </Button>
        </div>
      </div>
    </header>

    {#if tracks.data}
      <section>
        <TrackList tracks={tracks.data} showAlbum={false} />
      </section>
    {:else if tracks.loading}
      <div class="p-8 text-center text-subtext">Loading tracks...</div>
    {/if}
  {:else if album.loading}
    <div class="p-8 text-center text-subtext">Loading album...</div>
  {:else if album.error}
    <div class="p-8 text-center text-red">Error: {album.error}</div>
  {/if}
</div>
