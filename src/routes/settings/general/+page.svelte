<script lang="ts">
  import { commands } from "$lib/bindings";
  import Button from "$lib/components/Button.svelte";
  import { config } from "$lib/stores/config.svelte";
  import { Check, LoaderCircle, Music2 } from "@lucide/svelte";
  import { openUrl } from "@tauri-apps/plugin-opener";

  let isLoggingIn = $state(false);
  let error = $state<string | null>(null);

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
</div>
