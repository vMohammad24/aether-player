<script lang="ts">
  import { commands } from "$lib/bindings";
  import Button from "$lib/components/Button.svelte";
  import { config } from "$lib/stores/config.svelte";
  import { createMutation, createResource } from "$lib/stores/resource.svelte";
  import { Check, Gamepad2, LoaderCircle, Music2 } from "@lucide/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

  let isLoggingIn = $state(false);
  let error = $state<string | null>(null);
  const devices = createResource("getAudioDevices");
  const setDevice = createMutation("setAudioDevice", {
    invalidate: "getAudioDevices",
  });
  function toggleLastFm() {
    if (config.lastfmSession) {
      config.lastfmSession.enabled = !config.lastfmSession.enabled;
    } else {
      config.lastfmSession = {
        username: "",
        sessionKey: "",
        enabled: true,
      };
    }
  }

  function toggleDiscordRpc() {
    if (config.discordRpc) {
      config.discordRpc.enabled = !config.discordRpc.enabled;
    } else {
      config.discordRpc = {
        enabled: true,
      };
    }
  }

  async function startLogin() {
    isLoggingIn = true;
    error = null;

    try {
      const result = await commands.loginLastfm();
      if (result.status === "ok") {
        await openUrl(result.data.url);

        const token = result.data.token;
        let attempts = 0;
        const maxAttempts = 60;

        const interval = setInterval(async () => {
          attempts++;
          if (attempts > maxAttempts) {
            clearInterval(interval);
            isLoggingIn = false;
            error = "Login timed out. Please try again.";
            return;
          }

          if (!isLoggingIn) {
            clearInterval(interval);
            return;
          }

          try {
            const res = await commands.finishLastfmLogin(token);
            if (res.status === "ok") {
              clearInterval(interval);
              await config.forceSync();
              isLoggingIn = false;
            } else {
            }
          } catch (e) {}
        }, 2000);
      } else {
        error = result.error;
        isLoggingIn = false;
      }
    } catch (e) {
      error = String(e);
      isLoggingIn = false;
    }
  }
</script>

<header>
  <h2 class="text-xl font-semibold text-text border-b border-border pb-3">
    General Settings
  </h2>
</header>

