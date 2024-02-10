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
    <SongView :defaultDetails="defaultDetails" :songList="filteredSongList" :detailsButtonGroup="buttonGroups"
      :isRemote="isRemote" :isLoading="isLoading" @playAll="playAlbum" @addToQueue="addAlbumToQueue"
      @addToLibrary="addToLibrary" @onOptionalProviderChanged="onProviderChanged" :optionalProviders="albumSongProviders"
      @playRandom="playRandom" />
  </div>
</template>

<script lang="ts">
import { Component, Watch } from 'vue-facing-decorator'
import SongView from '@/mainWindow/components/songView/SongView.vue'

import { mixins } from 'vue-facing-decorator'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import RemoteSong from '@/utils/ui/mixins/remoteSongMixin'
import { ProviderScopes } from '@/utils/commonConstants'
import { emptyGen, getRandomFromArray } from '@/utils/common'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import ProviderFetchMixin from '@/utils/ui/mixins/ProviderFetchMixin'

@Component({
  components: {
    SongView
  }
})
export default class SingleAlbumView extends mixins(ContextMenuMixin, PlayerControls, RemoteSong, ProviderFetchMixin) {
  private album: Album | null = null

  // TODO: Find some better method to check if song is remote
  isRemote(songs: Song[]) {
    for (const s of songs) {
      for (const op of Object.values(this.optionalSongList)) {
        if (op.findIndex((val) => s._id === val) !== -1) {
          return true
        }
      }
    }
    return false
  }

  private fetchProviders() {
    const providers = this.getProvidersByScope(ProviderScopes.ALBUM_SONGS)
    return providers.map((val) => ({
      key: val.key,
      title: val.Title
    }))
  }

  get albumSongProviders(): TabCarouselItem[] {
    return this.fetchProviders()
  }

  get buttonGroups(): SongDetailButtons {
    return {
      enableContainer: true,
      enableLibraryStore: this.hasRemoteSongs,
      playRandom: this.filteredSongList.length > 150,
      fetchAll: this.hasNextPage()
    }
  }

  get defaultDetails(): SongDetailDefaults {
    return {
      defaultTitle: this.album?.album_name,
      defaultSubtitle: this.album?.album_artist,
      defaultSubSubtitle: this.$t('songView.details.songCount', this.filteredSongList.length),
      defaultCover: this.album?.album_coverPath_high
    }
  }

  get hasRemoteSongs() {
    return Object.keys(this.activeProviders).some((val) => val !== 'local' && this.activeProviders[val])
  }

  async created() {
    this.localSongFetch = async (sortBy) =>
      window.SearchUtils.searchSongsByOptions({
        album: {
          album_id: this.$route.query.id as string
        },
        sortBy
      })

    this.generator = (provider, nextPageToken) => {
      if (this.album) {
        return provider.getAlbumSongs(this.album, nextPageToken)
      } else {
        return emptyGen()
      }
    }
  }

  @Watch('$route.query.id')
  private async onAlbumChange() {
    const promises: Promise<void>[] = []

    if (!this.$route.query.id && this.$route.query.name) {
      const id = ((await window.SearchUtils.searchEntityByOptions({
        album: {
          album_name: this.$route.query.name.toString()
        }
      }))?.[0] as Album)?.album_id

      if (id)
        this.$route.query.id = id
    }

    if (typeof this.$route.query.id === 'string') {
      this.album = null
      this.clearNextPageTokens()
      this.clearSongList()
      promises.push(this.fetchAlbum())
      promises.push(this.fetchSongList())
    }
    await Promise.all(promises)
  }

  async mounted() {
    await this.onAlbumChange()

    if (this.$route.query.defaultProviders) {
      for (const p of this.$route.query.defaultProviders) {
        if (p) {
          this.onProviderChanged({ key: p, checked: true })
          bus.emit(EventBus.UPDATE_OPTIONAL_PROVIDER, p)
        }
      }
    }
  }

  private async fetchAlbum() {
    this.album = {
      album_id: this.$route.query.id as string,
      album_name: this.$route.query.name as string,
      album_coverPath_high: this.$route.query.cover_high as string,
      album_coverPath_low: this.$route.query.cover_low as string,
      album_artist: this.$route.query.album_artist as string,
      year: parseInt(this.$route.query.year as string),
      album_extra_info: JSON.parse((this.$route.query.extra_info as string) || '{}')
    }

    await this.fetchAlbumCover()
  }

  private async fetchAlbumCover() {
    if (this.album) {
      if (!(this.album.album_coverPath_high ?? this.album.album_coverPath_low) && this.album.album_name) {
        const providers = this.getProvidersByScope(ProviderScopes.SEARCH_ALBUM)
        for (const p of providers) {
          const res = (await p.searchAlbum(this.album.album_name))[0]
          if (res) {
            this.album.album_coverPath_high = res.album_coverPath_high
            this.album.album_coverPath_low = res.album_coverPath_low

            window.DBUtils.updateAlbum({
              ...this.album,
              album_coverPath_high: res.album_coverPath_high,
              album_coverPath_low: res.album_coverPath_low,
              album_extra_info: res.album_extra_info
            })

            return
          }
        }
      }
    }
  }

  playAlbum() {
    this.playTop(this.filteredSongList)
  }

  addAlbumToQueue() {
    this.queueSong(this.filteredSongList)
  }

  async playRandom() {
    const randomSongs = getRandomFromArray(this.filteredSongList, 100)
    this.queueSong(randomSongs)
  }

  addToLibrary() {
    this.addSongsToLibrary(...this.filteredSongList)
  }
}
</script>
