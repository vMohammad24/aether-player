<script lang="ts">
  import { commands } from "$lib/bindings";
  import Button from "$lib/components/Button.svelte";
  import Image from "$lib/components/Image.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import { tooltip } from "$lib/hooks";
  import { createMutation, createResource } from "$lib/stores/resource.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  let showModal = $state(false);

  const recentAlbums = createResource("getRecentAlbums", "local", 6);
  const scanLibrary = createMutation("scanLibrary", {
    invalidate: ["getRecentAlbums"],
  });
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
                <Button variant="primary">Primary</Button>
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
            <h3 class="text-lg font-semibold text-text">Etc</h3>
            <Button onclick={() => (showModal = true)}>Open Modal</Button>
            <!-- <span
              use:contextMenu={[
                { label: "Context Menu" },
                { type: "separator" },
                { label: "Another Item", type: "danger" },
              ]}
            >
              <Button>Context Menu</Button>
            </span> -->
          </div>
        </div>
      </section>

      <section>
        <div class="flex items-center justify-between mb-4">
          <h2 class="text-sm font-bold uppercase tracking-wider text-gray">
            Recently Added (Resource Test)
          </h2>
          <Button variant="ghost" onclick={() => recentAlbums.refetch()}>
            {recentAlbums.isValidating ? "Refreshing..." : "Refresh"}
          </Button>
          <Button variant="ghost" onclick={() => scanLibrary.trigger("local")}>
            {scanLibrary.isPending ? "Scanning" : "Scan"}
          </Button>
        </div>

        {#if recentAlbums.loading}
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-4">
            {#each Array(6) as _}
              <div class="animate-pulse space-y-2">
                <div class="aspect-square bg-secondary rounded-md"></div>
                <div class="h-3 bg-secondary rounded w-3/4"></div>
                <div class="h-2 bg-secondary rounded w-1/2"></div>
              </div>
            {/each}
          </div>
        {:else if recentAlbums.error}
          <div
            class="p-4 rounded-md bg-red/10 border border-red/20 text-red text-sm"
          >
            Failed to load library: {recentAlbums.error}
          </div>
        {:else if recentAlbums.data && recentAlbums.data.length > 0}
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-6 gap-4">
            {#each recentAlbums.data as album}
              <div class="group flex flex-col gap-2 cursor-pointer">
                <div
                  class="relative aspect-square rounded-md bg-secondary border border-white/5 overflow-hidden shadow-lg group-hover:scale-[1.02] transition-transform duration-200"
                >
                  {#if album.coverArt}
                    <Image
                      src={album.coverArt}
                      alt={album.title}
                      type="cover"
                      class="w-full h-full object-cover"
                    />
                  {:else}
                    <div
                      class="w-full h-full flex items-center justify-center text-gray/20"
                    >
                      <svg
                        xmlns="http://www.w3.org/2000/svg"
                        width="32"
                        height="32"
                        viewBox="0 0 24 24"
                        fill="none"
                        stroke="currentColor"
                        stroke-width="1.5"
                        stroke-linecap="round"
                        stroke-linejoin="round"
                        ><path d="M9 18V5l12-2v13" /><circle
                          cx="6"
                          cy="18"
                          r="3"
                        /><circle cx="18" cy="16" r="3" /></svg
                      >
                    </div>
                  {/if}
                </div>
                <div class="flex flex-col min-w-0">
                  <span class="text-sm font-medium text-text truncate"
                    >{album.title}</span
                  >
                  <span class="text-xs text-gray truncate"
                    >{album.artistName}</span
                  >
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div
            class="text-center py-12 border-2 border-dashed border-border rounded-xl"
          >
            <p class="text-gray mb-4">No music found in your local library.</p>
            <Button
              variant="outline"
              onclick={async () =>
                commands.addLibraryRoot(
                  "local",
                  (await open({ directory: true })) || ""
                )}
            >
              Add Music Folder
            </Button>
          </div>
        {/if}
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
