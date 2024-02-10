<!-- 
  _id.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
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
    <SongView
      :defaultDetails="defaultDetails"
      :songList="filteredSongList"
      :isLoading="isLoading"
      :isRemote="isRemote"
      @playAll="playArtist"
      @addToQueue="addArtistToQueue"
      @addToLibrary="addArtistToLibrary"
      @onOptionalProviderChanged="onProviderChanged"
      :detailsButtonGroup="buttonGroups"
      :optionalProviders="artistSongProviders"
      @onScrollEnd="loadNextPage"
      @onSearchChange="onSearchChange"
      @playRandom="playRandom"
    />
  </div>
</template>

<script lang="ts">
import { Component, Watch } from 'vue-facing-decorator'
import SongView from '@/mainWindow/components/songView/SongView.vue'

import { mixins } from 'vue-facing-decorator'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import RemoteSong from '@/utils/ui/mixins/remoteSongMixin'
import { emptyGen, getRandomFromArray } from '@/utils/common'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import ProviderFetchMixin from '../../../utils/ui/mixins/ProviderFetchMixin'
import { ProviderScopes } from '@/utils/commonConstants'
import { convertProxy } from '@/utils/ui/common'

@Component({
  components: {
    SongView
  }
})
export default class SingleArtistView extends mixins(ContextMenuMixin, RemoteSong, ProviderFetchMixin) {
  private artist: Artists | null = null

  get artistSongProviders(): TabCarouselItem[] {
    return this.fetchProviders()
  }

  fetchProviders() {
    const providers = this.getProvidersByScope(ProviderScopes.ARTIST_SONGS)
    return providers.map((val) => ({
      title: val.Title,
      key: val.key
    }))
  }

  get buttonGroups(): SongDetailButtons {
    return {
      enableContainer: true,
      enableLibraryStore: true,
      playRandom: !!(this.filteredSongList.length >= 150),
      fetchAll: this.hasNextPage()
    }
  }

  get defaultDetails(): SongDetailDefaults {
    return {
      defaultTitle: this.artist?.artist_name,
      defaultSubSubtitle: this.$t('songView.details.songCount', this.filteredSongList.length),
      defaultCover: this.artist?.artist_coverPath
    }
  }

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

  created() {
    this.localSongFetch = async (sortBy) =>
      window.SearchUtils.searchSongsByOptions({
        artist: {
          artist_id: this.$route.query.id as string
        },
        sortBy
      })

    this.generator = (provider, nextPageToken) => {
      if (this.artist) {
        return provider.getArtistSongs(this.artist, nextPageToken)
      } else {
        return emptyGen()
      }
    }
  }

  @Watch('$route.query.id')
  private async onArtistChange() {
    if (typeof this.$route.query.id === 'string') {
      this.artist = null
      this.clearNextPageTokens()
      this.clearSongList()
      await this.fetchArtists()
      await this.fetchSongList()
    }
  }

  async mounted() {
    await this.onArtistChange()
    if (this.$route.query.defaultProviders) {
      for (const p of this.$route.query.defaultProviders) {
        if (p) {
          this.onProviderChanged({ key: p, checked: true })
          bus.emit(EventBus.UPDATE_OPTIONAL_PROVIDER, p)
        }
      }
    }
  }

  private async fetchArtists() {
    this.artist = (
      await window.SearchUtils.searchEntityByOptions<Artists>({
        artist: {
          artist_id: convertProxy(this.$route.query.id as string)
        }
      })
    )[0]

    if (!this.artist?.artist_name) {
      this.artist = {
        artist_id: this.$route.query.id as string,
        artist_name: this.$route.query.name as string,
        artist_coverPath: this.$route.query.cover as string,
        artist_extra_info: JSON.parse((this.$route.query.extra_info as string) || '{}')
      }
    }

    if (!this.artist.artist_coverPath) {
      const fetchedArtist = await this.fetchRemoteArtistDetails(this.artist)
      this.artist = {
        ...this.artist,
        artist_coverPath: fetchedArtist?.artist_coverPath
      }

      await window.DBUtils.updateArtist(convertProxy(this.artist, true))
    }
  }

  playArtist() {
    this.playTop(this.filteredSongList)
    this.fetchAll((songs) => this.queueSong(songs, false), this.showQueueSongsToast)
  }

  addArtistToQueue() {
    this.queueSong(this.filteredSongList)
    this.fetchAll((songs) => this.queueSong(songs, false), this.showQueueSongsToast)
  }

  addArtistToLibrary() {
    this.addSongsToLibrary(...this.filteredSongList)
    this.fetchAll((songs) => this.addSongsToLibrary(...songs))
  }

  async playRandom() {
    await this.fetchAll()
    const randomSongs = getRandomFromArray(this.filteredSongList, 100)
    this.queueSong(randomSongs)
  }

  onSearchChange() {
    this.fetchAll()
  }
}
</script>
