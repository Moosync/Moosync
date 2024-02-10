<!-- 
  IncorrectPlaybackModal.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal class="song-url-modal" centered size="lg" :id="id" :ref="id" hide-footer hide-header>
    <div class="modal-content-container">
      <b-container fluid class="p-0">
        <b-row>
          <div class="w-100 searchbar-container full-border mb-4">
            <SearchIcon class="search-icon" />
            <b-form-input
              class="searchbar"
              :placeholder="$t('topbar.searchPlaceholder')"
              type="text"
              v-model="inputText"
              debounce="300"
              ref="inputfield"
              @keyup.enter="fetchSearchResults"
            />
            <AltArrowIcon @click="fetchSearchResults" v-if="inputText !== ''" class="go-arrow button-grow" />
          </div>
        </b-row>
        <b-row v-if="!isLoading">
          <SongListCompactItem
            class="mb-3"
            v-for="(item, index) of searchResults"
            :item="item"
            :key="item._id"
            :index="index"
            :showPlayHoverButton="false"
            :showEllipsis="false"
            :showAddToQueueButton="false"
            :selected="[selected]"
            @onRowSelected="selectItem(index)"
          />
        </b-row>
        <b-row v-else>
          <div class="loading-spinner d-flex justify-content-center">
            <b-spinner class="align-self-center" />
          </div>
        </b-row>
      </b-container>
      <b-button class="close-button ml-3" @click="close">Close</b-button>
      <b-button class="create-button" @click="save">Save</b-button>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import { vxm } from '@/mainWindow/store'
import SongListCompactItem from '../songView/components/SongListCompactItem.vue'
import SearchIcon from '@/icons/SearchIcon.vue'
import AltArrowIcon from '@/icons/AltArrowIcon.vue'
import { mixins } from 'vue-facing-decorator'
import CacheMixin from '@/utils/ui/mixins/CacheMixin'

@Component({
  components: {
    SongListCompactItem,
    SearchIcon,
    AltArrowIcon
  }
})
export default class IncorrectPlaybackModal extends mixins(CacheMixin) {
  @Prop({ default: 'IncorrectPlayback' })
  id!: string

  searchResults: Song[] = []

  isLoading = false
  selected = 0

  inputText = ''

  parsedSong: Song | null = null

  close() {
    this.$bvModal.hide(this.id)
    this.parsedSong = null
  }

  save() {
    if (this.parsedSong) {
      const selectedSong = this.searchResults[this.selected]
      this.parsedSong.playbackUrl = selectedSong?.playbackUrl
      this.parsedSong.duration = selectedSong.duration

      const cache = { url: this.parsedSong.playbackUrl, duration: this.parsedSong.duration }
      this.setItem(`url_duration:${this.parsedSong._id}`, cache)

      vxm.player.setQueueDataSong(this.parsedSong)
      if (vxm.player.currentSong?._id === this.parsedSong?._id) {
        bus.emit(EventBus.FORCE_LOAD_SONG)
      }

      this.close()
    }
  }

  selectItem(index: number) {
    this.selected = index
  }

  async fetchSearchResults() {
    this.isLoading = true

    this.selected = 0
    const resp = await vxm.providers.youtubeProvider.searchSongs(this.inputText)
    this.searchResults = resp.slice(0, 5)

    this.isLoading = false
  }

  mounted() {
    bus.on(EventBus.SHOW_INCORRECT_PLAYBACK_MODAL, (song: Song) => {
      this.parsedSong = song
      this.inputText = `${song.artists?.map((val) => val.artist_name)?.join(', ')} ${song.title}`

      this.fetchSearchResults()

      this.$bvModal.show(this.id)
    })
  }
}
</script>

<style lang="sass" scoped>
.title
  font-size: 26px
  max-width: 100%
  color: var(--textPrimary)
  background-color: transparent
  border: 0
  border-bottom: 1px solid var(--divider)
  border-radius: 0
  padding: 0
  &:hover
    border-bottom: 1px solid var(--accent)
  &:focus
    outline: none
    -webkit-box-shadow: none

.subtitle
  font-size: 14px
  font-weight: normal
  max-width: 100%
  color: var(--textPrimary)
  background-color: transparent
  border: 0
  border-bottom: 1px solid var(--divider)
  border-radius: 0
  padding: 0
  &:hover
    border-bottom: 1px solid var(--accent)
  &:focus
    outline: none
    -webkit-box-shadow: none

.create-button, .close-button
  font-size: 16px
  font-weight: 400
  color: var(--textInverse)
  background-color: var(--accent)
  border-radius: 6px
  float: right
  margin-bottom: 20px
  margin-top: 15px
  border: 0

.close-button
  background-color: var(--textPrimary)

.loading-spinner
  position: absolute
  left:  0
  top: 0
  width: 100%
  height: 100%
  background: rgba(0, 0, 0, 0.4)
  border-radius: 16px

.searchbar-container
  height: 50px
  background: var(--secondary)
  position: relative

.search-container
  position: relative

.search-icon
  position: absolute
  height: 24px
  top: 50%
  left: 0
  margin-top: -12px
  margin-left: 15px

.full-border
  border-radius: 58px

.go-arrow
  position: absolute
  height: 20px
  top: 50%
  right: 0
  margin-top: -12px
  margin-right: 20px

.searchbar
  color: var(--textPrimary) !important
  background: rgba(0, 0, 0, 0)
  border: none
  height: 24px
  margin-top: -12px
  width: calc(100% - 24px - 18px - 15px - 30px)
  position: absolute
  transition: background 0.3s cubic-bezier(0.39, 0.58, 0.57, 1), border-radius 1000ms
  text-align: left
  top: 50%
  box-shadow: none
  margin-left: calc(24px + 18px)
  &::-webkit-input-placeholder
    color: var(--textSecondary)
  &:focus
    background: rgba(0, 0, 0, 0) !important
    outline: 0
</style>
