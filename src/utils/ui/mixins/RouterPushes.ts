/*
 *  RouterPushes.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component, Vue } from 'vue-facing-decorator'

import { v4 } from 'uuid'

@Component
export default class RouterPushes extends Vue {
  public gotoAlbum(album: Album, defaultProviders?: string[]) {
    try {
      this.$router.push({
        name: 'albums-single',
        query: {
          id: album.album_id as string,
          name: album.album_name,
          cover_high: album.album_coverPath_high,
          cover_low: album.album_coverPath_low,
          artist: album.album_artist,
          year: (album.year ?? 0).toString(),
          extra_info: JSON.stringify(album.album_extra_info) ?? '',
          defaultProviders,
        },
      })
    } catch (e) {
      console.debug(e)
    }
  }

  public gotoGenre(genre: Genre) {
    try {
      this.$router.push({
        name: 'genre-single',
        query: {
          id: genre.genre_id,
        },
      })
    } catch (e) {
      console.debug(e)
    }
  }

  public gotoArtist(artist: Artists, defaultProviders?: string[]) {
    try {
      this.$router.push({
        name: 'artists-single',
        query: {
          id: artist.artist_id,
          name: artist.artist_name ?? '',
          cover: artist.artist_coverPath ?? '',
          extra_info: JSON.stringify(artist.artist_extra_info) ?? '',
          defaultProviders,
        },
      })
    } catch (e) {
      console.debug(e)
    }
  }

  public gotoPlaylist(playlist: Playlist) {
    try {
      this.$router.push({
        name: 'playlists-single',
        query: {
          id: playlist.playlist_id,
          playlist_id: playlist.playlist_id,
          playlist_name: playlist.playlist_name,
          playlist_coverPath: playlist.playlist_coverPath ?? '',
          playlist_song_count: (playlist.playlist_song_count ?? 0).toString(),
          playlist_path: playlist.playlist_path ?? '',
          extension: playlist.extension ?? '',
        },
      })
    } catch (e) {
      console.debug(e)
    }
  }
}
