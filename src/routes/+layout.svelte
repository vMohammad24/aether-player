<script lang="ts">
  import Toaster from "$lib/components/Toaster.svelte";
  import { theme } from "$lib/stores/theme.svelte";
  import Sidebar from "../lib/components/Sidebar.svelte";
  import TitleBar from "../lib/components/TitleBar.svelte";
  import "./layout.css";

  const { children } = $props();
  theme.init();
</script>

<Toaster />
<div class="flex h-screen w-full overflow-hidden bg-background text-text">
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
