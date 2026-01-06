<script lang="ts">
  import { page } from "$app/state";
  import Button from "$lib/components/Button.svelte";
  import AlbumCard from "$lib/components/player/AlbumCard.svelte";
  import ArtistCard from "$lib/components/player/ArtistCard.svelte";
  import TrackList from "$lib/components/player/TrackList.svelte";
  import { media } from "$lib/stores/player/media.svelte";
  import { createResource } from "$lib/stores/resource.svelte";
  import { Search, X } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { fade } from "svelte/transition";

  let query = $state("");

  onMount(() => {
    const queryParam = page.url.searchParams.get("query");
    if (queryParam) {
      query = queryParam;
    }
  });
  const searchResults = createResource("search", () => query);

  const randomAlbums = media.randomAlbums(10);
  const mostPlayed = media.mostPlayed(5);
  const newArrivals = media.recentAlbums(10);
  const genres = media.genres();

  function clearSearch() {
    query = "";
  }
</script>

<div class="flex flex-col h-full gap-8 pb-10">
  <header class="flex flex-col gap-6">
    <div class="flex flex-col gap-2">
      <h1 class="text-3xl font-bold text-text">Explore</h1>
      <p class="text-subtext">explore all your trackssssss.</p>
    </div>

    <div class="relative max-w-2xl group">
      <Search
        class="absolute left-4 top-1/2 -translate-y-1/2 text-subtext group-focus-within:text-cyan transition-colors"
        size={20}
      />
      <input
        type="text"
        bind:value={query}
        placeholder="Search for tracks, albums, or artists..."
        class="w-full pl-12 pr-12 py-3 bg-secondary border border-border rounded-xl text-lg text-text
               focus:outline-none focus:border-cyan focus:bg-secondary transition-all placeholder:text-subtext/50"
      />
      {#if query}
        <button
          onclick={clearSearch}
          class="absolute right-4 top-1/2 -translate-y-1/2 text-subtext hover:text-text p-1"
        >
          <X size={20} />
        </button>
      {/if}
    </div>
  </header>

  {#if query}
    {#if searchResults.loading}
      <div class="flex items-center justify-center py-20">
        <div
          class="w-8 h-8 border-2 border-cyan border-t-transparent rounded-full animate-spin"
        ></div>
      </div>
    {:else if searchResults.data}
      <div class="flex flex-col gap-12 animate-slide-up" in:fade>
        {#if searchResults.data.tracks.length > 0}
          <section>
            <h2 class="text-xl font-bold text-text mb-4">Tracks</h2>
            <TrackList tracks={searchResults.data.tracks} showAlbum={true} />
          </section>
        {/if}

        {#if searchResults.data.albums.length > 0}
          <section>
            <h2 class="text-xl font-bold text-text mb-4">Albums</h2>
            <div
              class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 gap-4"
            >
              {#each searchResults.data.albums as album}
                <AlbumCard {album} />
              {/each}
            </div>
          </section>
        {/if}

        {#if searchResults.data.artists.length > 0}
          <section>
            <h2 class="text-xl font-bold text-text mb-4">Artists</h2>
            <div
              class="grid grid-cols-2 sm:grid-cols-4 md:grid-cols-5 lg:grid-cols-6 gap-4"
            >
              {#each searchResults.data.artists as artist}
                <ArtistCard {artist} />
              {/each}
            </div>
          </section>
        {/if}

        {#if searchResults.data.tracks.length === 0 && searchResults.data.albums.length === 0 && searchResults.data.artists.length === 0}
          <div
            class="flex flex-col items-center justify-center py-20 text-subtext italic"
          >
            No results found for "{query}"
          </div>
        {/if}
      </div>
    {/if}
  {:else}
    <div class="flex flex-col gap-10 animate-in" in:fade>
      {#if randomAlbums.data && randomAlbums.data.length > 0}
        <section class="flex flex-col gap-4">
          <h2 class="text-xl font-bold text-text">Rediscover</h2>
          <div
            class="flex gap-4 overflow-x-auto pb-4 scrollbar-hide -mx-2 px-2"
          >
            {#each randomAlbums.data as album}
              <div class="w-40 sm:w-48 shrink-0">
                <AlbumCard {album} />
              </div>
            {/each}
          </div>
        </section>
      {/if}

      {#if mostPlayed.data && mostPlayed.data.length > 0}
        <section class="flex flex-col gap-4">
          <h2 class="text-xl font-bold text-text">On Repeat</h2>
          <TrackList
            tracks={mostPlayed.data}
            showAlbum={true}
            showCover={true}
          />
        </section>
      {/if}

      {#if newArrivals.data && newArrivals.data.length > 0}
        <section class="flex flex-col gap-4">
          <h2 class="text-xl font-bold text-text">New Arrivals</h2>
          <div
            class="flex gap-4 overflow-x-auto pb-4 scrollbar-hide -mx-2 px-2"
          >
            {#each newArrivals.data as album}
              <div class="w-40 sm:w-48 shrink-0">
                <AlbumCard {album} />
              </div>
            {/each}
          </div>
        </section>
      {/if}

      {#if genres.data && genres.data.length > 0}
        <section class="flex flex-col gap-4">
          <h2 class="text-xl font-bold text-text">Browse by Genre</h2>
          <div class="flex flex-wrap gap-2">
            {#each genres.data.slice(0, 20) as genre}
              <Button
                href="/library/genre/{encodeURIComponent(genre.name)}"
                class="px-4 py-2 bg-secondary hover:bg-accent border border-border rounded-full text-sm font-medium transition-colors text-subtext hover:text-text"
              >
                {genre.name}
              </Button>
            {/each}
          </div>
        </section>
      {/if}
    </div>
  {/if}
</div>
