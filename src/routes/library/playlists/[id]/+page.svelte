<script lang="ts">
  import { page } from "$app/state";
  import Button from "$lib/components/Button.svelte";
  import Image from "$lib/components/Image.svelte";
  import TrackList from "$lib/components/player/TrackList.svelte";
  import { media } from "$lib/stores/player/media.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { Music, Play } from "@lucide/svelte";

  const id = page.params.id ?? "";

  const parts = id.split(":");
  const providerId = parts.length >= 2 ? parts[0] : "local";
  const playlistId = parts.length >= 2 ? parts.slice(1).join(":") : id;

  const tracks = media.playlistTracks(providerId, playlistId);

  const playlists = media.playlists(providerId);

  const playlist = $derived(playlists.data?.find((p) => p.id === playlistId));

  function playPlaylist() {
    if (tracks.data && tracks.data.length > 0) {
      player.playTrack.trigger(tracks.data[0].id);
    }
  }
</script>

<div class="flex flex-col h-full gap-8">
  {#if playlist}
    <header class="flex flex-col md:flex-row gap-8 items-end">
      <div
        class="w-48 h-48 md:w-64 md:h-64 rounded-2xl overflow-hidden shrink-0 bg-secondary flex items-center justify-center border border-border"
      >
        {#if playlist.coverArt}
          <Image
            src={playlist.coverArt}
            alt={playlist.name}
            type="cover"
            class="w-full h-full object-cover"
          />
        {:else}
          <div
            class="w-full h-full flex items-center justify-center bg-secondary"
          >
            <Music size={64} class="text-subtext" />
          </div>
        {/if}
      </div>

      <div class="flex flex-col gap-2 pb-2 w-full">
        <span class="text-sm font-medium text-subtext uppercase tracking-wider"
          >Playlist</span
        >
        <h1 class="text-4xl md:text-5xl font-bold text-text tracking-tight">
          {playlist.name}
        </h1>
        <div class="flex items-center gap-2 text-text text-lg">
          <span class="font-medium text-text">{playlist.owner}</span>
          <span class="text-subtext">•</span>
          <span class="text-subtext"
            >{tracks.data ? tracks.data.length : playlist.trackCount} Songs</span
          >
        </div>

        <div class="mt-4">
          <Button
            variant="primary"
            size="lg"
            onclick={playPlaylist}
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
        <TrackList tracks={tracks.data} showAlbum={true} />
      </section>
    {:else if tracks.loading}
      <div class="p-8 text-center text-subtext">Loading tracks...</div>
    {/if}
  {:else if playlists.loading}
    <div class="p-8 text-center text-subtext">Loading playlist info...</div>
  {:else if playlists.error}
    <div class="p-8 text-center text-red">
      Error loading playlists: {playlists.error}
    </div>
  {:else}
    <div class="p-8 text-center text-red">
      Playlist not found (ID: {playlistId}, Provider: {providerId})
    </div>
  {/if}
</div>
