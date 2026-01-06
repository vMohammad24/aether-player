<script lang="ts">
  import { commands, type SourceConfig } from "$lib/bindings";
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import { config } from "$lib/stores/config.svelte";
  import { confirm } from "$lib/stores/confirm.svelte";
  import { createMutation } from "$lib/stores/resource.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { Folder, Globe, Music2, RefreshCcw, Trash } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";

  let showAddSourceModal = $state(false);
  let newSourceType = $state<"local" | "subsonic">("local");
  let localPath = $state("");
  let localName = $state("My Music");

  const KEYS_TO_INVALIDATE: (keyof typeof commands)[] = [
    "getRecentAlbums",
    "getFavorites",
    "getPlaylists",
    "getArtist",
    "getAlbum",
    "getArtistAlbums",
    "getAlbumTracks",
    "search",
    "getLibraryStats",
  ];

  const addSource = createMutation("addSource", {
    onError: (e) => toast.error(`Failed to add source: ${e}`),
    onSuccess: () => {
      config.forceSync();
      showAddSourceModal = false;
      resetSourceForm();
      toast.success("Source added");
    },
    invalidate: KEYS_TO_INVALIDATE,
  });

  const deleteSource = createMutation("deleteSource", {
    onError: (e) => toast.error(`Failed to remove source: ${e}`),
    onSuccess: () => {
      config.forceSync();
      toast.success("Source removed");
    },
    invalidate: KEYS_TO_INVALIDATE,
  });

  const toggleSource = createMutation("toggleSource", {
    onError: (e) => toast.error(`Failed to toggle source: ${e}`),
    onSuccess: () => {
      config.forceSync();
    },
    invalidate: KEYS_TO_INVALIDATE,
  });

  const rescanSource = createMutation("scanLibrary");

  function generateId() {
    return Math.random().toString(36).substring(2, 15);
  }

  async function handleAddSource() {
    if (newSourceType === "local") {
      if (!localPath) return toast.error("Path is required");

      const newSource: SourceConfig = {
        type: "local",
        id: generateId(),
        name: localName,
        path: localPath,
        enabled: true,
      };

      const res = await addSource.trigger(newSource);
    } else {
      toast.error("Subsonic not fully implemented in UI yet");
    }
  }

  async function handleDeleteSource(id: string) {
    if (await confirm("Are you sure you want to remove this source?")) {
      const res = await deleteSource.trigger(id);
      if (res) {
        await config.forceSync();
        toast.success("Source removed");
      }
    }
  }

  async function handleToggleSource(id: string, enabled: boolean) {
    await toggleSource.trigger(id, enabled);
  }

  function resetSourceForm() {
    localPath = "";
    localName = "My Music";
  }
</script>

<div class="flex items-center justify-between border-b border-border pb-3">
  <h2 class="text-xl font-semibold text-text">Library Sources</h2>
  <Button onclick={() => (showAddSourceModal = true)} size="sm"
    >Add Source</Button
  >
</div>

