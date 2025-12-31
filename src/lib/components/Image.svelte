<script lang="ts">
  import { ImageIcon } from "@lucide/svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";
  import { appLocalDataDir, join } from "@tauri-apps/api/path";

  let {
    src,
    alt = "",
    type,
    class: className = "",
  }: {
    src?: string | null;
    alt?: string;
    class?: string;
    type?: "cover";
  } = $props();
  let finalSrc: string | null = $state("");
  const appDataDirPath = await appLocalDataDir();

  $effect(() => {
    if (!src) return;
    if (src.startsWith("http")) {
      finalSrc = src;
      return;
    }

    let active = true;

    async function resolvePath() {
      if (!src) return;
      const path = await join(
        appDataDirPath,
        type === "cover" ? "covers" : "",
        src
      );
      if (active) {
        finalSrc = convertFileSrc(path);
      }
    }

    resolvePath();

    return () => {
      active = false;
    };
  });
</script>

{#if finalSrc}
  <img
    {alt}
    class={className}
    src={finalSrc}
    onerror={() => (finalSrc = null)}
  />
{:else}
  <ImageIcon class={className} />
{/if}
