<script lang="ts">
  import type { ContextMenuItem } from "$lib/hooks/contextMenu";
  import {
    autoUpdate,
    computePosition,
    flip,
    offset,
    shift,
    type Placement,
    type ReferenceElement,
  } from "@floating-ui/dom";

  import { Check, ChevronRight } from "@lucide/svelte";
  import { mount, unmount } from "svelte";
  import { fade } from "svelte/transition";
  import { twMerge } from "tailwind-merge";
  import ContextMenuPopup from "./ContextMenuPopup.svelte";

  let {
    items,
    anchor,
    onClose,
    placement = "bottom-start",
  }: {
    items: ContextMenuItem[];
    anchor: ReferenceElement;
    onClose: () => void;
    placement?: Placement;
  } = $props();

  let menuEl: HTMLElement | undefined = $state();
  let x = $state(0);
  let y = $state(0);

  let activeIndex = $state(-1);
  let activeSubMenu: {
    component: ReturnType<typeof mount>;
    unmount: () => void;
    index: number;
  } | null = $state(null);
  let hoverTimeout: number | undefined;

  $effect(() => {
    return () => {
      closeSubMenu();
      clearTimeout(hoverTimeout);
    };
  });

  $effect(() => {
    if (menuEl && anchor) {
      return autoUpdate(anchor, menuEl, () => {
        computePosition(anchor, menuEl!, {
          placement,
          strategy: "fixed",
          middleware: [
            offset(placement.includes("start") ? 4 : 10),
            flip(),
            shift({ padding: 10 }),
          ],
        }).then((pos) => {
          x = pos.x;
          y = pos.y;
        });
      });
    }
  });

  function closeSubMenu() {
    if (activeSubMenu) {
      activeSubMenu.unmount();
      activeSubMenu = null;
    }
  }

  function handleItemClick(
    item: ContextMenuItem,
    event: MouseEvent | KeyboardEvent
  ) {
    if (item.type === "separator" || item.type === "label" || item.disabled)
      return;

    if (item.type === "action" && item.children) {
      event.stopPropagation();
      return;
    }

    if (item.type === "checkbox" && item.onChange) {
      item.onChange(!item.checked);
    } else if (item.type === "radio" && item.onSelect) {
      item.onSelect();
    } else if ((!item.type || item.type === "action") && item.onClick) {
      item.onClick();
    }

    onClose();
  }

  function handleMouseEnter(
    index: number,
    item: ContextMenuItem,
    event: MouseEvent
  ) {
    clearTimeout(hoverTimeout);
    activeIndex = index;

    if (activeSubMenu && activeSubMenu.index === index) return;

    hoverTimeout = window.setTimeout(() => {
      closeSubMenu();
      if (
        (!item.type || item.type === "action") &&
        item.children &&
        !item.disabled &&
        event.target
      ) {
        openSubMenu(item.children, event.target as HTMLElement, index);
      }
    }, 150);
  }

  function openSubMenu(
    subItems: ContextMenuItem[],
    anchorEl: HTMLElement,
    index: number
  ) {
    const component = mount(ContextMenuPopup, {
      target: document.body,
      props: {
        items: subItems,
        anchor: anchorEl,
        onClose,
        placement: "right-start",
      },
    });
    activeSubMenu = { component, unmount: () => unmount(component), index };
  }

  function handleKeyDown(e: KeyboardEvent) {
    if (
      ![
        "ArrowUp",
        "ArrowDown",
        "ArrowLeft",
        "ArrowRight",
        "Enter",
        " ",
      ].includes(e.key)
    )
      return;
    e.preventDefault();

    const actionable = items.length;

    if (e.key === "ArrowDown")
      activeIndex = activeIndex === -1 ? 0 : (activeIndex + 1) % actionable;
    if (e.key === "ArrowUp")
      activeIndex =
        activeIndex === -1
          ? actionable - 1
          : (activeIndex - 1 + actionable) % actionable;
    if (e.key === "ArrowRight") {
      const item = items[activeIndex];
      if ((!item.type || item.type === "action") && item.children) {
        const btn = menuEl?.children[activeIndex] as HTMLElement;
        if (btn) openSubMenu(item.children, btn, activeIndex);
      }
    }
    if (e.key === "Enter" || e.key === " ") {
      const item = items[activeIndex];
      if (item) handleItemClick(item, e);
    }
  }
</script>

<div
  bind:this={menuEl}
  role="menu"
  tabindex="-1"
  oncontextmenu={(e) => {
    e.preventDefault();
    e.stopPropagation();
  }}
  onclick={(e) => e.stopPropagation()}
  onkeydown={handleKeyDown}
  class="fixed z-60 min-w-56 overflow-hidden rounded-lg border border-border bg-secondary shadow-2xl p-1 focus:outline-none text-text"
  style:left="{x}px"
  style:top="{y}px"
  transition:fade={{ duration: 75 }}
>
  {#each items as item, i}
    {#if item.type === "separator"}
      <div class="my-1 h-px bg-border mx-2" role="separator"></div>
    {:else if item.type === "label"}
      <div
        class="px-3 py-1.5 text-xs font-semibold text-subtext uppercase tracking-wider select-none"
      >
        {item.label}
      </div>
    {:else}
      {@const isDisabled = item.disabled}
      {@const isActive = activeIndex === i}
      {@const isSubMenuOpen = activeSubMenu?.index === i}
      {@const isDanger = item.type === "action" && item.variant === "danger"}

      <button
        type="button"
        role={item.type === "checkbox"
          ? "menuitemcheckbox"
          : item.type === "radio"
            ? "menuitemradio"
            : "menuitem"}
        disabled={isDisabled}
        aria-checked={item.type === "checkbox" || item.type === "radio"
          ? item.checked
          : undefined}
        onclick={(e) => handleItemClick(item, e)}
        onmouseenter={(e) => handleMouseEnter(i, item, e)}
        class={twMerge(
          "flex w-full items-center justify-between px-3 py-1.5 text-sm rounded transition-colors text-left outline-none cursor-default select-none relative group",
          isDisabled && "opacity-50 cursor-not-allowed",
          (isActive || isSubMenuOpen) && !isDisabled
            ? isDanger
              ? "bg-red/10 text-red"
              : "bg-accent text-text"
            : isDanger
              ? "text-red"
              : "text-text"
        )}
      >
        <div class="flex items-center gap-2.5">
          <div
            class={twMerge(
              "flex items-center justify-center w-4 h-4",
              isDanger ? "text-current" : "text-text"
            )}
          >
            {#if item.type === "checkbox" && item.checked}
              <Check class="size-3.5" strokeWidth={3} />
            {:else if item.type === "radio" && item.checked}
              <div class="size-2 bg-text rounded-full"></div>
            {:else if item.leftIcon}
              <item.leftIcon
                class={twMerge(
                  "size-4",
                  isDanger
                    ? "text-current opacity-70 group-hover:opacity-100"
                    : "text-subtext group-hover:text-text"
                )}
              />
            {/if}
          </div>

          <span>{item.label}</span>
        </div>

        <div class="flex items-center gap-3">
          {#if item.shortcut}
            <span class="text-xs text-subtext font-mono tracking-tighter"
              >{item.shortcut}</span
            >
          {/if}

          {#if (item.type === "action" || !item.type) && item.children}
            <ChevronRight class="size-4 text-subtext" />
          {/if}

          {#if item.rightIcon}
            <item.rightIcon class="size-4 text-subtext" />
          {/if}
        </div>
      </button>
    {/if}
  {/each}
</div>
