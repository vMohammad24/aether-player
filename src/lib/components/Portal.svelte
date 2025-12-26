<script lang="ts">
  import { tick } from "svelte";

  interface Props {
    target?: string | HTMLElement;
    children?: import("svelte").Snippet;
  }

  let { target = "body", children }: Props = $props();

  function portal(node: HTMLElement, _target: string | HTMLElement) {
    let targetElement: HTMLElement | null = null;

    async function update(newTarget: string | HTMLElement) {
      await tick();

      if (typeof newTarget === "string") {
        targetElement = document.querySelector(newTarget);
      } else if (newTarget instanceof HTMLElement) {
        targetElement = newTarget;
      } else {
        console.warn(
          `[Portal] Target "${newTarget}" is not a valid HTMLElement or selector.`
        );
        return;
      }

      if (targetElement) {
        targetElement.appendChild(node);
        node.hidden = false;
      }
    }
    update(_target);

    return {
      update,
      destroy() {
        if (node.parentNode) {
          node.parentNode.removeChild(node);
        }
      },
    };
  }
</script>

<div use:portal={target} hidden style="display: contents">
  {@render children?.()}
</div>
