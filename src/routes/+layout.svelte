<script lang="ts">
  import Sidebar from "../lib/components/Sidebar.svelte";
  import TitleBar from "../lib/components/TitleBar.svelte";
  import "./layout.css";

  const { children } = $props();
</script>

<div class="flex h-screen w-full overflow-hidden bg-background text-text">
  <Sidebar />
  <div class="flex flex-col flex-1 min-w-0 relative">
    <TitleBar />
    <main class="flex-1 overflow-y-auto relative">
      {@render children()}
    </main>
  </div>
</div>