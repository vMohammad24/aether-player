<script lang="ts">
  import { config } from "$lib/stores/config.svelte";
  import { Music2 } from "@lucide/svelte";

  function toggleLastFm() {
    if (config.lastfm) {
      config.lastfm.enabled = !config.lastfm.enabled;
    } else {
      config.lastfm = {
        apiKey: "",
        apiSecret: "",
        sessionKey: null,
        enabled: true,
      };
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
            checked={config.lastfm?.enabled ?? false}
            onchange={toggleLastFm}
            class="hidden"
          />
          <div
            class="w-10 h-5 rounded-full transition-colors relative {config
              .lastfm?.enabled
              ? 'bg-accent'
              : 'bg-primary'}"
          >
            <div
              class="absolute top-1 left-1 w-3 h-3 bg-text rounded-full transition-transform {config
                .lastfm?.enabled
                ? 'translate-x-5'
                : 'translate-x-0'}"
            ></div>
          </div>
        </label>
      </div>

      {#if config.lastfm?.enabled}
        <div
          class="pt-4 border-t border-border grid grid-cols-1 md:grid-cols-2 gap-4"
        >
          <label class="flex flex-col gap-2">
            <span
              class="text-xs font-medium text-subtext uppercase tracking-wider"
              >API Key</span
            >
            <input
              type="password"
              bind:value={config.lastfm.apiKey}
              class="bg-primary border border-border rounded-md p-2 text-sm text-text focus:border-accent focus:outline-none"
              placeholder="Last.fm API Key"
            />
          </label>
          <label class="flex flex-col gap-2">
            <span
              class="text-xs font-medium text-subtext uppercase tracking-wider"
              >API Secret</span
            >
            <input
              type="password"
              bind:value={config.lastfm.apiSecret}
              class="bg-primary border border-border rounded-md p-2 text-sm text-text focus:border-accent focus:outline-none"
              placeholder="Last.fm API Secret"
            />
          </label>
        </div>
        <p class="text-xs text-subtext italic">
          Get your api key from <a
            href="https://www.last.fm/api/account/create"
            target="_blank"
            rel="noopener noreferrer"
            class="underline">here</a
          >.
        </p>
      {/if}
    </div>
  </div>
</div>
