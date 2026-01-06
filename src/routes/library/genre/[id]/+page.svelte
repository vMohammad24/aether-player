<script lang="ts">
  import { page } from "$app/state";
  import TrackList from "$lib/components/player/TrackList.svelte";
  import { createResource } from "$lib/stores/resource.svelte";

  const genre = $derived(decodeURIComponent(page.params.id ?? ""));
  const tracks = createResource("getGenreTracks", () => genre);
</script>

<div class="flex flex-col h-full gap-6">
  <header class="flex flex-col gap-2 pb-6 border-b border-border">
    <span class="text-xs font-bold text-subtext uppercase tracking-widest">Genre</span>
    <h1 class="text-4xl font-bold text-text tracking-tight">{genre}</h1>
  </header>

  {#if tracks.data}
     <div class="flex flex-col gap-4">
        <h2 class="text-lg font-bold text-text">Tracks ({tracks.data.length})</h2>
        <TrackList tracks={tracks.data} showAlbum={true} showCover={true} />
     </div>
  {:else if tracks.loading}
    <div class="p-8 text-center text-subtext">Loading tracks...</div>
  {:else if tracks.error}
    <div class="p-8 text-center text-red-500">Error: {tracks.error}</div>
  {:else}
     <div class="p-8 text-center text-subtext">No tracks found.</div>
  {/if}
</div>
