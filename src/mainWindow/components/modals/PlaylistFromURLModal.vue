<!-- 
  PlaylistFromURLModal.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal class="playlist-url-modal" centered size="lg" :id="id" :ref="id" hide-footer hide-header>
    <div class="modal-content-container">
      <b-container fluid class="p-0">
        <b-row no-gutters class="d-flex">
          <b-col cols="auto">
            <SongDefault v-if="forceEmptyImg || !playlist || !playlist.playlist_coverPath" class="playlist-url-cover" />
            <b-img v-else class="playlist-url-cover" :src="playlist.playlist_coverPath" @error="handleImageError"
              referrerPolicy="no-referrer"></b-img>

            <div v-if="isLoading" class="loading-spinner d-flex justify-content-center">
              <b-spinner class="align-self-center" />
            </div>
          </b-col>
          <b-col cols="9">
            <b-row no-gutters class="playlist-url-details">
              <b-col cols="12" class="w-100">
                <b-row class="w-100">
                  <div class="title text-truncate" :class="{ deactivated: !playlist }">
                    {{ playlist ? playlist.playlist_name : 'New Playlist' }}
                  </div>
                </b-row>
                <b-row class="w-100">
                  <div class="subtitle text-truncate" :class="{ deactivated: !playlist }">
                    {{ songCount ? songCount + ' Songs' : '' }}
                  </div>
                </b-row>
              </b-col>
            </b-row>
            <b-row no-gutters>
              <b-col cols="12">
                <InputGroup class="input-group" :hint="$t('playlists.url.input_hint')" v-model="playlistUrl"
                  @update="parseURL" />
              </b-col>
            </b-row>
          </b-col>
        </b-row>
        <b-row v-if="songList && songList.length !== 0" no-gutters class="playlist-content-recycler-row">
          <div class="h-100 w-100">
            <RecycleScroller class="scroller" :items="songList" :item-size="83" key-field="_id" v-slot="{ item, index }"
              :direction="'vertical'">
              <SingleSearchResult class="single-result" :title="item.title"
                :subtitle="item.artists ? item.artists.map((val: Artists) => val.artist_name).join(', ') : ''"
                :coverImg="getImgSrc(getValidImageLow(item))" :divider="index != songList.length - 1" :id="index"
                @imgClick="handleClick" />
            </RecycleScroller>
          </div>
        </b-row>
      </b-container>
      <b-button class="close-button ml-3" @click="close">{{ $t('buttons.close') }}</b-button>
      <b-button class="create-button" :disabled="!addButtonEnabled" @click="addToLibrary">{{ $t('buttons.add')
      }}</b-button>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import { mixins } from 'vue-facing-decorator'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import SingleSearchResult from '@/mainWindow/components/generic/SingleSearchResult.vue'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import InputGroup from '../generic/InputGroup.vue'
import { v4 } from 'uuid'
import RemoteSong from '@/utils/ui/mixins/remoteSongMixin'
import ProviderMixin from '@/utils/ui/mixins/ProviderMixin'
import { ProviderScopes } from '@/utils/commonConstants'
import { toast } from 'vue3-toastify'
import { convertProxy } from '../../../utils/ui/common';

@Component({
  components: {
    SongDefault,
    InputGroup,
    SingleSearchResult
  }
})
export default class PlaylistFromUrlModal extends mixins(PlayerControls, ImgLoader, RemoteSong, ProviderMixin) {
  @Prop({ default: 'PlaylistFromURL' })
  id!: string

  forceEmptyImg = false

  songList: Song[] = []
  playlist: Playlist | null = null

  isLoading = false

  addButtonEnabled = false

  get songCount() {
    return this.playlist?.playlist_song_count || this.songList.length
  }

  handleImageError() {
    this.forceEmptyImg = true
  }

  private refreshCallback?: () => void

  close() {
    this.songList = []
    this.playlist = null
    this.$bvModal.hide(this.id)
  }

  playlistUrl!: string

