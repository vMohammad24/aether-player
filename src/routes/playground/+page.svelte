<script lang="ts">
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import { contextMenu, tooltip } from "$lib/hooks";
  import { media } from "$lib/stores/player/media.svelte";
  import { player } from "$lib/stores/player/player.svelte";
  import { queue } from "$lib/stores/player/queue.svelte";
  import { baseColors } from "$lib/stores/theme.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { LogOut, Settings, User } from "@lucide/svelte";
  let showModal = $state(false);
  let placeholderCheck = $state(true);
  let placeHolderTheme = $state("dark");
  const colors = baseColors.map((color) => ({
    name: color,
    cls: `bg-${color}`,
  }));
</script>

<div class="grid lg:grid-cols-[1fr_300px] gap-12">
  <div class="space-y-12">
    <section>
      <h2 class="text-sm font-bold uppercase tracking-wider text-subtext mb-4">
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
      <h2 class="text-sm font-bold uppercase tracking-wider text-subtext mb-4">
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
          <span
            use:contextMenu={[
              { type: "label", label: "View Options" },
              {
                type: "checkbox",
                label: "Check BOX",
                checked: placeholderCheck,
                onChange: (v) => (placeholderCheck = v),
                shortcut: "Ctrl+B",
              },
              { type: "separator" },
              { type: "label", label: "Theme" },
              {
                type: "radio",
                label: "Light",
                checked: placeHolderTheme === "light",
                onSelect: () => (placeHolderTheme = "light"),
              },
              {
                type: "radio",
                label: "Dark",
                checked: placeHolderTheme === "dark",
                onSelect: () => (placeHolderTheme = "dark"),
              },
              { type: "separator" },
              {
                label: "Account",
                leftIcon: User,
                children: [
                  { label: "Profile", leftIcon: Settings },
                  { type: "separator" },
                  { label: "Log out", leftIcon: LogOut, variant: "danger" },
                ],
              },
            ]}
          >
            <Button>Context Menu</Button>
          </span>
        </div>
      </div>
    </section>

    <section>
      <h2 class="text-sm font-bold uppercase tracking-wider text-subtext mb-4">
        Store Playground
      </h2>
      <div class="flex flex-col gap-8">
        <div class="space-y-4">
          <h3 class="text-lg font-semibold text-text">Player Store</h3>
          <div
            class="bg-secondary p-4 rounded-md font-mono text-xs overflow-auto max-h-40"
          >
            <pre>{JSON.stringify(player.state, null, 2)}</pre>
          </div>
          <div class="flex flex-wrap gap-4">
            <Button onclick={() => player.play.trigger()}>Play</Button>
            <Button onclick={() => player.pause.trigger()}>Pause</Button>
            <Button onclick={() => player.next.trigger()}>Next</Button>
            <Button onclick={() => player.prev.trigger()}>Prev</Button>
            <Button onclick={() => player.toggleShuffle.trigger()}
              >Shuffle</Button
            >
          </div>
        </div>

        <div class="space-y-4">
          <h3 class="text-lg font-semibold text-text">Queue Store</h3>
          <div
            class="bg-secondary p-4 rounded-md font-mono text-xs overflow-auto max-h-40"
          >
            <pre>{JSON.stringify(queue.data, null, 2)}</pre>
          </div>
          <div class="flex flex-wrap gap-4">
            <Button onclick={() => queue.clear.trigger()}>Clear Queue</Button>
          </div>
        </div>

        <div class="space-y-4">
          <h3 class="text-lg font-semibold text-text">Media Store</h3>
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div>
              <h4 class="font-medium mb-2 text-sm text-subtext">
                Recent Albums
              </h4>
              <div
                class="bg-secondary p-4 rounded-md font-mono text-xs overflow-auto max-h-40"
              >
                <pre>{JSON.stringify(media.recentAlbums(5).data, null, 2)}</pre>
              </div>
            </div>
            <div>
              <h4 class="font-medium mb-2 text-sm text-subtext">Favorites</h4>
              <div
                class="bg-secondary p-4 rounded-md font-mono text-xs overflow-auto max-h-40"
              >
                <pre>{JSON.stringify(media.favorites.data, null, 2)}</pre>
              </div>
            </div>
          </div>
          <div class="flex flex-wrap gap-4">
            <Button onclick={() => media.scanLibraries.trigger()}
              >Scan Libraries</Button
            >
          </div>
        </div>
      </div>
    </section>
  </div>
</div>
<Modal bind:show={showModal}>
  <div class="p-6">
    <h2 class="text-xl font-bold mb-4 text-text">Test Modal</h2>
    <p class="text-subtext mb-6">
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
