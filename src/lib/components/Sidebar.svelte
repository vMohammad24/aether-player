<script lang="ts">
  import { dev } from "$app/environment";
  import { page } from "$app/state";
  import {
    ChevronRight,
    CirclePlay,
    Database,
    FileText,
    Folder,
    Headphones,
    House,
    Menu,
    Mic,
    Palette,
    Settings,
    X,
  } from "@lucide/svelte";
  import { onMount } from "svelte";
  import { fade, slide } from "svelte/transition";
  import Button from "./Button.svelte";

  let isOpen = $state(true);
  let isMobile = $state(false);
  let expandedSections = $state<Record<string, boolean>>({
    playlists: true,
    settings: true,
  });

  let config = $state(
    [
      {
        id: "home",
        label: "Home",
        href: "/",
        icon: House,
        type: "single",
        disabled: false,
      },
      {
        id: "explore",
        label: "Explore",
        href: "/explore",
        icon: FileText,
        type: "single",
      },
      {
        id: "feed",
        label: "Feed",
        href: "/feed",
        icon: Mic,
        type: "single",
      },
      {
        id: "library",
        label: "Your Library",
        icon: Folder,
        type: "section",
        children: [
          { label: "Liked Songs", href: "/library/liked", icon: Headphones },
          { label: "Mix 1", href: "/playlist/1", icon: CirclePlay },
          { label: "Mix 2", href: "/playlist/2", icon: CirclePlay },
          { label: "Mix 3", href: "/playlist/3", icon: CirclePlay },
        ],
      },
      {
        id: "settings",
        label: "Settings",
        icon: Settings,
        type: "section",
        children: [
          { label: "General", href: "/settings/general", icon: Settings },
          { label: "Sources", href: "/settings/sources", icon: Database },
          { label: "Appearance", href: "/settings/appearance", icon: Palette },
        ],
      },
      dev && {
        id: "dev-playground",
        label: "Playground",
        href: "/playground",
        icon: FileText,
        type: "single",
      },
    ].filter((a) => typeof a !== "boolean")
  );

  function toggleSection(sectionId: string) {
    if (!isOpen) {
      isOpen = true;
      expandedSections[sectionId] = true;
    } else {
      expandedSections[sectionId] = !expandedSections[sectionId];
    }
  }

  function handleNavClick() {
    if (isMobile) {
      isOpen = false;
    }
  }

  onMount(() => {
    const checkMobile = () => {
      const wasMobile = isMobile;
      isMobile = window.innerWidth < 768;
      if (!isMobile && wasMobile) {
        isOpen = true;
      } else if (isMobile && !wasMobile) {
        isOpen = false;
      }
    };

    checkMobile();
    let touchStartX: number;
    let touchStartY: number;
    const swipeThreshold = 50;

    function handleTouchStart(e: TouchEvent) {
      if (isMobile) {
        touchStartX = e.touches[0].clientX;
        touchStartY = e.touches[0].clientY;
      }
    }

    function handleTouchEnd(e: TouchEvent) {
      if (!isMobile) return;
      const touchEndX = e.changedTouches[0].clientX;
      const touchEndY = e.changedTouches[0].clientY;
      const hDist = touchEndX - touchStartX;
      const vDist = Math.abs(touchEndY - touchStartY);

      if (vDist < 100) {
        if (hDist > swipeThreshold && touchStartX < 50 && !isOpen)
          isOpen = true;
        else if (hDist < -swipeThreshold && isOpen) isOpen = false;
      }
    }

    document.addEventListener("touchstart", handleTouchStart, {
      passive: true,
    });
    document.addEventListener("touchend", handleTouchEnd, { passive: true });

    return () => {
      window.removeEventListener("resize", checkMobile);
      document.removeEventListener("touchstart", handleTouchStart);
      document.removeEventListener("touchend", handleTouchEnd);
    };
  });
</script>

