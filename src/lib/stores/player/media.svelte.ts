import { createMutation, createResource } from '$lib/stores/resource.svelte';

class MediaStore {

    get favorites() { return createResource('getFavorites'); }

    recentAlbums(limit = 20) { return createResource('getRecentAlbums', limit); }
    randomAlbums(limit = 20) { return createResource('getRandomAlbums', limit); }
    mostPlayed(limit = 20) { return createResource('getMostPlayedTracks', limit); }

    artist(id: string) { return createResource('getArtist', id); }
    album(id: string) { return createResource('getAlbum', id); }
    artistAlbums(id: string) { return createResource('getArtistAlbums', id); }
    albumTracks(id: string) { return createResource('getAlbumTracks', id); }
    stats() { return createResource('getLibraryStats'); }
    genres() { return createResource('getGenres'); }
    playlists(providerId: string) { return createResource('getPlaylists', providerId); }
    playlistTracks(providerId: string, playlistId: string) { return createResource('getPlaylistTracks', providerId, playlistId); }

    search(query: string) { return createResource('search', query); }


    createPlaylist = createMutation('createPlaylist', { invalidate: 'getPlaylists' });
    deletePlaylist = createMutation('deletePlaylist', { invalidate: 'getPlaylists' });

    addToPlaylist = createMutation('addToPlaylist', { invalidate: 'getPlaylistTracks' });
    removeFromPlaylist = createMutation('removeFromPlaylist', { invalidate: 'getPlaylistTracks' });

    setFavorite = createMutation('setFavorite', { invalidate: ['getFavorites', 'getAlbumTracks', 'getPlaylistTracks', 'search'] });

    addSource = createMutation('addSource', { invalidate: 'getAppConfig' });
    deleteSource = createMutation('deleteSource', { invalidate: 'getAppConfig' });
    addLibraryRoot = createMutation('addLibraryRoot', { invalidate: ['getRecentAlbums', 'getArtistAlbums', 'getAlbumTracks', 'search'] });
    scanLibrary = createMutation('scanLibrary', { invalidate: ['getRecentAlbums', 'getArtistAlbums', 'getAlbumTracks', 'search'] });
    scanLibraries = createMutation('scanLibraries', { invalidate: ['getRecentAlbums', 'getArtistAlbums', 'getAlbumTracks', 'search'] });
}

export const media = new MediaStore();
