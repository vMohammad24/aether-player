<script lang="ts">
  import { commands, type Album } from "$lib/bindings";
  import Image from "$lib/components/Image.svelte";
  import AlbumCard from "$lib/components/player/AlbumCard.svelte";
  import { media } from "$lib/stores/player/media.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { ArrowRight, Play, Sparkles } from "@lucide/svelte";
  import { fade } from "svelte/transition";

  const recentAlbums = media.recentAlbums(12);
  const spotlightResource = media.randomAlbums(1);

  let spotlight = $derived(spotlightResource.data?.[0]);

  function getGreeting() {
    const hour = new Date().getHours();
    if (hour < 12) return "Good Morning";
    if (hour < 18) return "Good Afternoon";
    return "Good Evening";
  }

  async function playAlbum(album: Album) {
    try {
      const res = await commands.getAlbumTracks(album.id);
      if (res.status === "ok") {
        const tracks = res.data;
        if (tracks.length === 0) {
          toast.error("No tracks found in this album");
          return;
        }
        await player.playTracks(tracks, 0);
      } else {
        toast.error(`Failed to play album: ${res.error}`);
      }
    } catch (e) {
      console.error(e);
      toast.error("An error occurred");
    }
  }
</script>

<div class="flex flex-col gap-8 pb-10">
  <header class="flex flex-col gap-1">
    <h1 class="text-3xl font-bold text-text">{getGreeting()}</h1>
  </header>

  {#if spotlight}
    <section
      in:fade
      class="relative overflow-hidden rounded-3xl bg-secondary border border-border shadow-2xl group"
    >
      <div class="absolute inset-0 z-0">
        <Image
          src={spotlight.coverArt}
          type="cover"
          alt=""
          class="w-full h-full object-cover opacity-20 blur-3xl scale-110 transition-transform duration-1000 group-hover:scale-125"
        />
        <div
          class="absolute inset-0 bg-linear-to-r from-secondary/95 via-secondary/70 to-transparent"
        ></div>
      </div>

      <div
        class="relative z-10 p-8 md:p-10 flex flex-col md:flex-row gap-8 items-center md:items-end"
      >
        <div
          class="shrink-0 shadow-2xl rounded-lg overflow-hidden w-40 h-40 md:w-56 md:h-56 border border-white/10 group-hover:shadow-blue/20 transition-all duration-500"
        >
          <Image
            src={spotlight.coverArt}
            alt={spotlight.title}
            type="cover"
            class="w-full h-full object-cover"
          />
        </div>

        <div
          class="flex flex-col gap-4 text-center md:text-left flex-1 items-center md:items-start"
        >
          <div
            class="flex items-center gap-2 text-blue font-medium tracking-wider text-xs uppercase bg-accent/10 px-3 py-1 rounded-full border border-accent/20"
          >
            <Sparkles size={14} />
            <span>Spotlight</span>
          </div>

          <div class="flex flex-col gap-1">
            <h2
              class="text-2xl md:text-4xl font-bold text-text leading-tight text-balance"
            >
              {spotlight.title}
            </h2>
            <a
              href="/library/artists/{spotlight.artistId}"
              class="text-lg md:text-xl text-subtext"
            >
              {spotlight.artistName}
            </a>
          </div>

          <div class="mt-2 flex gap-3">
            <button
              onclick={() => playAlbum(spotlight!)}
              class="flex items-center gap-2 px-6 py-2.5 bg-text text-primary font-bold rounded-full hover:bg-accent hover:text-white transition-all transform hover:scale-105 active:scale-95 shadow-lg cursor-pointer"
            >
              <Play size={18} fill="currentColor" />
              Play
            </button>
            <a
              href="/library/albums/{spotlight.id}"
              class="flex items-center gap-2 px-5 py-2.5 bg-white/5 text-text font-medium rounded-full hover:bg-white/10 transition-colors border border-white/10"
            >
              Details
            </a>
          </div>
        </div>
      </div>
    </section>
  {/if}

  {#if recentAlbums.data && recentAlbums.data.length > 0}
    <section class="flex flex-col gap-6 mt-4">
      <div class="flex items-center justify-between">
        <h2 class="text-xl font-bold text-text flex items-center gap-2">
          Recently Added
        </h2>
        <a
          href="/library/albums"
          class="text-sm text-subtext hover:text-text flex items-center gap-1 transition-colors"
        >
          View All <ArrowRight size={16} />
        </a>
      </div>
      <div
        class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 lg:grid-cols-5 xl:grid-cols-6 gap-6"
      >
        {#each recentAlbums.data as album (album.id)}
          <AlbumCard {album} />
        {/each}
      </div>
    </section>
  {:else if recentAlbums.loading}
    <div class="flex flex-col items-center justify-center py-20 gap-4">
      <div
        class="w-8 h-8 border-2 border-blue border-t-transparent rounded-full animate-spin"
      ></div>
      <p class="text-subtext">Loading your music...</p>
    </div>
  {:else if recentAlbums.data && recentAlbums.data.length === 0}
    <div
      class="flex flex-col items-center justify-center py-20 text-center gap-4 bg-secondary rounded-3xl border border-border"
    >
      <div class="p-4 bg-primary rounded-full">
        <Sparkles size={48} class="text-subtext" />
      </div>
      <div class="flex flex-col gap-1">
        <h3 class="text-xl font-bold text-text">Your library is empty</h3>
        <p class="text-subtext max-w-xs">
          Add some local folders or connect a service in settings to start
          listening.
        </p>
      </div>
      <a
        href="/settings/sources"
        class="mt-2 px-6 py-2 bg-text text-primary font-bold rounded-full hover:bg-blue transition-colors"
      >
        Add Source
      </a>
    </div>
  {/if}
</div>
