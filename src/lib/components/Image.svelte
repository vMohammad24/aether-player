<script module lang="ts">
  import { appLocalDataDir, join, sep } from "@tauri-apps/api/path";

  let pathConfigPromise: Promise<{
    coversDir: string;
    appDataDir: string;
    separator: string;
  }> | null = null;

  function getPathConfig() {
    if (!pathConfigPromise) {
      pathConfigPromise = (async () => {
        const separator = sep();
        const appDataDir = await appLocalDataDir();
        const coversDir = await join(appDataDir, "covers");

        return { coversDir, appDataDir, separator };
      })();
    }
    return pathConfigPromise;
  }
</script>

<script lang="ts">
  import { ImageIcon } from "@lucide/svelte";
  import { convertFileSrc } from "@tauri-apps/api/core";

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

  let finalSrc: string | null = $state(null);

  $effect(() => {
    if (!src) {
      finalSrc = null;
      return;
    }

    if (src.startsWith("http")) {
      finalSrc = src;
      return;
    }

    let active = true;

    getPathConfig().then(({ coversDir, appDataDir, separator }) => {
      if (!active) return;

      const basePath = type === "cover" ? coversDir : appDataDir;

      const cleanSrc = src!.startsWith(separator) ? src!.slice(1) : src;
      const fullPath = `${basePath}${separator}${cleanSrc}`;

      finalSrc = convertFileSrc(fullPath);
    });

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
