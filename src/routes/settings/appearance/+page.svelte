<script lang="ts">
  import Button from "$lib/components/Button.svelte";
  import Modal from "$lib/components/Modal.svelte";
  import { confirm } from "$lib/stores/confirm.svelte";
  import { theme, type Theme } from "$lib/stores/theme.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { Download, Pencil, Plus, Trash2, Upload } from "@lucide/svelte";

  let showThemeEditor = $state(false);
  let editingTheme = $state<Theme | null>(null);

  function handleThemeSelect(themeId: string) {
    theme.set(themeId);
  }

  function handleCreateTheme() {
    editingTheme = {
      id: `custom-${Date.now()}`,
      name: "New Theme",
      colors: { ...theme.current.colors },
      options: { ...theme.current.options },
    };
    showThemeEditor = true;
  }

  function handleEditTheme(t: Theme) {
    editingTheme = JSON.parse(JSON.stringify(t));
    showThemeEditor = true;
  }

  function saveTheme() {
    if (!editingTheme) return;
    const res = theme.add(editingTheme);
    if (res.success) {
      toast.success("Theme saved");
      showThemeEditor = false;
      handleThemeSelect(editingTheme.id);
    } else {
      toast.error(res.error || "Failed to save theme");
    }
  }

  async function deleteTheme(id: string) {
    if (await confirm("Delete this theme?")) {
      theme.remove(id);
      toast.success("Theme deleted");
    }
  }

  const colorKeys = [
    "background",
    "primary",
    "secondary",
    "accent",
    "text",
    "subtext",
    "border",
    "blue",
    "mauve",
    "red",
    "green",
    "pink",
    "yellow",
    "orange",
    "purple",
    "teal",
    "cyan",
  ];
</script>

<div class="flex items-center justify-between border-b border-white/10 pb-3">
  <h2 class="text-xl font-semibold text-text">Appearance</h2>
  <div class="flex gap-2">
    <Button
      onclick={handleCreateTheme}
      size="sm"
      variant="outline"
      leftIcon={Plus}>Create Theme</Button
    >
    <label class="cursor-pointer">
      <span class="sr-only">Import Theme</span>
      <Button
        size="sm"
        variant="outline"
        onclick={() => document.getElementById("theme-import")?.click()}
        leftIcon={Upload}>Import</Button
      >
      <input
        type="file"
        id="theme-import"
        accept=".json"
        class="hidden"
        onchange={(e) => {
          const file = (e.target as HTMLInputElement).files?.[0];
          if (file)
            theme.import(file).then((res) => {
              if (res.success) toast.success("Theme imported");
              else toast.error(res.error || "Failed to import");
            });
        }}
      />
    </label>
  </div>
</div>

