<script lang="ts">
  const palette = {
    "Core Structure": [
      { name: "background", cls: "bg-background" },
      { name: "secondary", cls: "bg-secondary" },
      { name: "primary", cls: "bg-primary" },
      { name: "accent", cls: "bg-accent" },
      { name: "border", cls: "bg-border" },
    ],
    Typography: [
      { name: "text", cls: "bg-text" },
      { name: "gray", cls: "bg-gray" },
    ],
    Brand: [
      { name: "cyan", cls: "bg-cyan" },
      { name: "blue", cls: "bg-blue" },
      { name: "gold", cls: "bg-gold" },
    ],
    Status: [
      { name: "red", cls: "bg-red" },
      { name: "green", cls: "bg-green" },
      { name: "purple", cls: "bg-purple" },
      { name: "pink", cls: "bg-pink" },
      { name: "teal", cls: "bg-teal" },
      { name: "mauve", cls: "bg-mauve" },
      { name: "orange", cls: "bg-orange" },
      { name: "yellow", cls: "bg-yellow" },
    ],
  };
</script>

<div
  class="min-h-screen bg-background text-text p-8 md:p-12 font-sans selection:bg-cyan selection:text-background"
>
  <div class="grid lg:grid-cols-[1fr_300px] gap-12">
    <div class="space-y-12">
      {#each Object.entries(palette) as [category, colors]}
        <section>
          <h2 class="text-sm font-bold uppercase tracking-wider text-gray mb-4">
            {category}
          </h2>
          <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-4">
            {#each colors as color}
              <div class="group flex flex-col gap-2">
                <div
                  class="relative w-full aspect-video rounded-md {color.cls} border border-white/5 shadow-lg group-hover:scale-[1.02] transition-transform duration-200"
                >
                  <div
                    class="absolute inset-0 bg-linear-to-tr from-white/0 to-white/10 opacity-0 group-hover:opacity-100 transition-opacity"
                  ></div>
                </div>
                <span class="text-sm font-medium text-text">{color.name}</span>
              </div>
            {/each}
          </div>
        </section>
      {/each}
    </div>
  </div>
</div>