{#if isMobile && !isOpen}
  <header
    class="fixed top-0 left-0 right-0 z-30 flex items-center justify-between p-4
            bg-background/80 backdrop-blur-xl border-b border-white/5"
    transition:fade={{ duration: 200 }}
  >
    <Button
      type="button"
      variant="ghost"
      onclick={() => (isOpen = true)}
      class="p-2 hover:bg-white/10 rounded-xl transition-colors"
    >
      <Menu size={24} class="text-text" />
    </Button>
    <span class="text-lg font-bold tracking-tight text-text"> Aether </span>
    <div class="w-8"></div>
  </header>
{/if}

{#if isMobile && isOpen}
  <button
    type="button"
    class="fixed inset-0 bg-black/60 z-40 backdrop-blur-sm transition-opacity"
    onclick={() => (isOpen = false)}
    transition:fade={{ duration: 200 }}
    aria-label="Close sidebar"
  ></button>
{/if}

<aside
  class="pt-4 flex flex-col
        {isMobile ? 'fixed top-0 h-dvh z-50' : 'relative h-full z-30 shrink-0'} 
        {isOpen ? 'w-64' : 'w-20'} 
        {isMobile && !isOpen ? '-translate-x-full' : 'translate-x-0'}
        transition-all duration-500 ease-[cubic-bezier(0.34,1.56,0.64,1)]
        bg-background/95 backdrop-blur-2xl border-r border-white/5"
>
  <div class="flex flex-col h-full relative z-10">
    <div
      class="mb-8 flex items-center {isOpen
        ? 'justify-between px-6'
        : 'justify-center'}"
    >
      {#if isOpen}
        <a
          href="/"
          class="text-2xl font-bold tracking-tight text-text truncate"
        >
          Aether
        </a>
      {/if}

      <Button
        type="button"
        variant="ghost"
        onclick={() => (isOpen = !isOpen)}
        class="p-2 hover:bg-white/5 rounded-lg text-subtext hover:text-white transition-colors
                    {isOpen ? '' : 'mx-auto'}"
      >
        {#if isMobile}
          <X size={20} />
        {:else}
          <Menu
            size={20}
            class="transition-transform duration-500 {isOpen
              ? 'rotate-180'
              : ''}"
          />
        {/if}
      </Button>
    </div>

    <nav class="flex flex-col gap-2 px-3 grow overflow-y-auto scrollbar-hide">
      {#each config as item}
        <div>
          {#if item.type === "single"}
            <a
              href={item.disabled ? undefined : item.href}
              onclick={handleNavClick}
              class="group flex items-center gap-4 rounded-xl relative
                                {isOpen ? 'px-4 py-3' : 'p-3 justify-center'}
                                transition-all duration-300
                                {item.disabled
                ? 'opacity-50 cursor-not-allowed'
                : 'hover:bg-white/5'}
                                {page.url.pathname === item.href
                ? 'bg-cyan/10 text-cyan'
                : 'text-subtext hover:text-text'}"
            >
              <item.icon
                size={22}
                class="transition-transform duration-300 {page.url.pathname ===
                item.href
                  ? 'text-cyan'
                  : 'text-subtext group-hover:text-white'} 
                                    {!isOpen && !item.disabled
                  ? 'group-hover:scale-110'
                  : ''}"
              />

              {#if isOpen}
                <span class="font-medium text-sm tracking-wide"
                  >{item.label}</span
                >
              {/if}

              {#if page.url.pathname === item.href}
                <div
                  class="absolute left-0 top-1/2 -translate-y-1/2 h-6 w-1 rounded-r-full bg-cyan/80"
                ></div>
              {/if}
            </a>
          {:else if item.type === "section"}
            <button
              onclick={() => toggleSection(item.id)}
              class="w-full flex items-center gap-4 rounded-xl
                                {isOpen
                ? 'px-4 py-3 justify-between'
                : 'p-3 justify-center'}
                                text-subtext hover:text-text hover:bg-white/5 transition-all duration-300 group"
            >
              <div class="flex items-center gap-4">
                <item.icon
                  size={22}
                  class="group-hover:text-white transition-colors"
                />
                {#if isOpen}
                  <span class="font-medium text-sm tracking-wide"
                    >{item.label}</span
                  >
                {/if}
              </div>
              {#if isOpen}
                <ChevronRight
                  size={16}
                  class="transition-transform duration-300 {expandedSections[
                    item.id
                  ]
                    ? 'rotate-90'
                    : ''}"
                />
              {/if}
            </button>

            {#if expandedSections[item.id] && item.children && isOpen}
              <div
                class="mt-1 space-y-0.5 ml-4 border-l border-white/10"
                transition:slide={{ duration: 200 }}
              >
                {#each item.children as child}
                  <a
                    href={child.href}
                    class="flex items-center gap-3 px-4 py-2.5 text-sm text-subtext hover:text-white hover:bg-white/5 rounded-r-lg transition-all
                                            {page.url.pathname === child.href
                      ? 'text-cyan bg-cyan/5'
                      : ''}"
                  >
                    <child.icon size={16} class="opacity-70" />
                    <span class="truncate">{child.label}</span>
                  </a>
                {/each}
              </div>
            {/if}
          {/if}
        </div>
      {/each}
    </nav>
  </div>
</aside>
