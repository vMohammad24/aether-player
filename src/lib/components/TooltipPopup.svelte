<script lang="ts">
  import { fade } from "svelte/transition";
  import { twMerge } from "tailwind-merge";

  let { content, class: className }: { content: string; class?: string } =
    $props();
</script>

<div
  id="tooltip-portal-root"
  class={twMerge(
    "absolute top-0 left-0 z-50 w-max max-w-xs px-2 py-1 text-xs font-medium text-text bg-gray rounded shadow-lg pointer-events-none",
    className
  )}
  transition:fade={{ duration: 100 }}
>
  {content}
  <div id="tooltip-arrow" class="absolute w-2 h-2 bg-gray rotate-45"></div>
</div>