<div class="grid gap-3">
  {#each config.sources as source (source.id)}
    <div
      class="flex items-center justify-between p-4 bg-secondary rounded-xl border border-border group hover:border-accent transition-colors"
      class:opacity-50={!source.enabled}
    >
      <div class="flex items-center gap-4">
        <div
          class="h-10 w-10 rounded-full bg-accent/20 flex items-center justify-center text-accent"
        >
          {#if source.type === "local"}
            <Folder size={20} />
          {:else if source.type === "subsonic"}
            <Globe size={20} />
          {:else}
            <Music2 size={20} />
          {/if}
        </div>
        <div>
          <h3 class="font-medium text-text">{source.name}</h3>
          <p class="text-sm text-subtext break-all">
            {source.type === "local"
              ? source.path
              : source.type === "subsonic"
                ? source.url
                : "Tidal"}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-2">
        <label class="flex items-center gap-2 mr-2 cursor-pointer select-none">
          <input
            type="checkbox"
            checked={source.enabled}
            onchange={(e) =>
              handleToggleSource(source.id, e.currentTarget.checked)}
            class="hidden"
          />
          <div
            class="w-10 h-5 rounded-full transition-colors relative {source.enabled
              ? 'bg-accent'
              : 'bg-primary'}"
          >
            <div
              class="absolute top-1 left-1 w-3 h-3 bg-text rounded-full transition-transform {source.enabled
                ? 'translate-x-5'
                : 'translate-x-0'}"
            ></div>
          </div>
        </label>
        <Button
          variant="ghost"
          size="sm"
          onclick={() =>
            toast.promise(rescanSource.trigger(source.id), {
              loading: `Rescanning ${source.name}...`,
              success: "Rescan complete!",
              error: (e) => `Rescan failed: ${e}`,
            })}
          class="text-subtext hover:text-text h-10 w-10 p-0"
          leftIcon={RefreshCcw}
        ></Button>
        <Button
          variant="ghost"
          size="sm"
          onclick={() => handleDeleteSource(source.id)}
          class="text-subtext hover:text-red h-10 w-10 p-0"
          leftIcon={Trash}
        ></Button>
      </div>
    </div>
  {:else}
    <div
      class="text-center py-12 text-subtext border-2 border-dashed border-border rounded-xl"
    >
      <p>No sources added yet.</p>
      <Button
        variant="ghost"
        class="mt-2 text-accent"
        onclick={() => (showAddSourceModal = true)}
        >Add your first source</Button
      >
    </div>
  {/each}
</div>

<Modal bind:show={showAddSourceModal} maxWidth="max-w-md">
  <div class="p-6 space-y-6">
    <h2 class="text-xl font-bold text-text">Add Library Source</h2>
    <div class="flex p-1 bg-primary rounded-lg">
      <button
        class="flex-1 py-1.5 text-sm font-medium rounded-md transition-colors {newSourceType ===
        'local'
          ? 'bg-accent text-white'
          : 'text-subtext hover:text-text'}"
        onclick={() => (newSourceType = "local")}>Local Folder</button
      >
      <button
        class="flex-1 py-1.5 text-sm font-medium rounded-md transition-colors {newSourceType ===
        'subsonic'
          ? 'bg-accent text-white'
          : 'text-subtext hover:text-text'}"
        onclick={() => (newSourceType = "subsonic")}>Subsonic</button
      >
    </div>

    {#if newSourceType === "local"}
      <div class="space-y-4">
        <label class="flex flex-col gap-2">
          <span class="text-sm font-medium text-subtext">Name</span>
          <input
            type="text"
            bind:value={localName}
            class="bg-primary border border-border rounded-md p-2 text-text focus:border-accent focus:outline-none"
            placeholder="My Music"
          />
        </label>
        <label class="flex flex-col gap-2">
          <span class="text-sm font-medium text-subtext">Folder Path</span>
          <div class="relative">
            <input
              type="text"
              bind:value={localPath}
              class="bg-primary border border-border rounded-md p-2 text-text focus:border-accent focus:outline-none"
              placeholder="C:\Music"
            />
            {#if !localPath}
              <Button
                variant="ghost"
                size="sm"
                class="absolute right-2 top-1/2 -translate-y-1/2 px-3"
                onclick={async () => {
                  const path = await open({
                    directory: true,
                    multiple: false,
                    title: "Select Music Folder",
                    recursive: true,
                  });
                  if (path && typeof path === "string") {
                    localPath = path;
                  }
                }}
              >
                Explore
              </Button>
            {/if}
          </div>
          <p class="text-xs text-subtext">
            Absolute path to your music folder.
          </p>
        </label>
      </div>
    {:else}
      <div class="text-center py-8 text-subtext">
        Subsonic support coming soon.
      </div>
    {/if}

    <div class="flex justify-end gap-3 pt-2">
      <Button variant="ghost" onclick={() => (showAddSourceModal = false)}
        >Cancel</Button
      >
      <Button onclick={handleAddSource} disabled={addSource.isPending}>
        {#if addSource.isPending}Adding...{:else}Add Source{/if}
      </Button>
    </div>
  </div>
</Modal>
