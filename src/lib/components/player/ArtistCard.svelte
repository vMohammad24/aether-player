<script lang="ts">
  import { goto } from "$app/navigation";
  import type { Artist } from "$lib/bindings";
  import Image from "../Image.svelte";

  let { artist }: { artist: Artist } = $props();

  function handleNavigate() {
    goto(`/library/artists/${artist.id}`);
  }
</script>

<div
  role="button"
  tabindex="0"
  onclick={handleNavigate}
  onkeydown={(e) => e.key === 'Enter' && handleNavigate()}
  class="group flex flex-col items-center text-center gap-3 p-4 rounded-lg hover:bg-secondary/50 transition-colors cursor-pointer border border-transparent hover:border-border"
>
  <div class="relative w-full aspect-square rounded-full overflow-hidden bg-secondary border border-border shadow-sm">
    {#if artist.imageUrl}
      <Image
        src={artist.imageUrl}
        alt={artist.name}
        type="cover"
        class="w-full h-full object-cover transition-opacity duration-300"
      />
    {:else}
      <div class="w-full h-full flex items-center justify-center bg-secondary text-2xl font-bold text-subtext">
        {artist.name.slice(0, 2).toUpperCase()}
      </div>
    {/if}
  </div>

  <div class="flex flex-col gap-0.5 w-full">
    <span class="text-text font-medium truncate w-full leading-tight" title={artist.name}>{artist.name}</span>
    <span class="text-subtext text-xs uppercase tracking-wider font-semibold">Artist</span>
  </div>
</div>