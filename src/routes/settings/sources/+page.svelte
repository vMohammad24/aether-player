<script lang="ts">
  import {
    commands,
    type DeviceAuthPending,
    type SourceConfig,
  } from "$lib/bindings";
  import Button from "$lib/components/Button.svelte";
  import Tidal from "$lib/components/logos/Tidal.svg";
  import Modal from "$lib/components/Modal.svelte";
  import { config } from "$lib/stores/config.svelte";
  import { confirm } from "$lib/stores/confirm.svelte";
  import { createMutation } from "$lib/stores/resource.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { md5 } from "$lib/util";
  import { Folder, Globe, Music2, RefreshCcw, Trash } from "@lucide/svelte";
  import { open } from "@tauri-apps/plugin-dialog";
  import { openUrl } from "@tauri-apps/plugin-opener";

  let showAddSourceModal = $state(false);
  let newSourceType = $state<"local" | "subsonic" | "tidal">("local");

  let localPath = $state("");
  let localName = $state("My Music");

  let subsonicName = $state("My Subsonic");
  let subsonicUrl = $state("");
  let subsonicUser = $state("");
  let subsonicPass = $state("");

  let tidalAuth = $state<DeviceAuthPending | null>(null);
  let tidalLoading = $state(false);

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

      await addSource.trigger(newSource);
    } else {
      if (!subsonicUrl) return toast.error("URL is required");
      if (!subsonicUser) return toast.error("Username is required");
      if (!subsonicPass) return toast.error("Password is required");

      const salt = Math.random().toString(36).substring(2, 10);
      const token = md5(subsonicPass + salt);

      const newSource: SourceConfig = {
        type: "subsonic",
        id: generateId(),
        name: subsonicName,
        url: subsonicUrl,
        username: subsonicUser,
        token: token,
        salt: salt,
        enabled: true,
      };

      await addSource.trigger(newSource);
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
    subsonicName = "My Subsonic";
    subsonicUrl = "";
    subsonicUser = "";
    subsonicPass = "";
    tidalAuth = null;
    tidalLoading = false;
  }

  async function handleTidalLogin() {
    tidalLoading = true;
    try {
      const res = await commands.startTidalLogin();
      if (res.status === "ok") {
        tidalAuth = res.data;
        pollTidal();
      } else {
        toast.error(res.error);
        tidalLoading = false;
      }
    } catch (e) {
      toast.error(String(e));
      tidalLoading = false;
    }
  }

  async function pollTidal() {
    try {
      const res = await commands.pollTidalLogin(tidalAuth!);
      if (res.status === "ok") {
        toast.success("Tidal account connected");
        config.forceSync();

        showAddSourceModal = false;
        resetSourceForm();
      } else {
        if (showAddSourceModal) {
          toast.error("Login failed: " + res.error);
        }
        tidalLoading = false;
        tidalAuth = null;
      }
    } catch (e) {
      if (showAddSourceModal) toast.error(String(e));
      tidalLoading = false;
      tidalAuth = null;
    }
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
          {:else if source.type === "tidal"}
            <img src={Tidal} alt="Tidal Logo" class="h-5 w-5" />
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
                : source.user_id}
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
              loading:
                source.type === "local"
                  ? `Rescanning ${source.name}...`
                  : `Cleared cache for ${source.name}`,
              success:
                source.type === "local"
                  ? `Rescan of ${source.name} completed`
                  : `Cache for ${source.name} cleared`,
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
      <button
        class="flex-1 py-1.5 text-sm font-medium rounded-md transition-colors {newSourceType ===
        'tidal'
          ? 'bg-accent text-white'
          : 'text-subtext hover:text-text'}"
        onclick={() => (newSourceType = "tidal")}>Tidal</button
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
    {:else if newSourceType === "subsonic"}
      <div class="space-y-4">
        <label class="flex flex-col gap-2">
          <span class="text-sm font-medium text-subtext">Name</span>
          <input
            type="text"
            bind:value={subsonicName}
            class="bg-primary border border-border rounded-md p-2 text-text focus:border-accent focus:outline-none"
            placeholder="My Subsonic"
          />
        </label>
        <label class="flex flex-col gap-2">
          <span class="text-sm font-medium text-subtext">Server URL</span>
          <input
            type="text"
            bind:value={subsonicUrl}
            class="bg-primary border border-border rounded-md p-2 text-text focus:border-accent focus:outline-none"
            placeholder="https://music.example.com"
          />
          <p class="text-xs text-subtext">
            The base URL of your Subsonic/Navidrome server.
          </p>
        </label>
        <div class="grid grid-cols-2 gap-4">
          <label class="flex flex-col gap-2">
            <span class="text-sm font-medium text-subtext">Username</span>
            <input
              type="text"
              bind:value={subsonicUser}
              class="bg-primary border border-border rounded-md p-2 text-text focus:border-accent focus:outline-none"
              placeholder="user"
            />
          </label>
          <label class="flex flex-col gap-2">
            <span class="text-sm font-medium text-subtext">Password</span>
            <input
              type="password"
              bind:value={subsonicPass}
              class="bg-primary border border-border rounded-md p-2 text-text focus:border-accent focus:outline-none"
              placeholder="••••••"
            />
          </label>
        </div>
      </div>
    {:else if newSourceType === "tidal"}
      {#if !tidalAuth}
        <div class="space-y-4 text-center py-4">
          <p class="text-sm text-text">
            Connect your Tidal account to access your music library.
          </p>
          <p class="text-sm text-yellow">
            WARNING: This feature is still VERY experimental and may not work
            perfectly. (playback is broken atm)
          </p>
          <Button onclick={handleTidalLogin} disabled={tidalLoading}>
            {#if tidalLoading}Starting...{:else}Login with Tidal{/if}
          </Button>
        </div>
      {:else}
        <div class="space-y-4 text-center">
          <p class="text-sm text-text font-medium">Action Required</p>
          <p class="text-sm text-subtext">
            Please visit the link below and enter the code to authorize Aether
            Player.
          </p>

          <div class="bg-secondary p-4 rounded-md border border-border">
            <p class="text-2xl font-mono text-accent tracking-widest">
              {tidalAuth.user_code}
            </p>
          </div>

          <Button
            variant="ghost"
            onclick={() =>
              openUrl(`https://${tidalAuth!.verification_uri_complete}`)}
          >
            Open Login Page
          </Button>

          <div
            class="flex items-center justify-center gap-2 text-xs text-subtext mt-4"
          >
            <div
              class="w-4 h-4 border-2 border-subtext border-t-accent rounded-full animate-spin"
            ></div>
            Waiting for authorization...
          </div>
        </div>
      {/if}
    {/if}

    <div class="flex justify-end gap-3 pt-2">
      <Button
        variant="ghost"
        onclick={() => {
          showAddSourceModal = false;
          resetSourceForm();
        }}>Cancel</Button
      >
      {#if newSourceType !== "tidal"}
        <Button onclick={handleAddSource} disabled={addSource.isPending}>
          {#if addSource.isPending}Adding...{:else}Add Source{/if}
        </Button>
      {:else}{/if}
    </div>
  </div>
</Modal>
