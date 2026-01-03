<script lang="ts">
  import { afterNavigate } from "$app/navigation";
  import { type UnifiedSearchResult } from "$lib/bindings";
  import { media } from "$lib/stores/player/media.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { createMutation } from "$lib/stores/resource.svelte";
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
  import Image from "./Image.svelte";

  let isFocused = $state(false);
  let searchQuery = $state("");
  let canGoBack = $state(false);
  let canGoForward = $state(true);

  afterNavigate(({ from }) => {
    isFocused = false;
    searchQuery = "";
    if (from) canGoBack = true;
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
      console.log("Search results:", data);
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
</script>

<header
  class="w-full px-6 py-4 flex items-center justify-between gap-4
           bg-background/80 backdrop-blur-xl border-b border-white/5 select-none relative z-50"
  data-tauri-drag-region
>
  <div class="flex items-center gap-3 shrink-0">
    <button
      disabled={!canGoBack}
      class="p-2 rounded-full bg-black/20 text-white border border-white/5
                   transition-all duration-200
                   {canGoBack
        ? 'hover:bg-white/10 hover:text-cyan'
        : 'opacity-30 cursor-default'}"
      aria-label="Go Back"
      onclick={() => handleNavigationAction("back")}
    >
      <ChevronLeft size={18} />
    </button>

    <button
      disabled={!canGoForward}
      class="p-2 rounded-full bg-black/20 text-white border border-white/5
                   transition-all duration-200
                   {canGoForward
        ? 'hover:bg-white/10 hover:text-cyan'
        : 'opacity-30 cursor-default'}"
      aria-label="Go Forward"
      onclick={() => handleNavigationAction("forward")}
    >
      <ChevronRight size={18} />
    </button>
  </div>

  <div class="flex-1 max-w-xl mx-4 relative group">
    <div
      class="absolute inset-y-0 left-3 flex items-center pointer-events-none"
    >
      <Search
        size={16}
        class="transition-colors duration-300 {isFocused
          ? 'text-cyan'
          : 'text-subtext'}"
      />
    </div>

    <input
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
      class="w-full bg-secondary/50 border border-white/5 rounded-full py-2.5 pl-10 pr-4 text-sm text-text
                   placeholder:text-subtext/50 outline-none transition-all duration-300
                   focus:bg-secondary focus:border-cyan/50"
    />

    {#if !isFocused && !searchQuery}
      <div
        class="absolute inset-y-0 right-4 flex items-center pointer-events-none"
      >
        <kbd
          class="hidden sm:inline-block px-1.5 py-0.5 text-[10px] font-mono text-subtext bg-white/5 rounded border border-white/10"
        >
          Ctrl K
        </kbd>
      </div>
    {/if}

    {#if isFocused && searchQuery && (searchResults.artists.length > 0 || searchResults.albums.length > 0 || searchResults.tracks.length > 0)}
      <div
        class="absolute top-full left-0 right-0 mt-2 bg-secondary/95 backdrop-blur-xl border border-white/5 rounded-xl shadow-2xl overflow-hidden z-50 max-h-[70vh] overflow-y-auto flex flex-col"
        onmousedown={(e) => e.preventDefault()}
        role="listbox"
        tabindex="-1"
      >
        {#if searchResults.artists.length > 0}
          <div class="p-2">
            <h3
              class="text-xs font-medium text-subtext uppercase px-2 py-1 mb-1"
            >
              Artists
            </h3>
            {#each searchResults.artists as artist}
              <button
                onclick={() => console.log("Navigate to artist", artist.id)}
                class="w-full flex items-center gap-3 p-2 hover:bg-white/5 rounded-lg transition-colors text-left group/item"
              >
                <div
                  class="w-10 h-10 rounded-full overflow-hidden bg-black/20 shrink-0"
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
                      <User size={20} />
                    </div>
                  {/if}
                </div>
                <span
                  class="font-medium text-text group-hover/item:text-cyan transition-colors"
                  >{artist.name}</span
                >
              </button>
            {/each}
          </div>
        {/if}

        {#if searchResults.albums.length > 0}
          <div class="p-2 border-t border-white/5">
            <h3
              class="text-xs font-medium text-subtext uppercase px-2 py-1 mb-1"
            >
              Albums
            </h3>
            {#each searchResults.albums as album}
              <button
                onclick={() => console.log("Navigate to album", album.id)}
                class="w-full flex items-center gap-3 p-2 hover:bg-white/5 rounded-lg transition-colors text-left group/item"
              >
                <div
                  class="w-10 h-10 rounded-md overflow-hidden bg-black/20 shrink-0"
                >
                  <Image
                    src={album.coverArt}
                    type="cover"
                    alt={album.title}
                    class="w-full h-full object-cover"
                  />
                </div>
                <div class="flex flex-col overflow-hidden">
                  <span
                    class="font-medium text-text truncate group-hover/item:text-cyan transition-colors"
                    >{album.title}</span
                  >
                  <span class="text-xs text-subtext truncate"
                    >{album.artistName}</span
                  >
                </div>
              </button>
            {/each}
          </div>
        {/if}

        {#if searchResults.tracks.length > 0}
          <div class="p-2 border-t border-white/5">
            <h3
              class="text-xs font-medium text-subtext uppercase px-2 py-1 mb-1"
            >
              Tracks
            </h3>
            {#each searchResults.tracks as track}
              <button
                onclick={() => player.playTrack.trigger(track.id)}
                class="w-full flex items-center gap-3 p-2 hover:bg-white/5 rounded-lg transition-colors text-left group/item"
              >
                <div
                  class="w-10 h-10 rounded-md overflow-hidden bg-black/20 shrink-0 relative group-hover/item:bg-black/40 transition-colors"
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
                <div class="flex flex-col overflow-hidden flex-1">
                  <span
                    class="font-medium text-text truncate group-hover/item:text-cyan transition-colors"
                    >{track.title}</span
                  >
                  <span class="text-xs text-subtext truncate"
                    >{track.artistName} • {track.albumTitle}</span
                  >
                </div>
                <span class="text-xs text-subtext font-mono"
                  >{formatDuration(track.durationSec)}</span
                >
              </button>
            {/each}
          </div>
        {/if}
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-4 shrink-0">
    <div class="h-6 w-px bg-white/10 mx-1"></div>

    <div class="flex items-center gap-1">
      <button
        onclick={() => handleWindowAction("minimize")}
        class="p-2 text-subtext hover:text-white hover:bg-white/5 rounded-lg transition-colors"
      >
        <Minus size={18} />
      </button>

      <button
        onclick={() => handleWindowAction("maximize")}
        class="p-2 text-subtext hover:text-white hover:bg-white/5 rounded-lg transition-colors"
      >
        <Square size={16} />
      </button>

      <button
        onclick={() => handleWindowAction("close")}
        class="p-2 text-subtext hover:bg-red hover:text-white rounded-lg transition-colors"
      >
        <X size={18} />
      </button>
    </div>
  </div>
</header>
