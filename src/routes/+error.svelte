<script lang="ts">
  import { page } from "$app/state";
  import Button from "$lib/components/Button.svelte";
  import { House, TriangleAlert } from "@lucide/svelte";
</script>

<div
  class="flex h-full w-full flex-col items-center justify-center gap-8 text-center"
>
  <div class="flex flex-col items-center gap-4">
    <div class="rounded-full bg-red/10 p-4">
      <TriangleAlert class="h-12 w-12 text-red" />
    </div>
    <h1 class="text-6xl font-bold tracking-tight text-text">{page.status}</h1>
  </div>

  <div class="max-w-md space-y-2">
    <h2 class="text-2xl font-semibold text-text">Something went wrong</h2>
    <p class="text-subtext text-lg">
      {page.error?.message || "An unexpected error occurred."}
    </p>
  </div>

  <Button href="/" variant="secondary" leftIcon={House}>Return Home</Button>
</div>
