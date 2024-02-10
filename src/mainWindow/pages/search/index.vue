<!-- 
  index.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="w-100 h-100">
    <b-container fluid class="h-100 w-100 search-container">
      <TabCarousel
        :items="providers"
        :showExtraSongListActions="false"
        :singleSelectMode="true"
        :defaultSelected="activeProvider"
        :showBackgroundOnSelect="true"
        @onItemsChanged="onProviderChanged"
      />

      <TabCarousel
        :items="subCategories"
        :showExtraSongListActions="false"
        :singleSelectMode="true"
        :defaultSelected="activeSubcategory"
        :showBackgroundOnSelect="true"
        @onItemsChanged="onSubcategoriesChanged"
      />

      <div v-if="!isFetching">
        <transition
          appear
          name="custom-slide-fade"
          :enter-active-class="`animate__animated ${transitionEnterActiveClass} animate__fast`"
          :leave-active-class="`animate__animated ${transitionExitActiveClass} animate__fast`"
        >
          <b-row
            class="scroller-row w-100"
            v-if="activeSubcategory === 'songs'"
            :key="`${activeProvider}-${activeSubcategory}`"
          >
            <b-col class="h-100">
              <RecycleScroller
                class="scroller w-100 h-100"
                :items="currentSongList"
                :item-size="94"
                key-field="_id"
                :direction="'vertical'"
              >
                <template v-slot="{ item, index }">
                  <SongListCompactItem
                    :item="item"
                    :index="index"
                    :selected="selected"
                    @onRowDoubleClicked="queueSong([$event])"
                    @onRowSelected="onRowSelected"
                    @onRowContext="onRowContext"
                    @onPlayNowClicked="playTop([$event])"
                    @onArtistClicked="gotoArtist"
                  />
                </template>
              </RecycleScroller>
            </b-col>
          </b-row>

          <b-row class="scroller-row w-100" v-else :key="`${activeProvider}-${activeSubcategory}-else`">
            <b-col col xl="2" md="3" v-for="entity in currentEntityList" :key="(entity as any)[entityKeyField]">
              <CardView
                @click="onCardClick(entity)"
                :title="(entity as any)[entityTitleField]"
                :imgSrc="(entity as any)[entityImageField]"
                @CardContextMenu="onCardContextMenu($event, entity)"
              >
                <template #defaultCover> <component :is="defaultCoverComponent" /></template>
              </CardView>
            </b-col>
          </b-row>
        </transition>
      </div>
      <div v-else>
        <b-spinner label="Loading..."></b-spinner>
      </div>
      <div v-if="noResults" class="no-results">{{ noResultsReason }}</div>
    </b-container>
  </div>
</template>

<script lang="ts">
import { vxm } from '@/mainWindow/store'
import { Component, Watch } from 'vue-facing-decorator'
import TabCarousel from '@/mainWindow/components/generic/TabCarousel.vue'
import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider'
import SongListCompactItem from '@/mainWindow/components/songView/components/SongListCompactItem.vue'
import CardView from '@/mainWindow/components/generic/CardView.vue'
import ArtistDefault from '@/icons/ArtistDefaultIcon.vue'
import AlbumDefault from '@/icons/AlbumDefaultIcon.vue'
import PlaylistDefault from '@/icons/PlaylistDefaultIcon.vue'
import GenreDefault from '@/icons/SongDefaultIcon.vue'
import { mixins } from 'vue-facing-decorator'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import SongListMixin from '@/utils/ui/mixins/SongListMixin'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import RouterPushes from '@/utils/ui/mixins/RouterPushes'
import ProviderMixin from '@/utils/ui/mixins/ProviderMixin'
import { ProviderScopes } from '@/utils/commonConstants'
import { YoutubeAlts } from '@/mainWindow/store/providers'

