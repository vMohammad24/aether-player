<script lang="ts">
  import type { Snippet } from "svelte";
  import { backOut, quintOut } from "svelte/easing";
  import { fade, scale } from "svelte/transition";
  import { twMerge } from "tailwind-merge";
  import { lockScroll, trapFocus } from "../hooks";
  import Portal from "./Portal.svelte";

  interface Props {
    show: boolean;
    maxWidth?: string;
    onClose?: () => void;
    children: Snippet;
    class?: string;
  }

  let {
    show = $bindable(),
    maxWidth = "max-w-lg",
    onClose,
    children,
    class: className = "",
  }: Props = $props();

  function close() {
    show = false;
    onClose?.();
  }
</script>

{#if show}
  <Portal>
    <div
      class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:p-6"
      role="presentation"
      use:lockScroll
    >
      <button
        type="button"
        class="fixed inset-0 bg-background/80 backdrop-blur-sm cursor-default transition-opacity"
        onclick={close}
        aria-label="Close modal"
        transition:fade={{ duration: 200, easing: quintOut }}
      ></button>

      <div
        class={twMerge(
          "relative w-full max-h-[90vh] overflow-y-auto bg-secondary text-text rounded-xl shadow-2xl border border-accent focus:outline-none",
          maxWidth,
          className
        )}
        role="dialog"
        aria-modal="true"
        use:trapFocus={{ onClose: close }}
        transition:scale={{ duration: 250, start: 0.95, easing: backOut }}
      >
        {@render children()}
      </div>
    </div>
  </Portal>
{/if}