  async parseURL(url: string) {
    if (url) {
      this.isLoading = true

      this.songList = []
      this.playlist = null
      this.addButtonEnabled = false

      const trimmed = url.trim()

      if (url.startsWith('http')) {
        const providers = this.getProvidersByScope(ProviderScopes.PLAYLIST_FROM_URL)
        for (const p of providers) {
          if (p.matchPlaylist(trimmed)) {
            try {
              this.playlist = (await p.getPlaylistDetails(trimmed)) ?? null
              if (this.playlist) {
                this.addButtonEnabled = true
                break
              }
            } catch (e) {
              console.error(e)
            }
          }
        }
      } else {
        const data = await window.FileUtils.scanSinglePlaylist(trimmed)
        this.playlist = {
          playlist_id: data.playlist?.playlist_id ?? v4(),
          playlist_name: data.playlist?.playlist_name ?? 'New Playlist',
          playlist_path: data.playlist?.playlist_path,
          playlist_coverPath: data.playlist?.playlist_coverPath,
          playlist_desc: data.playlist?.playlist_desc,
          playlist_song_count: data.playlist?.playlist_song_count
        }

        this.songList.push(...data.songs)

        this.addButtonEnabled = true
      }
    } else {
      this.addButtonEnabled = false
      this.songList = []
      this.playlist = null
    }

    this.isLoading = false
  }

  handleClick(index: number) {
    this.playTop([this.songList[index]])
  }

  async addToLibrary() {
    if (this.playlist) {
      const playlistId = await window.DBUtils.createPlaylist(convertProxy(this.playlist))

      if (!this.playlist.extension) await window.DBUtils.addToPlaylist(playlistId, ...convertProxy(this.songList, true))

      toast(`Added ${this.playlist.playlist_name} to library`)

      this.playlist = null
      this.songList = []

      this.refreshCallback && this.refreshCallback()

      this.$bvModal.hide(this.id)
    }
  }

  mounted() {
    bus.on(EventBus.SHOW_PLAYLIST_FROM_URL_MODAL, (refreshCallback: () => void) => {
      this.addButtonEnabled = false
      this.refreshCallback = refreshCallback
      this.$bvModal.show(this.id)
    })
  }
}
</script>

<style lang="sass" scoped>
.title
  font-size: 26px

.subtitle
  font-size: 14px
  font-weight: normal

.title.deactivated, .subtitle.deactivated
  color: var(--textSecondary)

.topbar-container
  background: var(--primary)
  height: 70px
  padding-left: calc(261px + 30px + 7.5px)

.playlist-url-cover
  width: 157px
  height: 157px
  object-fit: cover
  border-radius: 16px

.playlist-url-desc
  width: 100%
  background-color: transparent
  color: var(--textPrimary)
  border: 0
  padding: 0
  padding-left: 10px
  margin-top: 15px
  border-radius: 0
  border-left: 1px solid transparent
  &:focus
    -webkit-box-shadow: none
    box-shadow: none
  &:hover
    border-left: 1px solid var(--accent)

.playlist-url-details
  margin-top: -10px
  margin-left: 40px
  max-width: 100%

.playlists-count
  font-size: 14px
  text-align: start

.edit-icon
  width: 15px
  height: 15px
  min-width: 15px
  min-height: 15px
  margin-left: 15px
  margin-top: 5px

.playlist-url-title
  font-size: 26px
  max-width: 100%
  margin-bottom: 15px !important
  color: var(--textPrimary)
  background-color: transparent
  border: 0
  border-bottom: 1px solid transparent
  border-radius: 0
  padding: 0
  &:hover
    border-bottom: 1px solid var(--accent)
  &:focus
    border-bottom: 1px solid var(--accent)
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

.input-group
  margin-top: 15px
  margin-left: 10px

.playlist-content-recycler-row
  height: 200px
  margin-top: 30px
  margin-bottom: 8px

.scroller
  margin-right: -10px
  margin-left: -10px

.loading-spinner
  position: absolute
  left:  0
  top: 0
  width: 100%
  height: 100%
  background: rgba(0, 0, 0, 0.4)
  border-radius: 16px
</style>
