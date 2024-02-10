<!-- 
  _id.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<!-- <route>
{
  "props": true
}
</route> -->
<template>
  <div class="w-100 h-100">
    <SongView :defaultDetails="defaultDetails" :songList="songList" :detailsButtonGroup="buttonGroups"
      :isLoading="isLoading" :isRemote="isRemote" :onSongContextMenuOverride="onSongContextMenuOverride"
      @playAll="playPlaylist" @addToQueue="addPlaylistToQueue" @addToLibrary="addPlaylistToLibrary"
      @onScrollEnd="loadNextPage" @onSearchChange="onSearchChange" @playRandom="playRandom" @fetchAll="fetchAll" />
  </div>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import SongView from '@/mainWindow/components/songView/SongView.vue'

import { mixins } from 'vue-facing-decorator'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import { arrayDiff, emptyGen, getRandomFromArray } from '@/utils/common'
import ProviderFetchMixin from '@/utils/ui/mixins/ProviderFetchMixin'
import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider'
import { ProviderScopes } from '@/utils/commonConstants'
import { toast } from 'vue3-toastify'
import { FAVORITES_PLAYLIST_ID } from '../../../utils/commonConstants'

@Component({
  components: {
    SongView
  }
})
export default class SinglePlaylistView extends mixins(ContextMenuMixin, ProviderFetchMixin) {
  @Prop({ default: () => () => undefined })
  private enableRefresh!: () => void

  private isAddedInLibrary = false

  private invalidateCache = false

  private providers: GenericProvider[] = []

  private getPlaylistOwnerProvider() {
    if (this.playlist?.playlist_id) {
      for (const p of this.providers) {
        if (p.matchEntityId(this.playlist?.playlist_id)) {
          return p
        }
      }
    }
  }

  get buttonGroups(): SongDetailButtons {
    return {
      enableContainer: true,
      enableLibraryStore: this.isRemote() && !this.isAddedInLibrary,
      playRandom: !!(this.filteredSongList.length > 150),
      fetchAll: this.isRemote()
    }
  }

  get defaultDetails(): SongDetailDefaults {
    return {
      defaultTitle: this.playlist?.playlist_name ?? '',
      defaultSubSubtitle: this.$t('songView.details.songCount', this.filteredSongList.length),
      defaultCover:
        this.playlist.playlist_id === FAVORITES_PLAYLIST_ID
          ? FAVORITES_PLAYLIST_ID
          : this.playlist?.playlist_coverPath ?? ''
    }
  }

  private get isYoutube() {
    return (this.$route?.query?.id as string)?.startsWith('youtube')
  }

  private get isSpotify() {
    return (this.$route?.query?.id as string)?.startsWith('spotify')
  }

  private get isExtension() {
    return this.$route?.query?.extension
  }

  isRemote() {
    return !!(this.isYoutube || this.isSpotify || this.isExtension)
  }

  private async clearAndFetch() {
    this.clearSongList()
    this.clearNextPageTokens()

    await this.fetchSongList()
  }

  private async refresh() {
    for (const [key, checked] of Object.entries(this.activeProviders)) {
      if (checked) {
        await this.onProviderChanged({ key, checked })
      }
    }
  }

  async created() {
    this.providers = this.getProvidersByScope(ProviderScopes.PLAYLIST_SONGS)

    this.localSongFetch = async (sortBy: SongSortOptions[]) =>
      window.SearchUtils.searchSongsByOptions({
        playlist: {
          playlist_id: this.$route?.query?.playlist_id as string
        },
        sortBy
      })

    this.generator = (provider, nextPageToken) => {
      if (this.playlist) {
        return provider.getPlaylistContent(
          provider.sanitizeId(this.playlist.playlist_id, 'PLAYLIST'),
          this.invalidateCache,
          nextPageToken
        )
      } else {
        return emptyGen()
      }
    }

    this.isAddedInLibrary = !!(
      await window.SearchUtils.searchEntityByOptions<Playlist>({
        playlist: { playlist_id: this.playlist.playlist_id }
      })
    )[0]

    const owner = this.getPlaylistOwnerProvider()
    if (owner) {
      this.onProviderChanged({ key: owner?.key, checked: true })
    }
  }

  mounted() {
    this.enableRefresh()
    this.listenGlobalRefresh()
    this.clearAndFetch()
  }

  private listenGlobalRefresh() {
    bus.on(EventBus.REFRESH_PAGE, async () => {
      this.invalidateCache = true
      await this.clearAndFetch()
      await this.refresh()
    })
  }

  private get playlist() {
    return {
      playlist_id: this.$route?.query?.playlist_id as string,
      playlist_name: this.$route?.query?.playlist_name as string,
      playlist_coverPath: this.$route?.query?.playlist_coverPath as string,
      playlist_song_count: parseInt(this.$route?.query?.playlist_song_count as string) ?? 0,
      playlist_path: this.$route?.query?.playlist_path as string | undefined,
      extension: this.$route?.query?.extension as string | undefined
    }
  }

  async playPlaylist() {
    const clearQueue = (await window.PreferenceUtils.loadSelectiveArrayItem<Checkbox>('queue.clear_queue_playlist'))
      ?.enabled
    if (clearQueue) {
      this.clearQueue()
    }

    this.playTop(this.filteredSongList)
    this.fetchAll((songs) => this.queueSong(songs, false), this.showQueueSongsToast)
  }

  async addPlaylistToQueue() {
    this.queueSong(this.filteredSongList)
    this.fetchAll((songs) => this.queueSong(songs, false), this.showQueueSongsToast)
  }

  addPlaylistToLibrary() {
    window.DBUtils.createPlaylist(this.playlist)
    toast(`Added ${this.playlist.playlist_name} to library`)
  }

  onSearchChange() {
    this.fetchAll()
  }

  async playRandom() {
    await this.fetchAll()
    const randomSongs = getRandomFromArray(this.filteredSongList, 100)
    this.queueSong(randomSongs)
  }

  onSongContextMenuOverride(event: PointerEvent, songs: Song[]) {
    this.getContextMenu(event, {
      type: 'PLAYLIST_CONTENT',
      args: {
        playlistId: this.playlist.playlist_id,
        songs,
        isRemote: this.isRemote(),
        refreshCallback: () =>
          this.filteredSongList.splice(0, this.filteredSongList.length, ...arrayDiff(this.filteredSongList, songs))
      }
    })
  }
}
</script>
