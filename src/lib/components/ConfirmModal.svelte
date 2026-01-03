<script lang="ts">
  import { browser } from "$app/environment";
  import { escapeHtml } from "$lib/util";
  import { X } from "@lucide/svelte";
  import Button from "./Button.svelte";
  import Modal from "./Modal.svelte";

  let {
    show = $bindable(false),
    title = "Confirm",
    message = "Are you sure?",
    confirmText = "Confirm",
    declineText = "Cancel",
    variant = "danger",
    onConfirm = () => {},
    onDecline = () => {},
    onClose = () => {},
  } = $props<{
    show?: boolean;
    title?: string;
    message?: string;
    confirmText?: string;
    declineText?: string;
    variant?: "danger" | "warning" | "info";
    onConfirm?: () => void;
    onDecline?: () => void;
    onClose?: () => void;
  }>();

  function handleConfirm() {
    show = false;
    onConfirm();
  }

  function handleDecline() {
    show = false;
    onDecline();
  }

  function handleClose() {
    show = false;
    onClose();
  }

  function parseMessage(text: string) {
    const escaped = browser && text ? escapeHtml(text) : "";
    return escaped.replace(
      /\*\*(.*?)\*\*/g,
      '<strong class="text-text font-bold">$1</strong>'
    );
  }
</script>

<Modal bind:show maxWidth="max-w-md" onClose={handleClose}>
  <div class="bg-primary rounded-xl p-6">
    <div class="flex items-center justify-between mb-4">
      <h3 class="text-lg font-semibold text-text">
        {title}
      </h3>
      <Button
        variant="ghost"
        size="sm"
        onclick={handleClose}
        leftIcon={X}
        aria-label="Close"
        class="text-subtext-400 hover:text-text p-2"
      />
    </div>

    <div class="mb-6">
      <p class="text-subtext-400">
        {@html parseMessage(message)}
      </p>
    </div>

    <div class="flex gap-3 justify-end">
      <Button variant="secondary" onclick={handleDecline}>
        {declineText}
      </Button>
      <Button
        variant={variant === "danger" ? "destructive" : "primary"}
        onclick={handleConfirm}
      >
        {confirmText}
      </Button>
    </div>
  </div>
</Modal>
