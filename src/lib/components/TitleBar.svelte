<script lang="ts">
  import { afterNavigate } from "$app/navigation";
  import { type UnifiedSearchResult } from "$lib/bindings";
  import { media } from "$lib/stores/player/media.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { createMutation } from "$lib/stores/resource.svelte";
  import { shortcuts } from "$lib/stores/shortcuts.svelte";
  import { formatDuration } from "$lib/util";
  import {
    ChevronLeft,
    ChevronRight,
    Minus,
    Play,
    Search,
    Square,
    User,
    X,
  } from "@lucide/svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";
  import { onMount } from "svelte";
  import Button from "./Button.svelte";
  import Image from "./Image.svelte";

  let isFocused = $state(false);
  let searchQuery = $state("");
  let canGoBack = $state(false);
  let canGoForward = $state(false);
  let searchInput = $state<HTMLInputElement | null>(null);

  // @ts-expect-error
  const nav = typeof navigation !== "undefined" ? navigation : null;

  afterNavigate(({ from }) => {
    isFocused = false;
    searchQuery = "";

    if (nav) {
      canGoBack = nav.canGoBack;
      canGoForward = nav.canGoForward;
    } else {
      if (from) canGoBack = true;
      canGoForward = true;
    }
  });
  const win = getCurrentWindow();
  function handleWindowAction(action: "minimize" | "maximize" | "close") {
    switch (action) {
      case "minimize":
        win.minimize();
        break;
      case "maximize":
        win.toggleMaximize();
        break;
      case "close":
        win.close();
        break;
    }
  }
  let searchResults: UnifiedSearchResult = $state({
    albums: [],
    artists: [],
    tracks: [],
  });
  const search = createMutation("search", {
    onSuccess: (data) => {
      searchResults = data;
    },
    onError: (error) => {
      console.error("Search error:", error);
      searchResults = { albums: [], artists: [], tracks: [] };
    },
  });
  function handleNavigationAction(action: "back" | "forward") {
    if (action === "back") canGoBack && history.back();
    else canGoForward && history.forward();
  }

  onMount(() => {
    const cleanup = shortcuts.on("app.search", () => {
      if (!searchInput) return;
      isFocused = true;
      searchInput.focus();
      searchInput.select();
    });
    return cleanup;
  });
</script>

<header
  class="w-full px-6 py-4 flex items-center justify-between gap-4
           bg-background border-b border-border select-none relative z-50"
  data-tauri-drag-region