@Component({
  components: {
    TabCarousel,
    SongListCompactItem,
    CardView,
    ArtistDefault,
    AlbumDefault,
    PlaylistDefault,
    GenreDefault
  }
})
export default class SearchPage extends mixins(
  PlayerControls,
  SongListMixin,
  ContextMenuMixin,
  RouterPushes,
  ProviderMixin
) {
  private oldSubcategory = this.activeSubcategory

  private fetchMap: Record<string, boolean> = {}

  get isFetching() {
    return this.fetchMap[this.activeProvider]
  }

  get activeProvider() {
    return vxm.themes.lastSearchTab[0]
  }

  set activeProvider(item: string) {
    vxm.themes.lastSearchTab = [item, this.activeSubcategory]
  }

  get activeSubcategory() {
    return vxm.themes.lastSearchTab[1]
  }

  set activeSubcategory(item: keyof SearchResult) {
    vxm.themes.lastSearchTab = [this.activeProvider, item]
  }

  get noResultsReason() {
    if (this.activeProvider === vxm.providers.youtubeProvider.key) {
      if (vxm.providers.youtubeAlt === YoutubeAlts.INVIDIOUS) {
        if (this.activeSubcategory === 'albums') {
          return 'Searching albums is currently not supported using Invidious'
        }
      } else if (vxm.providers.youtubeAlt === YoutubeAlts.YOUTUBE) {
        if (this.activeSubcategory === 'albums') {
          return 'Searching albums is currently not supported for Youtube'
        }
      }
    } else if (this.activeProvider === vxm.providers.spotifyProvider.key && !vxm.providers.spotifyProvider.loggedIn) {
      return 'Login to Spotify to use this feature'
    } else if (this.activeProvider !== 'local') {
      return 'Nothing found'
    }

    return 'Nothing found'
  }

  get noResults() {
    if (!this.isFetching) {
      if (this.activeSubcategory === 'songs') {
        return this.currentSongList.length === 0
      }
      return this.currentEntityList.length === 0
    }
    return false
  }

  get defaultCoverComponent() {
    switch (this.activeSubcategory) {
      case 'artists':
        return 'ArtistDefault'
      case 'playlists':
        return 'PlaylistDefault'
      case 'albums':
        return 'AlbumDefault'
      case 'genres':
      default:
        return 'GenreDefault'
    }
  }

  private fetchProviders() {
    const parsedProviders: TabCarouselItem[] = []
    parsedProviders.push({
      title: 'Local',
      key: 'local',
      defaultChecked: this.activeProvider === 'local'
    })

    const providers = this.getProvidersByScope(ProviderScopes.SEARCH)
    parsedProviders.push(
      ...providers.map((val) => ({
        title: val.Title,
        key: val.key,
        defaultChecked: this.activeProvider === val.key
      }))
    )

    return parsedProviders
  }

  get providers() {
    return this.fetchProviders()
  }

  get subCategories(): TabCarouselItem[] {
    const subCategories: TabCarouselItem[] = [
      {
        title: 'Songs',
        key: 'songs'
      },
      {
        title: 'Artists',
        key: 'artists'
      },
      {
        title: 'Playlists',
        key: 'playlists'
      },
      {
        title: 'Albums',
        key: 'albums'
      }
    ]

    for (const s of subCategories) {
      s.defaultChecked = this.activeSubcategory === s.key
    }

    return subCategories
  }

  get currentSongList() {
    if (this.activeProvider) {
      return this.results[this.activeProvider]?.songs ?? []
    }

    return []
  }

  get currentEntityList() {
    if (this.activeProvider) {
      const providerResults = this.results[this.activeProvider]
      if (providerResults) {
        return providerResults[this.activeSubcategory] ?? []
      }
    }
    return []
  }

  get entityKeyField() {
    switch (this.activeSubcategory) {
      default:
      case 'songs':
        return '_id'
      case 'artists':
        return 'artist_id'
      case 'playlists':
        return 'playlist_id'
      case 'albums':
        return 'album_id'
      case 'genres':
        return 'genre_id'
    }
  }

  get entityTitleField() {
    switch (this.activeSubcategory) {
      default:
      case 'songs':
        return 'title'
      case 'artists':
        return 'artist_name'
      case 'playlists':
        return 'playlist_name'
      case 'albums':
        return 'album_name'
      case 'genres':
        return 'genre_name'
    }
  }

  get entityImageField() {
    switch (this.activeSubcategory) {
      default:
      case 'songs':
        return 'song_coverPath_high'
      case 'artists':
        return 'artist_coverPath'
      case 'playlists':
        return 'playlist_coverPath'
      case 'albums':
        return 'album_coverPath_high'
      case 'genres':
        return 'genre_coverPath'
    }
  }

  transitionEnterActiveClass = 'animate__slideInLeft'
  transitionExitActiveClass = 'animate__slideOutLeft'

  private results: Record<string, SearchResult> = {}

  private get searchTerm() {
    return this.$route.query.search_term as string
  }

  @Watch('searchTerm', { immediate: true })
  private onSearchTermChanged(val: string) {
    if (val) {
      this.fetchLocalSongList()

      for (const p of this.providers) {
        const provider = this.getProviderByKey(p.key)
        if (provider) {
          this.fetchProviderSongList(provider)
        }
      }
    }
  }

  private async fetchLocalSongList() {
    this.fetchMap['local'] = true
    this.results['local'] = await window.SearchUtils.searchAll(`%${this.searchTerm}%`)
    this.fetchMap['local'] = false
  }

  private async fetchProviderSongList(provider: GenericProvider) {
    this.fetchMap[provider.key] = true

    try {
      this.results[provider.key] = {
        songs: await provider.searchSongs(this.searchTerm),
        artists: await provider.searchArtists(this.searchTerm),
        playlists: await provider.searchPlaylists(this.searchTerm),
        albums: await provider.searchAlbum(this.searchTerm),
        genres: []
      }
    } catch (e) {
      console.error(e)
    }
    this.fetchMap[provider.key] = false
  }

  onProviderChanged({ key, checked }: { key: string; checked: boolean }) {
    if (checked) this.activeProvider = key
  }

  onSubcategoriesChanged({ key, checked }: { key: string; checked: boolean }) {
    if (checked) {
      this.activeSubcategory = key as keyof SearchResult

      const oldIndex = this.subCategories.findIndex((val) => val.key === this.oldSubcategory)
      const newIndex = this.subCategories.findIndex((val) => val.key === this.activeSubcategory)

      this.oldSubcategory = this.activeSubcategory

      if (oldIndex < newIndex) {
        this.transitionEnterActiveClass = 'animate__slideInRight'
        this.transitionExitActiveClass = 'animate__slideOutLeft'
      } else {
        this.transitionEnterActiveClass = 'animate__slideInLeft'
        this.transitionExitActiveClass = 'animate__slideOutRight'
      }
    }
  }

  onRowContext(event: PointerEvent, item: Song) {
    this.getContextMenu(event, {
      type: 'SONGS',
      args: { songs: [item], isRemote: this.activeProvider !== 'local' }
    })
  }

  onCardClick(item: (typeof this.currentEntityList)[0]) {
    switch (this.activeSubcategory) {
      case 'artists':
        this.gotoArtist(item as Artists, [this.activeProvider])
        break
      case 'playlists':
        this.gotoPlaylist(item as Playlist)
        break
      case 'albums':
        this.gotoAlbum(item as Album, [this.activeProvider])
        break
      case 'genres':
        this.gotoGenre(item as Genre)
        break
    }
  }

  onCardContextMenu(event: PointerEvent, item: (typeof this.currentEntityList)[0]) {
    switch (this.activeSubcategory) {
      case 'playlists':
        this.getContextMenu(event, {
          type: 'PLAYLIST',
          args: { playlist: item as ExtendedPlaylist, isRemote: this.activeProvider !== 'local' }
        })
        break
      default:
        break
    }
  }
}
</script>

<style lang="sass" scoped>
.scroller-row
  position: absolute
  height: calc(100% - 140px)
  overflow: auto

.no-results
  font-size: 18px
  margin-top: 35px

.search-container
  padding-top: 20px
</style>
