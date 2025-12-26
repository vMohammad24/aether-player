<script lang="ts">
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import { tooltip } from "$lib/hooks";
  import { createResource } from "$lib/stores/resource.svelte";
  import { toast } from "$lib/stores/toast.svelte";

  let showModal = $state(false);

  const colors = [
    { name: "background", cls: "bg-background" },
    { name: "secondary", cls: "bg-secondary" },
    { name: "primary", cls: "bg-primary" },
    { name: "accent", cls: "bg-accent" },
    { name: "border", cls: "bg-border" },
    { name: "red", cls: "bg-red" },
    { name: "green", cls: "bg-green" },
    { name: "purple", cls: "bg-purple" },
    { name: "pink", cls: "bg-pink" },
    { name: "teal", cls: "bg-teal" },
    { name: "mauve", cls: "bg-mauve" },
    { name: "orange", cls: "bg-orange" },
    { name: "yellow", cls: "bg-yellow" },
    { name: "cyan", cls: "bg-cyan" },
    { name: "blue", cls: "bg-blue" },
    { name: "gold", cls: "bg-gold" },
    { name: "text", cls: "bg-text" },
    { name: "gray", cls: "bg-gray" },
  ];

  const track = createResource("helloWorld", "From My window!");
</script>

<div
  class="min-h-screen bg-background text-text p-8 md:p-12 font-sans selection:bg-cyan selection:text-background"
>
  <div class="grid lg:grid-cols-[1fr_300px] gap-12">
    <div class="space-y-12">
      <section>
        <h2 class="text-sm font-bold uppercase tracking-wider text-gray mb-4">
          Color Palette
        </h2>
        <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-8 gap-4">
          {#each colors as color}
            <div class="group flex flex-col gap-2">
              <div
                class="relative w-full aspect-video rounded-md {color.cls} border border-white/5 shadow-lg group-hover:scale-[1.02] transition-transform duration-200"
              ></div>
              <span class="text-sm font-medium text-text">{color.name}</span>
            </div>
          {/each}
        </div>
      </section>

      <section>
        <h2 class="text-sm font-bold uppercase tracking-wider text-gray mb-4">
          Component Playground
        </h2>
        <div class="flex flex-col gap-8">
          <div class="space-y-4">
            <h3 class="text-lg font-semibold text-text">Buttons & Tooltips</h3>
            <div class="flex flex-wrap gap-4 items-center">
              <span use:tooltip={"Primary Button"}>
                <Button
                  variant="primary"
                  onclick={() => console.log("clicked")}
                >
                  Primary
                </Button>
              </span>
              <span
                use:tooltip={{
                  content: "Secondary Button",
                  placement: "bottom",
                }}
              >
                <Button variant="secondary">Secondary</Button>
              </span>
              <Button variant="outline">Outline</Button>
              <Button variant="ghost">Ghost</Button>
              <Button variant="destructive">Destructive</Button>
              <span use:tooltip={"I am disabled"}>
                <Button disabled>Disabled</Button>
              </span>
            </div>
          </div>

          <div class="space-y-4">
            <h3 class="text-lg font-semibold text-text">Toasts</h3>
            <div class="flex flex-wrap gap-4">
              <Button onclick={() => toast.info("This is an info toast")}>
                Info
              </Button>
              <Button
                variant="secondary"
                onclick={() => toast.success("Operation successful!")}
              >
                Success
              </Button>
              <Button
                variant="outline"
                onclick={() => toast.warning("Warning: Check your input")}
              >
                Warning
              </Button>
              <Button
                variant="destructive"
                onclick={() => toast.error("Something went wrong")}
              >
                Error
              </Button>
              <Button
                onclick={() =>
                  toast.promise(new Promise((r) => setTimeout(r, 2000)), {
                    loading: "Loading...",
                    success: "Done!",
                    error: "Failed",
                  })}
              >
                Promise
              </Button>
              <Button
                variant="primary"
                onclick={() =>
                  toast.info("File deleted", {
                    actions: [
                      {
                        label: "Undo",
                        onClick: () => toast.success("Undo successful"),
                      },
                    ],
                  })}
              >
                Action
              </Button>
            </div>
          </div>

          <div class="space-y-4">
            <h3 class="text-lg font-semibold text-text">Modals</h3>
            <Button onclick={() => (showModal = true)}>Open Modal</Button>
          </div>
        </div>
      </section>

      <section>
        <h2 class="text-sm font-bold uppercase tracking-wider text-gray mb-4">
          Resource Store Test
        </h2>
        <code
          class="bg-secondary text-text px-3 py-1 rounded-md text-sm font-mono"
        >
          {#if track.loading}
            Loading...
          {:else if track.error}
            Error: {track.error}
          {:else}
            {JSON.stringify(track.data, null, 2)}
          {/if}
        </code>
      </section>
    </div>
  </div>
  <Modal bind:show={showModal}>
    <div class="p-6">
      <h2 class="text-xl font-bold mb-4 text-text">Test Modal</h2>
      <p class="text-gray mb-6">
        This is a modal to test the Modal component functionality. You can close
        it by clicking the backdrop or the button below.
      </p>
      <div class="flex justify-end gap-3">
        <Button variant="ghost" onclick={() => (showModal = false)}>
          Cancel
        </Button>
        <Button variant="primary" onclick={() => (showModal = false)}>
          Confirm
        </Button>
      </div>
    </div>
  </Modal>
</div>