<div class="space-y-6 mt-6">
  <div class="p-4 bg-secondary rounded-xl border border-border space-y-4">
    <div>
      <span class="block text-sm font-medium text-subtext mb-2">
        Audio Backend
      </span>
      <div class="text-sm text-text">
        Currently using <span class="font-mono p-1">MPV</span>
      </div>
      <div class="mt-2">
        <label
          for="audio-device"
          class="block text-sm font-medium text-subtext mb-1"
          >Audio Output Device</label
        >
        <select
          id="audio-device"
          class="w-full bg-primary border border-border rounded-md p-2 text-sm text-text focus:border-accent focus:outline-none"
          bind:value={config.audioOutputDevice}
          onchange={() => setDevice.trigger(config.audioOutputDevice)}
        >
          <option value="" disabled>
            {#if devices.loading}
              Loading devices...
            {:else if devices.error}
              Error loading devices
            {:else}
              Select an audio device
            {/if}
          </option>
          {#each devices.data ?? [] as device}
            <option value={device.id}>
              {device.name}
              {device.isDefault ? "(Default)" : ""}{" "}
              {device.isCurrent ? "(Current)" : ""}
            </option>
          {/each}
        </select>
      </div>
    </div>
  </div>

  <div>
    <h3 class="text-lg font-medium text-text mb-4">Last.fm Integration</h3>
    <div class="p-4 bg-secondary rounded-xl border border-border space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-red-500/10 text-red-500 rounded-lg">
            <Music2 size={20} />
          </div>
          <div>
            <span class="block text-sm font-medium text-text"
              >Last.fm Scrobbling</span
            >
            <span class="text-xs text-subtext"
              >Track your listening history</span
            >
          </div>
        </div>

        <label class="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            checked={config.lastfmSession?.enabled ?? false}
            onchange={toggleLastFm}
            class="hidden"
          />
          <div
            class="w-10 h-5 rounded-full transition-colors relative {config
              .lastfmSession?.enabled
              ? 'bg-accent'
              : 'bg-primary'}"
          >
            <div
              class="absolute top-1 left-1 w-3 h-3 bg-text rounded-full transition-transform {config
                .lastfmSession?.enabled
                ? 'translate-x-5'
                : 'translate-x-0'}"
            ></div>
          </div>
        </label>
      </div>

      {#if config.lastfmSession?.enabled}
        <div
          class="pt-4 border-t border-border grid grid-cols-1 md:grid-cols-2 gap-4"
        >
          <label class="flex flex-col gap-2 md:col-span-2">
            <span
              class="text-xs font-medium text-subtext uppercase tracking-wider"
              >Username</span
            >
            <div class="flex items-center gap-2">
              <input
                type="text"
                bind:value={config.lastfmSession.username}
                readonly
                class="bg-primary border border-border rounded-md p-2 text-sm text-text focus:border-accent focus:outline-none flex-1 disabled:opacity-50"
                placeholder="Not connected"
              />
              {#if config.lastfmSession.sessionKey}
                <span
                  class="text-green flex items-center gap-1 text-xs font-medium px-2 py-1 bg-green/10 rounded"
                >
                  <Check size={12} /> Connected
                </span>
              {/if}
            </div>
          </label>

          <div class="md:col-span-2 pt-2">
            {#if error}
              <div class="text-red text-sm mb-2">{error}</div>
            {/if}

            <Button onclick={startLogin} disabled={isLoggingIn}>
              {#if isLoggingIn}
                <LoaderCircle size={16} class="animate-spin" />
                Waiting for approval...
              {:else}
                {config.lastfmSession.sessionKey
                  ? "Reconnect Account"
                  : "Connect Account"}
              {/if}
            </Button>
          </div>
        </div>
      {/if}
    </div>
  </div>

  <div>
    <h3 class="text-lg font-medium text-text mb-4">Discord Rich Presence</h3>
    <div class="p-4 bg-secondary rounded-xl border border-border space-y-4">
      <div class="flex items-center justify-between">
        <div class="flex items-center gap-3">
          <div class="p-2 bg-indigo-500/10 text-indigo-500 rounded-lg">
            <Gamepad2 size={20} />
          </div>
          <div>
            <span class="block text-sm font-medium text-text"
              >Discord Status</span
            >
            <span class="text-xs text-subtext"
              >Show what you're listening to on Discord</span
            >
          </div>
        </div>

        <label class="flex items-center gap-2 cursor-pointer select-none">
          <input
            type="checkbox"
            checked={config.discordRpc?.enabled ?? false}
            onchange={toggleDiscordRpc}
            class="hidden"
          />
          <div
            class="w-10 h-5 rounded-full transition-colors relative {config
              .discordRpc?.enabled
              ? 'bg-accent'
              : 'bg-primary'}"
          >
            <div
              class="absolute top-1 left-1 w-3 h-3 bg-text rounded-full transition-transform {config
                .discordRpc?.enabled
                ? 'translate-x-5'
                : 'translate-x-0'}"
            ></div>
          </div>
        </label>
      </div>

      {#if config.discordRpc?.enabled}
        <div class="pt-4 border-t border-border space-y-4">
          <div class="grid grid-cols-1 md:grid-cols-2 gap-4">
            <div class="space-y-2">
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={config.discordRpc.showDetails}
                  class="rounded border-border bg-primary text-accent focus:ring-accent"
                />
                <span class="text-sm text-text">Top Line (Details)</span>
              </label>
              {#if config.discordRpc.showDetails}
                <input
                  type="text"
                  bind:value={config.discordRpc.detailsFormat}
                  class="w-full bg-primary border border-border rounded-md p-2 text-sm text-text focus:border-accent focus:outline-none placeholder-subtext"
                  placeholder="{`{track}`} by {`{artist}`}"
                />
                <p class="text-xs text-subtext">
                  Variables: <code class="bg-primary px-1 rounded"
                    >{`{track}`}</code
                  >, <code class="bg-primary px-1 rounded">{`{artist}`}</code>,
                  <code class="bg-primary px-1 rounded">{`{album}`}</code>
                </p>
              {/if}
            </div>

            <div class="space-y-2">
              <label class="flex items-center gap-3 cursor-pointer">
                <input
                  type="checkbox"
                  bind:checked={config.discordRpc.showState}
                  class="rounded border-border bg-primary text-accent focus:ring-accent"
                />
                <span class="text-sm text-text">Bottom Line (State)</span>
              </label>
              {#if config.discordRpc.showState}
                <input
                  type="text"
                  bind:value={config.discordRpc.stateFormat}
                  class="w-full bg-primary border border-border rounded-md p-2 text-sm text-text focus:border-accent focus:outline-none placeholder-subtext"
                  placeholder="on {`{album}`}"
                />
              {/if}
            </div>
          </div>

          <div class="grid grid-cols-1 md:grid-cols-2 gap-3 pt-2">
            <label class="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                bind:checked={config.discordRpc.showTime}
                class="rounded border-border bg-primary text-accent focus:ring-accent"
              />
              <span class="text-sm text-text">Show Time Elapsed</span>
            </label>

            <label class="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                bind:checked={config.discordRpc.activityOnPause}
                class="rounded border-border bg-primary text-accent focus:ring-accent"
              />
              <span class="text-sm text-text">Show Status when Paused</span>
            </label>

            <label class="flex items-center gap-3 cursor-pointer">
              <input
                type="checkbox"
                bind:checked={config.discordRpc.showArtistIcon}
                class="rounded border-border bg-primary text-accent focus:ring-accent"
              />
              <span class="text-sm text-text">Show Artist Icon (Small)</span>
            </label>
          </div>
        </div>
      {/if}
    </div>
  </div>
</div>
