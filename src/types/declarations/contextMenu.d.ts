/*
 *  contextMenu.d.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

type ContextMenuArgs =
  | {
      type: 'SONGS'
      args: {
        exclude?: string
        refreshCallback?: () => void
        songs: Song[]
        isRemote?: boolean
      }
    }
  | {
      type: 'GENERAL_SONGS'
      args: {
        sortOptions?: Sort<SongSortOptions[]>
        showHiddenToggle?: boolean
        isShowingHidden?: boolean
        refreshCallback?: (showHidden?: boolean) => void
      }
    }
  | {
      type: 'PLAYLIST'
      args: {
        playlist: ExtendedPlaylist
        isRemote: boolean
        deleteCallback?: () => void
      }
    }
  | {
      type: 'GENERAL_PLAYLIST'
      args: {
        sort: Sort<PlaylistSortOptions>
        refreshCallback?: () => void
      }
    }
  | {
      type: 'QUEUE_ITEM'
      args: {
        isRemote: boolean
        refreshCallback: () => void
        song: Song
        songIndex: number
        sortOptions: Optional<Sort<SongSortOptions[]>, 'current'>
      }
    }
  | {
      type: 'ENTITY_SORT'
      args: {
        sortOptions: Sort<NormalSortOptions>
      }
    }
  | {
      type: 'PLAYLIST_SORT'
      args: {
        sortOptions: Sort<PlaylistSortOptions>
      }
    }
  | {
      type: 'ARTIST'
      args: {
        artist: Artists
        refreshCallback: () => void
      }
    }
  | {
      type: 'ALBUM'
      args: {
        album: Album
        refreshCallback: () => void
      }
    }
  | {
      type: 'SONG_SORT'
      args: {
        sortOptions: Sort<SongSortOptions[]>
      }
    }
  | {
      type: 'PLAYLIST_CONTENT'
      args: {
        exclude?: string
        refreshCallback?: () => void
        songs: Song[]
        isRemote?: boolean
        playlistId: string
      }
    }
  | {
      type: 'CURRENT_SONG'
      args: {
        song: Song
        isRemote?: boolean
      }
    }

type SongSortOptions = {
  type: 'title' | 'date_added' | 'playCount' | 'album' | 'artist' | 'albumartist' | 'genre' | 'track_no'
  asc: boolean
}
type PlaylistSortOptions = { type: 'name' | 'provider'; asc: boolean }
type NormalSortOptions = { type: 'name'; asc: boolean }

type Sort<T> = { callback: (options: T) => void; current: T }
