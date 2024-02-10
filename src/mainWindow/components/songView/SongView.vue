<!-- 
  SongView.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="song-container h-100" @contextmenu="onGeneralContextMenu">
    <transition name="custom-classes-transition"
      enter-active-class="animate__animated animate__slideInLeft animate__delay-1s animate__slideInLeft_delay"
      leave-active-class="animate__animated animate__slideOutRight animate__slideOutRight_faster">
      <SongViewCompact v-if="songView === 'SongViewCompact'" :songList="filteredSongList" :currentSong="currentSong"
        :defaultDetails="defaultDetails" :detailsButtonGroup="detailsButtonGroup" :optionalProviders="optionalProviders"
        :isLoading="isLoading" @onItemsChanged="onOptionalProviderChanged"
        @onRowDoubleClicked="(song: Song) => queueSong([song])" @onRowContext="onSongContextMenu"
        @onRowSelected="updateCoverDetails" @onRowSelectionClear="clearSelection"
        @onRowPlayNowClicked="(song: Song) => playTop([song])" @onArtistClicked="gotoArtist" @onAlbumClicked="gotoAlbum"
        @playAll="playAll" @addToQueue="addToQueue" @addToLibrary="addToLibrary" @onSortClicked="showSortMenu"
        @onSearchChange="onSearchChange" @playRandom="playRandom" @fetchAll="fetchAll" @onScrollEnd="onScrollEnd">
      </SongViewCompact>

      <SongViewClassic v-else :songList="filteredSongList" :currentSong="currentSong" :defaultDetails="defaultDetails"
        :detailsButtonGroup="detailsButtonGroup" :optionalProviders="optionalProviders" :isLoading="isLoading"
        @onItemsChanged="onOptionalProviderChanged" @onRowDoubleClicked="(song: Song) => queueSong([song])"
        @onRowContext="onSongContextMenu" @onRowSelected="updateCoverDetails" @onRowSelectionClear="clearSelection"
        @onRowPlayNowClicked="(song: Song) => playTop([song])" @onArtistClicked="gotoArtist" @onAlbumClicked="gotoAlbum"
        @playAll="playAll" @addToQueue="addToQueue" @addToLibrary="addToLibrary" @onSortClicked="showSortMenu"
        @onSearchChange="onSearchChange" @playRandom="playRandom" @fetchAll="fetchAll" @onScrollEnd="onScrollEnd">
      </SongViewClassic>
    </transition>
  </b-container>
</template>

<script lang="ts">
import { Component, Prop, Watch } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import ModelHelper from '@/utils/ui/mixins/ModelHelper'
import RemoteSong from '@/utils/ui/mixins/remoteSongMixin'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import { vxm } from '@/mainWindow/store'
import SongViewClassic from '@/mainWindow/components/songView/components/SongViewClassic.vue'
import SongViewCompact from '@/mainWindow/components/songView/components/SongViewCompact.vue'
import { arrayDiff, sortSongList } from '@/utils/common'
import RouterPushes from '@/utils/ui/mixins/RouterPushes'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'