>
  <div class="flex items-center gap-2 shrink-0">
    <Button
      disabled={!canGoBack}
      variant="secondary"
      class="p-2 text-subtext border border-border {canGoBack
        ? 'hover:bg-accent hover:text-text'
        : ''}"
      aria-label="Go Back"
      onclick={() => handleNavigationAction("back")}
    >
      <ChevronLeft size={16} />
    </Button>

    <Button
      disabled={!canGoForward}
      variant="secondary"
      class="p-2 text-subtext border border-border {canGoForward
        ? 'hover:bg-accent hover:text-text'
        : ''}"
      aria-label="Go Forward"
      onclick={() => handleNavigationAction("forward")}
    >
      <ChevronRight size={16} />
    </Button>
  </div>

  <div class="flex-1 max-w-xl mx-4 relative group">
    <div
      class="absolute inset-y-0 left-3 flex items-center pointer-events-none"
    >
      <Search
        size={14}
        class="transition-colors duration-300 {isFocused
          ? 'text-cyan'
          : 'text-subtext'}"
      />
    </div>

    <input
      bind:this={searchInput}
      type="text"
      placeholder="Search artists or tracks"
      bind:value={searchQuery}
      oninput={() => search.trigger(searchQuery)}
      onfocus={() => (isFocused = true)}
      onblur={(e) => {
        if (
          e.relatedTarget instanceof Node &&
          e.currentTarget.parentElement?.contains(e.relatedTarget)
        ) {
          return;
        }
        isFocused = false;
      }}
      class="w-full bg-secondary border border-border rounded-md py-2 pl-9 pr-4 text-sm text-text
                   placeholder:text-subtext outline-none transition-all duration-300
                   focus:bg-accent focus:border-border"
    />

    {#if !isFocused && !searchQuery}
      <div
        class="absolute inset-y-0 right-3 flex items-center pointer-events-none"
      >
        <kbd
          class="hidden sm:inline-block px-1.5 py-0.5 text-[10px] font-mono text-subtext bg-primary rounded border border-border"
        >
          Ctrl K
        </kbd>
      </div>
    {/if}

    {#if isFocused && searchQuery && (searchResults.artists.length > 0 || searchResults.albums.length > 0 || searchResults.tracks.length > 0)}
      <div
        class="absolute top-full left-0 right-0 mt-2 bg-secondary border border-border rounded-lg overflow-hidden z-50 max-h-[70vh] overflow-y-auto flex flex-col shadow-xl"
        onmousedown={(e) => e.preventDefault()}
        role="listbox"
        tabindex="-1"
      >
        {#if searchResults.artists.length > 0}
          <div class="p-2">
            <h3
              class="text-xs font-bold text-subtext uppercase tracking-wider px-3 py-2 mb-1"
            >
              Artists
            </h3>
            {#each searchResults.artists.slice(0, 3) as artist}
              <Button
                href={`/library/artists/${artist.id}`}
                variant="ghost"
                class="w-full justify-start p-3 h-auto hover:bg-accent text-left group/item font-normal gap-4"
              >
                <div
                  class="w-10 h-10 rounded-full overflow-hidden bg-primary shrink-0 border border-border"
                >
                  {#if artist.imageUrl}
                    <Image
                      src={artist.imageUrl}
                      alt={artist.name}
                      class="w-full h-full object-cover"
                    />
                  {:else}
                    <div
                      class="w-full h-full flex items-center justify-center text-subtext"
                    >
                      <User size={18} />
                    </div>
                  {/if}
                </div>
                <span
                  class="font-medium text-text text-base group-hover/item:text-cyan transition-colors"
                  >{artist.name}</span
                >
              </Button>
            {/each}
          </div>
        {/if}

        {#if searchResults.albums.length > 0}
          <div class="p-2 border-t border-border">
            <h3
              class="text-xs font-bold text-subtext uppercase tracking-wider px-3 py-2 mb-1"
            >
              Albums
            </h3>
            {#each searchResults.albums.slice(0, 3) as album}
              <Button
                href={`/library/albums/${album.id}`}
                variant="ghost"
                class="w-full justify-start p-3 h-auto hover:bg-accent text-left group/item font-normal gap-4"
              >
                <div
                  class="w-10 h-10 rounded-md overflow-hidden bg-primary shrink-0 border border-border"
                >
                  <Image
                    src={album.coverArt}
                    type="cover"
                    alt={album.title}
                    class="w-full h-full object-cover"
                  />
                </div>
                <div class="flex flex-col overflow-hidden gap-1 items-start">
                  <span
                    class="font-medium text-text text-sm truncate group-hover/item:text-cyan transition-colors leading-none"
                    >{album.title}</span
                  >
                  <span class="text-xs text-subtext truncate"
                    >{album.artistName}</span
                  >
                </div>
              </Button>
            {/each}
          </div>
        {/if}

        {#if searchResults.tracks.length > 0}
          <div class="p-2 border-t border-border">
            <h3
              class="text-xs font-bold text-subtext uppercase tracking-wider px-3 py-2 mb-1"
            >
              Tracks
            </h3>
            {#each searchResults.tracks.slice(0, 5) as track}
              <Button
                onclick={() => player.playTrack.trigger(track.id)}
                variant="ghost"
                class="w-full justify-start p-3 h-auto hover:bg-accent text-left group/item font-normal gap-4"
              >
                <div
                  class="w-10 h-10 rounded-md overflow-hidden bg-primary shrink-0 relative group-hover/item:bg-black/40 transition-colors border border-border"
                >
                  <div
                    class="absolute inset-0 flex items-center justify-center opacity-0 group-hover/item:opacity-100 transition-opacity"
                  >
                    <Play size={16} class="text-white fill-white" />
                  </div>
                  <div
                    class="w-full h-full flex items-center justify-center text-subtext group-hover/item:opacity-0 transition-opacity"
                  >
                    <Image
                      src={media.album(track.albumId).data?.coverArt}
                      type="cover"
                      alt={track.title}
                      class="w-full h-full object-cover"
                    />
                  </div>
                </div>
                <div
                  class="flex flex-col overflow-hidden flex-1 gap-1 items-start"
                >
                  <span
                    class="font-medium text-text text-sm truncate group-hover/item:text-cyan transition-colors leading-none"
                    >{track.title}</span
                  >
                  <span class="text-xs text-subtext truncate"
                    >{track.artistName} • {track.albumTitle}</span
                  >
                </div>
                <span class="text-xs text-subtext font-mono"
                  >{formatDuration(track.durationSec)}</span
                >
              </Button>
            {/each}
          </div>
        {/if}

        {#if searchResults.artists.length > 3 || searchResults.albums.length > 3 || searchResults.tracks.length > 5}
          <div class="p-2 border-t border-border bg-secondary/50">
            <Button
              variant="ghost"
              class="w-full p-2 text-xs text-subtext hover:text-text hover:bg-accent"
              href="/explore?query={encodeURIComponent(searchQuery)}"
            >
              See all results
            </Button>
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-4 shrink-0">
    <div class="h-6 w-px bg-border mx-1"></div>

    <div class="flex items-center gap-1">
      <Button
        variant="ghost"
        class="p-2 text-subtext hover:text-text hover:bg-secondary"
        onclick={() => handleWindowAction("minimize")}
      >
        <Minus size={16} />
      </Button>

      <Button
        variant="ghost"
        class="p-2 text-subtext hover:text-text hover:bg-secondary"
        onclick={() => handleWindowAction("maximize")}
      >
        <Square size={14} />
      </Button>

      <Button
        variant="ghost"
        class="p-2 text-subtext hover:bg-red hover:text-white"
        onclick={() => handleWindowAction("close")}
      >
        <X size={16} />
      </Button>
    </div>
  </div>
</header>
