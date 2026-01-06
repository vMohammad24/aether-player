<script lang="ts">
  import { page } from "$app/state";
  import Image from "$lib/components/Image.svelte";
  import AlbumCard from "$lib/components/player/AlbumCard.svelte";
  import { media } from "$lib/stores/player/media.svelte";

  const id = page.params.id ?? "";

  const artist = media.artist(id);
  const albums = media.artistAlbums(id);
</script>

<div class="flex flex-col h-full gap-8">
  {#if artist.data}
    <header class="flex flex-col md:flex-row gap-8 items-center md:items-end">
      <div
        class="w-48 h-48 md:w-64 md:h-64 rounded-full overflow-hidden shrink-0 bg-secondary border border-border"
      >
        {#if artist.data.imageUrl}
          <Image
            src={artist.data.imageUrl}
            alt={artist.data.name}
            type="cover"
            class="w-full h-full object-cover"
          />
        {:else}
          <div
            class="w-full h-full flex items-center justify-center bg-secondary text-4xl font-bold text-subtext"
          >
            {artist.data.name.slice(0, 2).toUpperCase()}
          </div>
        {/if}
      </div>

      <div class="flex flex-col gap-2 pb-2 w-full text-center md:text-left">
        <span class="text-sm font-medium text-subtext uppercase tracking-wider"
          >Artist</span
        >
        <h1 class="text-4xl md:text-6xl font-bold text-text tracking-tight">
          {artist.data.name}
        </h1>

        {#if artist.data.bio}
          <p class="text-subtext max-w-2xl line-clamp-3">
            {artist.data.bio}
          </p>
        {/if}
      </div>
    </header>

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
  {:else if artist.loading}
    <div class="p-8 text-center text-subtext">Loading artist...</div>
  {:else if artist.error}
    <div class="p-8 text-center text-red">Error: {artist.error}</div>
  {/if}
</div>