@Component({
  components: {
    SongViewClassic,
    SongViewCompact
  },
  emits: [
    'playAll',
    'addToQueue',
    'addToLibrary',
    'playRandom',
    'fetchAll',
    'onOptionalProviderChanged',
    'onSearchChange',
    'onScrollEnd'
  ]
})
export default class SongView extends mixins(
  PlayerControls,
  ModelHelper,
  RemoteSong,
  ImgLoader,
  RouterPushes,
  ContextMenuMixin
) {
  @Prop({ default: () => [] })
  private songList!: Song[]

  private ignoreSort = false

  @Prop({ default: false })
  isLoading!: boolean

  @Prop({ default: () => [] })
  optionalProviders!: TabCarouselItem[]

  @Prop()
  private afterSongAddRefreshCallback!: ((showHidden?: boolean) => void) | undefined

  @Prop()
  private isRemote!: ((songs: Song[]) => boolean) | undefined

  private searchText = ''

  get filteredSongList(): Song[] {
    let songList = this.songList.filter((val) => !!val.title.match(new RegExp(this.searchText, 'i')))
    songList = vxm.themes.songSortBy && sortSongList(songList, vxm.themes.songSortBy)
    return songList
  }

  @Watch('songList', { immediate: true })
  private async onSongListChanged(newVal: Song[], oldVal: Song[]) {
    const difference = newVal.filter((x) => {
      return (oldVal ?? []).findIndex((val) => val._id === x._id) === -1
    })

    const playCounts = await window.SearchUtils.getPlayCount(...difference.map((val) => val._id))
    for (const song of difference) {
      song['playCount'] = playCounts[song._id]?.playCount ?? 0
    }
  }

  get songView() {
    return vxm.themes.songView === 'compact' ? 'SongViewCompact' : 'SongViewClassic'
  }

  private selected: Song[] | null = null
  private selectedCopy: Song[] | null = null

  currentSong: Song | null | undefined = null

  @Prop({
    default: () => {
      return { defaultTitle: '', defaultSubtitle: '', defaultCover: '' }
    }
  })
  defaultDetails!: SongDetailDefaults

  @Prop({
    default: () => {
      return {
        enableContainer: false,
        enableLibraryStore: false,
        playRandom: false
      }
    }
  })
  detailsButtonGroup!: SongDetailButtons

  @Prop({ default: null })
  private onSongContextMenuOverride!: ((event: PointerEvent, songs: Song[]) => void) | null

  @Prop({ default: null })
  private onGeneralSongContextMenuOverride!: ((event: PointerEvent) => void) | null

  clearSelection() {
    this.currentSong = null
    this.selected = this.selectedCopy
    this.selectedCopy = null
  }

  updateCoverDetails(items: Song[]) {
    if (items) this.currentSong = items[items.length - 1]
    this.selected = items
    this.selectedCopy = items
  }

  private sort(options: SongSortOptions[]) {
    vxm.themes.songSortBy = options
  }

  onSongContextMenu(event: PointerEvent, songs: Song[]) {
    if (this.onSongContextMenuOverride) {
      this.onSongContextMenuOverride(event, songs)
    } else {
      this.getContextMenu(event, {
        type: 'SONGS',
        args: {
          songs,
          isRemote: typeof this.isRemote === 'function' && this.isRemote(songs),
          refreshCallback: () => this.songList.splice(0, this.songList.length, ...arrayDiff(this.songList, songs))
        }
      })
    }
  }

  showSortMenu(event: MouseEvent) {
    this.getContextMenu(event, {
      type: 'SONG_SORT',
      args: {
        sortOptions: { callback: this.sort, current: vxm.themes.songSortBy }
      }
    })
  }

  onGeneralContextMenu(event: PointerEvent) {
    if (this.onGeneralSongContextMenuOverride) {
      this.onGeneralSongContextMenuOverride(event)
    } else {
      this.getContextMenu(event, {
        type: 'GENERAL_SONGS',
        args: {
          refreshCallback: this.afterSongAddRefreshCallback,
          sortOptions: {
            callback: (options) => (vxm.themes.songSortBy = options),
            current: vxm.themes.songSortBy
          }
        }
      })
    }
  }

  playAll() {
    if (this.selected) {
      this.playTop(this.selected)
      this.selected = this.selectedCopy
      return
    }
    this.$emit('playAll')
  }

  addToQueue() {
    if (this.selected) {
      this.queueSong(this.selected)
      this.selected = this.selectedCopy
      return
    }
    this.$emit('addToQueue')
  }

  addToLibrary() {
    if (this.selected) {
      this.addSongsToLibrary(...this.selected)
      this.selected = this.selectedCopy
      return
    }
    this.$emit('addToLibrary')
  }

  playRandom() {
    this.$emit('playRandom')
  }

  fetchAll() {
    this.$emit('fetchAll')
  }

  onOptionalProviderChanged(...args: unknown[]) {
    this.$emit('onOptionalProviderChanged', ...args)
  }

  onSearchChange(text: string) {
    this.searchText = text
    this.$emit('onSearchChange', text)
  }

  onScrollEnd() {
    this.$emit('onScrollEnd')
  }
}
</script>

<style lang="sass" scoped>
.song-container
  padding-top: 10px
  overflow: hidden

.compact-container
  padding-top: 25px

.song-list-compact
  padding-right: 30px
</style>