<div class="grid grid-cols-3 lg:grid-cols-4 gap-4">
  {#each theme.all as t (t.id)}
    <div
      class="relative overflow-hidden rounded-xl border transition-all cursor-pointer group
             {t.id === theme.selectedId
        ? 'border-accent ring-1 ring-accent'
        : 'border-white/10 hover:border-white/20'}"
      onclick={() => handleThemeSelect(t.id)}
      role="button"
      tabindex="0"
      onkeypress={() => handleThemeSelect(t.id)}
    >
      <div
        class="h-32 w-full flex text-[10px] select-none pointer-events-none"
        style="background: {t.colors.background}; color: {t.colors.text}"
      >
        <div
          class="w-1/3 h-full flex flex-col p-3 gap-2 border-r"
          style="background: {t.colors.secondary}; border-color: {t.colors
            .border}"
        >
          <div
            class="h-2 w-1/2 rounded-full opacity-50"
            style="background: {t.colors.text}"
          ></div>

          <div class="flex flex-col gap-1.5 mt-1">
            <div
              class="h-4 w-full rounded flex items-center px-2"
              style="background: {t.colors.accent}20; color: {t.colors
                .accent}; border-radius: {t.options.radius}"
            >
              <div
                class="h-1.5 w-1.5 rounded-full mr-1.5"
                style="background: currentColor"
              ></div>
              <div
                class="h-1.5 w-1/2 rounded-full opacity-80"
                style="background: currentColor"
              ></div>
            </div>
            <div class="h-4 w-full rounded flex items-center px-2 opacity-60">
              <div
                class="h-1.5 w-3/4 rounded-full"
                style="background: {t.colors.text}"
              ></div>
            </div>
            <div class="h-4 w-full rounded flex items-center px-2 opacity-60">
              <div
                class="h-1.5 w-1/2 rounded-full"
                style="background: {t.colors.text}"
              ></div>
            </div>
          </div>
        </div>

        <div class="flex-1 p-3 flex flex-col gap-3">
          <div class="flex justify-between items-center">
            <div
              class="h-2.5 w-1/3 rounded-full"
              style="background: {t.colors.text}"
            ></div>
            <div
              class="h-4 w-4 rounded-full opacity-20"
              style="background: {t.colors.text}"
            ></div>
          </div>

          <div
            class="flex-1 rounded border p-2 flex flex-col gap-2"
            style="background: {t.colors.primary}; border-color: {t.colors
              .border}; border-radius: {t.options.radius}"
          >
            <div class="flex gap-2">
              <div
                class="h-8 w-8 rounded shrink-0 opacity-20"
                style="background: {t.colors.text}; border-radius: {t.options
                  .radius}"
              ></div>
              <div class="flex flex-col gap-1.5 pt-0.5 w-full">
                <div
                  class="h-2 w-3/4 rounded-full"
                  style="background: {t.colors.text}"
                ></div>
                <div
                  class="h-1.5 w-1/2 rounded-full opacity-50"
                  style="background: {t.colors.text}"
                ></div>
              </div>
            </div>
            <div class="mt-auto flex justify-end">
              <div
                class="h-4 px-3 rounded flex items-center"
                style="background: {t.colors.accent}; border-radius: {t.options
                  .radius}"
              >
                <div class="h-1.5 w-6 bg-white rounded-full opacity-90"></div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <div class="p-3 bg-secondary flex justify-between items-center">
        <div>
          <div class="font-medium text-text">{t.name}</div>
          <div class="text-xs text-subtext">
            {t.id.startsWith("custom") ||
            !["default", "oled", "catppuccin-frappe"].includes(t.id)
              ? "Custom"
              : "Built-in"}
          </div>
        </div>

        <div
          class="flex gap-1 opacity-0 group-hover:opacity-100 transition-opacity"
        >
          <Button
            variant="ghost"
            size="sm"
            class="p-1.5 h-8 w-8 text-subtext hover:text-text"
            onclick={() => {
              const res = theme.export(t.id);
              if (res.success) {
                toast.success("Theme exported");
              } else {
                toast.error(res.error || "Failed to export theme");
              }
            }}
            leftIcon={Download}
            title="Export"
          />
          <Button
            variant="ghost"
            size="sm"
            class="p-1.5 h-8 w-8 text-subtext hover:text-text"
            onclick={() => {
              handleEditTheme(t);
            }}
            leftIcon={Pencil}
            title="Edit/Copy"
          />
          {#if !["default", "oled", "catppuccin-frappe", "catppuccin-macchiato", "catppuccin-mocha", "catppuccin-latte"].includes(t.id)}
            <Button
              variant="ghost"
              size="sm"
              class="p-1.5 h-8 w-8 text-subtext hover:text-red-500 hover:bg-red/10"
              onclick={() => {
                deleteTheme(t.id);
              }}
              leftIcon={Trash2}
              title="Delete"
            />
          {/if}
        </div>
      </div>
    </div>
  {/each}
</div>

<Modal bind:show={showThemeEditor} maxWidth="max-w-3xl">
  <div class="p-6 h-[80vh] flex flex-col">
    <div class="flex justify-between items-center mb-6">
      <h2 class="text-xl font-bold text-text">Edit Theme</h2>
      <div class="flex gap-2">
        <Button variant="ghost" onclick={() => (showThemeEditor = false)}
          >Cancel</Button
        >
        <Button onclick={saveTheme}>Save Theme</Button>
      </div>
    </div>

    {#if editingTheme}
      <div class="flex-1 overflow-y-auto pr-2 space-y-6">
        <div class="grid grid-cols-2 gap-6">
          <label class="flex flex-col gap-2">
            <span class="text-sm font-medium text-subtext">Theme Name</span>
            <input
              type="text"
              bind:value={editingTheme.name}
              class="bg-black/20 border border-white/10 rounded-md p-2 text-text focus:border-accent focus:outline-none"
            />
          </label>
          <div class="flex flex-col gap-2">
            <div class="flex justify-between items-center">
              <span class="text-sm font-medium text-subtext">Radius</span>
              <span
                class="text-xs font-mono text-accent bg-accent/10 px-1.5 py-0.5 rounded"
                >{editingTheme.options.radius}</span
              >
            </div>
            <input
              type="range"
              min="0"
              max="24"
              step="1"
              value={parseFloat(editingTheme.options.radius) || 0}
              oninput={(e) =>
                (editingTheme!.options.radius = `${e.currentTarget.value}px`)}
              class="w-full h-1.5 bg-black/40 rounded-lg appearance-none cursor-pointer accent-accent mt-2"
            />
            <div class="flex justify-between text-[10px] text-subtext px-1">
              <span>0px</span>
              <span>12px</span>
              <span>24px</span>
            </div>
          </div>
        </div>

        <h3 class="text-lg font-medium text-text border-b border-white/10 pb-2">
          Colors
        </h3>

        <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4">
          {#each colorKeys as key}
            <div
              class="flex items-center gap-3 p-3 bg-secondary/50 rounded-lg border border-white/5"
            >
              <input
                type="color"
                bind:value={editingTheme.colors[key]}
                class="h-8 w-8 rounded cursor-pointer border-none bg-transparent"
              />
              <div class="flex-1 min-w-0">
                <div class="text-sm font-medium text-text capitalize">
                  {key}
                </div>
                <input
                  type="text"
                  bind:value={editingTheme.colors[key]}
                  class="w-full text-xs text-subtext bg-transparent border-none p-0 focus:ring-0"
                />
              </div>
            </div>
          {/each}
        </div>

        <div class="p-4 bg-secondary rounded-xl border border-white/10">
          <h4 class="text-sm font-medium text-text mb-3">Preview</h4>
          <div
            class="p-4 rounded-lg flex flex-col gap-2 transition-all"
            style="background: {editingTheme.colors
              .background}; color: {editingTheme.colors
              .text}; border-radius: {editingTheme.options.radius}"
          >
            <div class="flex justify-between items-center">
              <span class="font-bold">Header</span>
              <span
                class="text-xs px-2 py-1 rounded"
                style="background: {editingTheme.colors.accent}; color: white"
                >Badge</span
              >
            </div>
            <div
              class="h-20 rounded p-2"
              style="background: {editingTheme.colors
                .secondary}; border: 1px solid {editingTheme.colors.border}"
            >
              Content
              <button
                class="px-3 py-1 text-sm rounded mt-2 block w-max"
                style="background: {editingTheme.colors
                  .primary}; color: {editingTheme.colors.text}">Button</button
              >
            </div>
          </div>
        </div>
      </div>
    {/if}
  </div>
</Modal>
