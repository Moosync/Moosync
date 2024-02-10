/*
 *  ContextMenuMixin.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { bus } from '@/mainWindow/main'
import { vxm } from '@/mainWindow/store'
import { EventBus } from '@/utils/preload/ipc/constants'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import RemoteSong from '@/utils/ui/mixins/remoteSongMixin'
import { Component } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import { toast } from 'vue3-toastify'
import { convertProxy } from '../common'
import JukeboxMixin from './JukeboxMixin'
import ProviderMixin from './ProviderMixin'
import RouterPushes from './RouterPushes'

export type MenuItem = {
  label?: string
  onClick?: () => void
  children?: MenuItem[]
}

@Component
export default class ContextMenuMixin extends mixins(
  PlayerControls,
  RemoteSong,
  JukeboxMixin,
  ProviderMixin,
  RouterPushes,
) {
  get playlists() {
    return vxm.playlist.playlists
  }

  private async addToPlaylist(playlist_id: string, songs: Song[]) {
    await window.DBUtils.addToPlaylist(playlist_id, ...songs.map((val) => convertProxy(val)))
  }

  private async removeAlbum(album: Album) {
    if (!album.album_id) {
      console.error('Failed to remove album, since album ID is empty')
      return
    }

    const albumSongs = await window.SearchUtils.searchSongsByOptions({
      album: {
        album_id: album.album_id,
      },
    })

    await window.DBUtils.removeSongs(albumSongs)
  }

  private async removeArtist(artist: Artists) {
    if (!artist.artist_id) {
      console.error('Failed to remove artist, since artist ID is empty')
      return
    }

    const artistSongs = await window.SearchUtils.searchSongsByOptions({
      artist: {
        artist_id: artist.artist_id,
      },
      inclusive: true,
    })

    await window.DBUtils.removeSongs(artistSongs)
  }

  private getSortIcon(currentType: SongSortOptions['type'], requiredType: SongSortOptions['type'], isAsc = true) {
    if (currentType === requiredType) {
      return isAsc ? '▲' : '▼'
    }
    return ''
  }

  private getSortonClick(type: SongSortOptions['type'], currentSort?: SongSortOptions): SongSortOptions[] {
    const ret: SongSortOptions[] = [
      {
        type,
        asc: currentSort?.type === type && !currentSort.asc,
      },
    ]

    if (type === 'album') {
      ret.push({
        type: 'track_no',
        asc: currentSort?.type === type && !currentSort.asc,
      })
    }

    return ret
  }

  private getSongSortByMenu(sort: Optional<Sort<SongSortOptions[]>, 'current'>) {
    const currentSort = sort?.current?.[0]
    const possibleSorts: SongSortOptions['type'][] = ['album', 'artist', 'date_added', 'genre', 'playCount', 'title']
    const menu: MenuItem[] = [
      {
        label: this.$t('contextMenu.sort_by'),
        children: [],
      },
    ]

    for (const p of possibleSorts) {
      menu[0].children?.push({
        label: `${this.$t(`contextMenu.sort.${p}`)} ${this.getSortIcon(
          currentSort?.type ?? 'title',
          p,
          currentSort?.asc,
        )}`,
        onClick: () => sort.callback(this.getSortonClick(p, currentSort)),
      })
    }
    return menu
  }

  private getPlaylistSortByMenu(sort: Sort<PlaylistSortOptions>) {
    const menu: MenuItem[] = [
      {
        label: this.$t('contextMenu.sort_by'),
        children: [
          {
            label: `${this.$t('contextMenu.sort.title')} ${sort.current.type === 'name' ? (sort.current.asc ? '▲' : '▼') : ''
              }`,
            onClick: () => sort.callback({ type: 'name', asc: sort.current.type === 'name' && !sort.current.asc }),
          },
          {
            label: `${this.$t('contextMenu.sort.provider')} ${sort.current.type === 'provider' ? (sort.current.asc ? '▲' : '▼') : ''
              }`,
            onClick: () =>
              sort.callback({ type: 'provider', asc: sort.current.type === 'provider' && !sort.current.asc }),
          },
        ],
      },
    ]
    return menu
  }

  private getGenericSortByMenu(sort: Sort<NormalSortOptions>) {
    const menu: MenuItem[] = [
      {
        label: this.$t('contextMenu.sort_by'),
        children: [
          {
            label: `${this.$t('contextMenu.sort.title')} ${sort.current.type === 'name' ? (sort.current.asc ? '▲' : '▼') : ''
              }`,
            onClick: () => sort.callback({ type: 'name', asc: sort.current.type === 'name' && !sort.current.asc }),
          },
        ],
      },
    ]
    return menu
  }

  private populatePlaylistMenu(item: Song[], exclude?: string): MenuItem[] {
    const menu: MenuItem[] = [
      {
        label: this.$t('contextMenu.playlist.new'),
        onClick: () => {
          bus.emit(EventBus.SHOW_NEW_PLAYLIST_MODAL, item)
        },
      },
    ]
    for (const [key, val] of Object.entries(this.playlists)) {
      if (key === exclude) {
        continue
      }
      menu.push({
        label: val,
        onClick: () => {
          this.addToPlaylist(key, item)
        },
      })
    }
    return menu
  }

  private getGeneralSongsContextMenu(
    refreshCallback?: (showHidden?: boolean) => void,
    showHiddenToggle?: boolean,
    isShowingHidden?: boolean,
    sort?: Sort<SongSortOptions[]>,
  ) {
    const items: MenuItem[] = []

    if (sort) {
      items.push(...this.getSongSortByMenu(sort))
    }

    if (refreshCallback) {
      items.push({
        label: this.$t('contextMenu.song.addFromURL'),
        onClick: () => bus.emit(EventBus.SHOW_SONG_FROM_URL_MODAL, refreshCallback),
      })

      if (showHiddenToggle) {
        items.push({
          label: isShowingHidden ? this.$t('contextMenu.song.hideHidden') : this.$t('contextMenu.song.showHidden'),
          onClick: () => refreshCallback(!isShowingHidden),
        })
      }
    }

    return items
  }

  private async getPlaylistSongContextMenu(
    playlistId: string,
    exclude: string | undefined,
    refreshCallback?: () => void,
    isRemote = false,
    ...item: Song[]
  ) {
    const items: MenuItem[] = [...(await this.getSongContextMenu(exclude, refreshCallback, isRemote, ...item))]

    if (!isRemote) {
      items.splice(4, 0, {
        label: this.$t('contextMenu.song.removeFromPlaylist'),
        onClick: async () => {
          await window.DBUtils.removeFromPlaylist(playlistId, ...convertProxy(item))
          refreshCallback?.()
        },
      })
    }

    return items
  }

  private async openInBrowser(song: Song) {
    const provider = this.getProviderBySong(song)

    const url = await provider?.getRemoteURL(song)
    if (url) window.WindowUtils.openExternal(url)
    else toast(`No URL found for ${song.title}`, { type: 'error' })
  }

  private addGotoAlbumArtist(item: Song) {
    const items: MenuItem[] = []
    const album = item?.album
    if (album) {
      items.push({
        label: this.$t('contextMenu.song.gotoAlbum', { title: album.album_name ?? '' }),
        onClick: () => this.gotoAlbum(album),
      })
    }

    const artists = item?.artists
    if (artists?.length) {
      const artistItems: MenuItem[] = artists.map((val) => ({
        label: val.artist_name,
        onClick: () => this.gotoArtist(val),
      }))

      items.push({
        label: this.$t('contextMenu.song.gotoArtists'),
        children: artistItems,
      })
    }

    return items
  }

  private async getSongContextMenu(
    exclude: string | undefined,
    refreshCallback?: () => void,
    isRemote = false,
    ...item: Song[]
  ) {
    const items: MenuItem[] = [
      {
        label: this.$t('contextMenu.song.playNow'),
        onClick: () => {
          this.playTop(item)
        },
      },
      {
        label: this.$t('contextMenu.song.playNext'),
        onClick: () => {
          this.playNext(item)
        },
      },
      {
        label: this.$t('contextMenu.song.clearAndPlay'),
        onClick: () => {
          this.clearQueue()
          this.playTop(item)
        },
      },
      {
        label: this.$t('contextMenu.song.addToQueue'),
        onClick: () => {
          this.queueSong(item)
        },
      },
      {
        label: this.$t('contextMenu.playlist.add'),
        children: this.populatePlaylistMenu(item, exclude),
      },
    ]

    if (!isRemote) {
      items.push(
        ...[
          {
            label: this.$t('contextMenu.song.remove'),
            onClick: async () => {
              try {
                await window.DBUtils.removeSongs(convertProxy(item))
              } catch (e) {
                console.error(e)
              }
              refreshCallback?.()
            },
          },
          {
            label: this.$t('contextMenu.song.addFromURL'),
            onClick: () => bus.emit(EventBus.SHOW_SONG_FROM_URL_MODAL, refreshCallback),
          },
        ],
      )
    } else {
      items.push({
        label: this.$t('contextMenu.song.add', item.length),
        onClick: () => this.addSongsToLibrary(...item),
      })

      items.push({
        label: this.$t('contextMenu.song.openInBrowser'),
        onClick: () => this.openInBrowser(item[0]),
      })
    }

    const songInLibrary = item.find((val) => typeof val.showInLibrary === 'boolean')
    if (songInLibrary) {
      const shownInLibrary = songInLibrary.showInLibrary
      items.push({
        label: shownInLibrary ? this.$t('contextMenu.song.hideFromLibrary') : this.$t('contextMenu.song.showInLibrary'),
        onClick: async () => {
          await window.DBUtils.updateSongs(
            item.map((val) => ({ ...convertProxy(val), showInLibrary: !shownInLibrary })),
          )
          refreshCallback?.()
        },
      })
    }

    item[0] && items.push(...this.addGotoAlbumArtist(item[0]))

    items.push({
      label: this.$t('contextMenu.moreInfo'),
      onClick: () => {
        bus.emit(EventBus.SHOW_SONG_INFO_MODAL, item[0])
      },
    })

    return items
  }

  private getPlaylistContextMenu(playlist: ExtendedPlaylist, isRemote: boolean, deleteCallback?: () => void) {
    const items = []
    if (!isRemote) {
      items.push({
        label: this.$t('contextMenu.playlist.remove'),
        onClick: () => {
          deleteCallback?.()
        },
      })

      items.push({
        label: this.$t('contextMenu.playlist.export'),
        onClick: () => {
          window.DBUtils.exportPlaylist(convertProxy(playlist))
        },
      })

      items.push(this.getEntityInfoMenu(playlist))
    } else {
      items.push({
        label: this.$t('contextMenu.playlist.save'),
        onClick: async () => {
          await window.DBUtils.createPlaylist(convertProxy(playlist))
          toast(`Added ${playlist.playlist_name} to library`)
        },
      })
    }

    return items
  }

  private getGeneralPlaylistMenu(sort: Sort<PlaylistSortOptions>, refreshCallback?: () => void) {
    const items = [
      {
        label: this.$t('contextMenu.playlist.addFromURL'),
        onClick: () => {
          bus.emit(EventBus.SHOW_PLAYLIST_FROM_URL_MODAL, refreshCallback)
        },
      },
      ...this.getPlaylistSortByMenu(sort),
    ]
    return items
  }

  private getQueueItemMenu(
    isRemote: boolean,
    refreshCallback: () => void,
    item: Song,
    itemIndex: number,
    sort: Optional<Sort<SongSortOptions[]>, 'current'>,
  ) {
    const items = [
      {
        label: this.$t('contextMenu.playlist.add'),
        children: this.populatePlaylistMenu([item], undefined),
      },
      {
        label: this.$t('contextMenu.song.moveToTop'),
        onClick: () => {
          this.setSongIndex(itemIndex, 0)
        },
      },
      {
        label: this.$t('contextMenu.song.moveToBottom'),
        onClick: () => {
          this.setSongIndex(itemIndex, -1)
        },
      },
      {
        label: this.$t('contextMenu.song.moveManually'),
        onClick: () => {
          bus.emit(EventBus.SHOW_FORM_MODAL, this.$t('contextMenu.song.manualIndex'), (value: number) => {
            this.setSongIndex(itemIndex, value)
          })
        },
      },
      ...this.getSongSortByMenu(sort),
    ]

    if (isRemote) {
      items.push({
        label: this.$t('contextMenu.song.add'),
        onClick: () => this.addSongsToLibrary(item),
      })

      items.push({
        label: this.$t('contextMenu.song.openInBrowser'),
        onClick: () => this.openInBrowser(item),
      })
    } else {
      items.push({
        label: this.$t('contextMenu.song.remove'),
        onClick: async () => {
          try {
            await window.DBUtils.removeSongs([convertProxy(item)])
          } catch (e) {
            console.error(e)
          }
          refreshCallback()
        },
      })
    }

    items.push(...this.addGotoAlbumArtist(item))

    if (item.type === 'YOUTUBE' || item.type === 'SPOTIFY') {
      items.push({
        label: this.$t('contextMenu.incorrectPlayback'),
        onClick: () => {
          bus.emit(EventBus.SHOW_INCORRECT_PLAYBACK_MODAL, item)
        },
      })
    }

    items.push({
      label: this.$t('contextMenu.moreInfo'),
      onClick: () => {
        bus.emit(EventBus.SHOW_SONG_INFO_MODAL, item)
      },
    })

    return items
  }

  private getEntityInfoMenu(entity: Album | Artists | Playlist) {
    return {
      label: this.$t('contextMenu.moreInfo'),
      onClick: () => {
        bus.emit(EventBus.SHOW_ENTITY_INFO_MODAL, entity)
      },
    }
  }

  private getArtistContextMenu(artist: Artists, refreshCallback: () => void) {
    const items = [this.getEntityInfoMenu(artist)]
    items.push({
      label: this.$t('contextMenu.artist.remove'),
      onClick: async () => {
        await this.removeArtist(artist)
        refreshCallback()
      },
    })
    return items
  }

  private getAlbumContextMenu(album: Album, refreshCallback: () => void) {
    const items = [this.getEntityInfoMenu(album)]
    items.push({
      label: this.$t('contextMenu.album.remove'),
      onClick: async () => {
        await this.removeAlbum(album)
        refreshCallback()
      },
    })
    return items
  }

  private getCurrentSongContextMenu(song: Song, isRemote?: boolean) {
    const items: MenuItem[] = [
      {
        label: this.$t('contextMenu.playlist.add'),
        children: this.populatePlaylistMenu([song]),
      },
      {
        label: this.$t('contextMenu.song.openInBrowser'),
        onClick: () => this.openInBrowser(song),
      },
    ]

    items.push(...this.addGotoAlbumArtist(song))

    if (isRemote) {
      items.push({
        label: this.$t('contextMenu.song.add', 1),
        onClick: () => this.addSongsToLibrary(song),
      })
    }

    if (song.type === 'YOUTUBE' || song.type === 'SPOTIFY') {
      items.push({
        label: this.$t('contextMenu.incorrectPlayback'),
        onClick: () => {
          bus.emit(EventBus.SHOW_INCORRECT_PLAYBACK_MODAL, song)
        },
      })
    }

    items.push({
      label: this.$t('contextMenu.moreInfo'),
      onClick: () => {
        bus.emit(EventBus.SHOW_SONG_INFO_MODAL, song)
      },
    })

    return items
  }

  public async getContextMenu(event: MouseEvent, options: ContextMenuArgs) {
    let items: MenuItem[] = []
    switch (options.type) {
      case 'SONGS':
        items = await this.getSongContextMenu(
          options.args.exclude,
          options.args.refreshCallback,
          options.args.isRemote,
          ...options.args.songs,
        )
        break
      case 'GENERAL_SONGS':
        items = this.getGeneralSongsContextMenu(
          options.args.refreshCallback,
          options.args.showHiddenToggle,
          options.args.isShowingHidden,
          options.args.sortOptions,
        )
        break
      case 'PLAYLIST':
        items = this.getPlaylistContextMenu(options.args.playlist, options.args.isRemote, options.args.deleteCallback)
        break
      case 'ALBUM':
        items = this.getAlbumContextMenu(options.args.album, options.args.refreshCallback)
        break
      case 'ARTIST':
        items = this.getArtistContextMenu(options.args.artist, options.args.refreshCallback)
        break
      case 'GENERAL_PLAYLIST':
        items = this.getGeneralPlaylistMenu(options.args.sort, options.args.refreshCallback)
        break
      case 'QUEUE_ITEM':
        items = this.getQueueItemMenu(
          options.args.isRemote,
          options.args.refreshCallback,
          options.args.song,
          options.args.songIndex,
          options.args.sortOptions,
        )
        break
      case 'ENTITY_SORT':
        items = this.getGenericSortByMenu(options.args.sortOptions)
        break
      case 'SONG_SORT':
        items = this.getSongSortByMenu(options.args.sortOptions)[0].children ?? []
        break
      case 'PLAYLIST_SORT':
        items = this.getPlaylistSortByMenu(options.args.sortOptions)
        break
      case 'PLAYLIST_CONTENT':
        items = await this.getPlaylistSongContextMenu(
          options.args.playlistId,
          options.args.exclude,
          options.args.refreshCallback,
          options.args.isRemote,
          ...options.args.songs,
        )
        break
      case 'CURRENT_SONG':
        items = this.getCurrentSongContextMenu(options.args.song, options.args.isRemote)
        break
    }

    items.push(...(await this.getExtensionItems(options.type, this.getExtensionArgs(options))))
    this.emitMenu(event, items)
  }

  private extensionContextMenuItems: ContextMenuTypes[] = [
    'SONGS',
    'ALBUM',
    'ARTIST',
    'GENERAL_PLAYLIST',
    'GENERAL_SONGS',
    'PLAYLIST',
    'PLAYLIST_CONTENT',
    'QUEUE_ITEM',
    'CURRENT_SONG',
  ]

  private getExtensionArgs(args: ContextMenuArgs): ExtensionContextMenuHandlerArgs<ContextMenuTypes> {
    switch (args.type) {
      case 'ALBUM':
        return args.args.album
      case 'ARTIST':
        return args.args.artist
      case 'PLAYLIST':
        return args.args.playlist
      case 'QUEUE_ITEM':
      case 'CURRENT_SONG':
        return args.args.song
      case 'SONGS':
        return args.args.songs
      default:
        return
    }
  }

  private async getExtensionItems(
    type: ContextMenuArgs['type'],
    arg: ExtensionContextMenuHandlerArgs<ContextMenuTypes>,
  ): Promise<MenuItem[]> {
    if (this.extensionContextMenuItems.includes(type as ContextMenuTypes)) {
      const items: (ExtendedExtensionContextMenuItems<ContextMenuTypes> & MenuItem)[] =
        await window.ExtensionUtils.getContextMenuItems(type as ContextMenuTypes)
      for (const i of items) {
        i.onClick = () => window.ExtensionUtils.fireContextMenuHandler(i.id, i.packageName, convertProxy(arg))
      }

      return items as MenuItem[]
    }

    return []
  }

  private emitMenu(event: MouseEvent, items: MenuItem[]) {
    if (!this.isJukeboxModeActive) {
      this.$contextmenu({
        x: event.x,
        y: event.y,
        customClass: 'context-menu',
        items,
      })
    }
  }
}
