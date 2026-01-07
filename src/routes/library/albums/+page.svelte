<script lang="ts">
  import AlbumCard from "$lib/components/player/AlbumCard.svelte";
  import { media } from "$lib/stores/player/media.svelte";
  const albums = media.recentAlbums(512);
</script>

<div class="flex flex-col h-full gap-8">
  {#if albums.data}
    <section>
      <h2 class="text-xl font-bold text-text mb-4">Albums</h2>
      <div
        class="grid grid-cols-2 md:grid-cols-3 lg:grid-cols-4 xl:grid-cols-5 gap-4"
      >
        {#each albums.data as album}
          <AlbumCard {album} />
        {/each}
      </div>
    </section>
  {:else if albums.loading}
    <div class="p-8 text-center text-subtext">Loading albums...</div>
  {/if}
</div>
