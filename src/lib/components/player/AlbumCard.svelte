<script lang="ts">
  import { goto } from "$app/navigation";
  import { commands, type Album } from "$lib/bindings";
  import { contextMenu } from "$lib/hooks";
  import { player } from "$lib/stores/player/player.svelte";
  import { toast } from "$lib/stores/toast.svelte";
  import { Mic, Play } from "@lucide/svelte";
  import Image from "../Image.svelte";

  let { album }: { album: Album } = $props();

  async function playAlbum() {
    try {
      const res = await commands.getAlbumTracks(album.id);
      if (res.status === "ok") {
        const tracks = res.data;
        if (tracks.length === 0) {
          toast.error("No tracks found in this album");
          return;
        }
        player.playTracks(tracks, 0);
      } else {
        toast.error(`Failed to play album: ${res.error}`);
      }
    } catch (e) {
      console.error(e);
      toast.error("An error occurred");
    }
  }

  function handleNavigate() {
    goto(`/library/albums/${album.id}`);
  }
</script>

<div
  role="button"
  tabindex="0"
  onclick={handleNavigate}
  onkeydown={(e) => e.key === "Enter" && handleNavigate()}
  class="group relative aspect-square rounded-lg overflow-hidden border border-border bg-secondary cursor-pointer"
  use:contextMenu={[
    {
      type: "label",
      label: album.title,
    },
    {
      type: "action",
      label: "Play",
      leftIcon: Play,
      onClick: playAlbum,
    },
    {
      type: "separator",
    },
    {
      type: "action",
      label: "Go to Artist",
      leftIcon: Mic,
      onClick: () => goto(`/library/artists/${album.artistId}`),
    },
  ]}
>
  <Image
    src={album.coverArt}
    alt={album.title}
    type="cover"
    class="absolute inset-0 w-full h-full object-cover transition-transform duration-500 group-hover:scale-105"
  />

  <div
    class="absolute inset-0 bg-linear-to-t from-black/90 via-black/40 to-transparent opacity-60 group-hover:opacity-80 transition-opacity duration-300"
  ></div>

  <div
    class="absolute inset-0 flex items-center justify-center opacity-0 group-hover:opacity-100 transition-opacity duration-300"
  >
    <button
      class="bg-white text-black p-3 rounded-full hover:scale-105 transition-transform shadow-lg"
      onclick={(e) => {
        e.preventDefault();
        e.stopPropagation();
        playAlbum();
      }}
    >
      <Play size={24} fill="currentColor" class="ml-1" />
    </button>
  </div>

  <div class="absolute bottom-0 left-0 right-0 p-4 flex flex-col gap-0.5 z-10">
    <span
      class="text-white font-bold truncate leading-tight drop-shadow-md"
      title={album.title}>{album.title}</span
    >
    <span
      class="text-gray-300 text-xs truncate font-medium drop-shadow-md"
      title={album.artistName}>{album.artistName}</span
    >
  </div>
</div>
