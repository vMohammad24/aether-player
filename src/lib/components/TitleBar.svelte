<script lang="ts">
  import { afterNavigate } from "$app/navigation";
  import {
    ChevronLeft,
    ChevronRight,
    Minus,
    Search,
    Square,
    X,
  } from "@lucide/svelte";
  import { getCurrentWindow } from "@tauri-apps/api/window";

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

  function handleNavigationAction(action: "back" | "forward") {
    if (action === "back") canGoBack && history.back();
    else canGoForward && history.forward();
  }
</script>

<header
  class="w-full px-6 py-4 flex items-center justify-between gap-4
           bg-background/80 backdrop-blur-xl border-b border-white/5 select-none"
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
          : 'text-gray'}"
      />
    </div>

    <input
      type="text"
      placeholder="Search artists or tracks"
      bind:value={searchQuery}
      onfocus={() => (isFocused = true)}
      onblur={() => (isFocused = false)}
      class="w-full bg-secondary/50 border border-white/5 rounded-full py-2.5 pl-10 pr-4 text-sm text-text
                   placeholder:text-gray/50 outline-none transition-all duration-300
                   focus:bg-secondary focus:border-cyan/50"
    />

    {#if !isFocused && !searchQuery}
      <div
        class="absolute inset-y-0 right-4 flex items-center pointer-events-none"
      >
        <kbd
          class="hidden sm:inline-block px-1.5 py-0.5 text-[10px] font-mono text-gray bg-white/5 rounded border border-white/10"
        >
          Ctrl K
        </kbd>
      </div>
    {/if}
  </div>

  <div class="flex items-center gap-4 shrink-0">
    <div class="h-6 w-px bg-white/10 mx-1"></div>

    <div class="flex items-center gap-1">
      <button
        onclick={() => handleWindowAction("minimize")}
        class="p-2 text-gray hover:text-white hover:bg-white/5 rounded-lg transition-colors"
      >
        <Minus size={18} />
      </button>

      <button
        onclick={() => handleWindowAction("maximize")}
        class="p-2 text-gray hover:text-white hover:bg-white/5 rounded-lg transition-colors"
      >
        <Square size={16} />
      </button>

      <button
        onclick={() => handleWindowAction("close")}
        class="p-2 text-gray hover:bg-red hover:text-white rounded-lg transition-colors"
      >
        <X size={18} />
      </button>
    </div>
  </div>
</header>
