<script lang="ts">
  import {
    CircleCheck,
    CircleX,
    Info,
    LoaderCircle,
    TriangleAlert,
    X,
  } from "@lucide/svelte";
  import { flip } from "svelte/animate";
  import { backOut } from "svelte/easing";
  import { fly } from "svelte/transition";

  import Button from "$lib/components/Button.svelte";
  import Portal from "$lib/components/Portal.svelte";
  import { toastState, type ToastPosition } from "$lib/stores/toast.svelte";

  interface Props {
    position?: ToastPosition;
    expand?: boolean;
  }

  let { position = "bottom-right" }: Props = $props();

  const positionClasses: Record<ToastPosition, string> = {
    "top-left": "top-0 left-0 items-start flex-col-reverse",
    "top-center":
      "top-0 left-1/2 -translate-x-1/2 items-center flex-col-reverse",
    "top-right": "top-0 right-0 items-end flex-col-reverse",
    "bottom-left": "bottom-0 left-0 items-start flex-col",
    "bottom-center": "bottom-0 left-1/2 -translate-x-1/2 items-center flex-col",
    "bottom-right": "bottom-0 right-0 items-end flex-col",
  };

  const iconMap = {
    info: Info,
    success: CircleCheck,
    warning: TriangleAlert,
    error: CircleX,
    loading: LoaderCircle,
  };

  const colorClasses = {
    info: "text-blue",
    success: "text-green",
    warning: "text-yellow",
    error: "text-red",
    loading: "text-blue",
  };
</script>

<Portal>
  <div
    class="fixed z-9999 flex p-4 gap-3 w-full max-w-sm pointer-events-none transition-all duration-300 {positionClasses[
      position
    ]}"
    aria-live="assertive"
  >
    {#each toastState.toasts as t (t.id)}
      <div
        role="status"
        animate:flip={{ duration: 400, easing: backOut }}
        transition:fly={{
          y: position.includes("top") ? -20 : 20,
          x: position.includes("center")
            ? 0
            : position.includes("right")
              ? 20
              : -20,
          duration: 300,
          easing: backOut,
        }}
        onmouseenter={() => toastState.pause(t.id)}
        onmouseleave={() => toastState.resume(t.id)}
        class="
                    pointer-events-auto w-full
                    bg-primary
                    border border-border
                    shadow-lg rounded-lg p-4 flex flex-col gap-3
                    transform transition-all duration-200
                "
      >
        <div class="flex items-start gap-3">
          <div class="shrink-0 mt-0.5">
            {#if t.icon}
              <t.icon class="w-5 h-5 text-text" />
            {:else}
              {@const Icon = iconMap[t.type]}
              <Icon
                class="w-5 h-5 {colorClasses[t.type]} {t.type === 'loading'
                  ? 'animate-spin'
                  : ''}"
              />
            {/if}
          </div>

          <div class="flex-1 min-w-0">
            {#if t.title}
              <h3 class="text-sm font-semibold text-white">
                {t.title}
              </h3>
            {/if}
            <p class="text-sm text-text leading-relaxed font-medium">
              {t.message}
            </p>
          </div>

          {#if t.dismissible !== false}
            <button
              onclick={() => toastState.remove(t.id)}
              class="
                                text-gray
                                hover:text-white
                                transition-colors -mt-1 -mr-1 rounded-md p-1
                                hover:bg-secondary
                                focus:outline-none focus:ring-2 focus:ring-border
                            "
              aria-label="Close notification"
            >
              <X class="w-4 h-4" />
            </button>
          {/if}
        </div>

        {#if t.actions && t.actions.length > 0}
          <div
            class="flex flex-wrap gap-2 mt-1 justify-end border-t border-border pt-3"
          >
            {#each t.actions as action}
              <Button
                variant={action.variant || "outline"}
                size="sm"
                onclick={() => action.onClick()}
              >
                {action.label}
              </Button>
            {/each}
          </div>
        {/if}
      </div>
    {/each}
  </div>
</Portal>
