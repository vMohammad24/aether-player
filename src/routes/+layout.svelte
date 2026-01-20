<script lang="ts">
  import AudioPlayer from "$lib/components/player/AudioPlayer.svelte";
  import Toaster from "$lib/components/Toaster.svelte";
  import { shortcuts } from "$lib/stores/shortcuts.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import { onMount } from "svelte";
  import Sidebar from "../lib/components/Sidebar.svelte";
  import TitleBar from "../lib/components/TitleBar.svelte";
  import "./layout.css";

  const { children } = $props();
  theme.init();

  onMount(() => {
    shortcuts.start();
    return shortcuts.stop;
  });
</script>

<Toaster />
<div
  class="flex flex-col h-screen w-full overflow-hidden bg-background text-text"
>
  <div class="flex flex-1 min-h-0 overflow-hidden">
    <Sidebar />
    <div class="flex flex-col flex-1 min-w-0 relative">
      <TitleBar />
      <main
        class="flex-1 overflow-y-auto relative w-full bg-background text-text p-6 md:p-12 font-sans selection:bg-cyan selection:text-background space-y-6 mx-auto"
      >
        {@render children()}
      </main>
    </div>
  </div>
  <AudioPlayer />
</div>
