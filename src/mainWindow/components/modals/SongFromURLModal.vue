<!-- 
  SongFromURLModal.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal class="song-url-modal" centered size="lg" :id="id" :ref="id" hide-footer hide-header>
    <div class="modal-content-container">
      <b-container fluid class="p-0">
        <b-row no-gutters class="d-flex">
          <b-col cols="auto">
            <SongDefault v-if="forceEmptyImg || !parsedSong || !getValidImageHigh(parsedSong)" class="song-url-cover" />
            <b-img
              v-else
              class="song-url-cover"
              :src="getImgSrc(getValidImageHigh(parsedSong))"
              @error="handleImageError"
              referrerPolicy="no-referrer"
            ></b-img>

            <div v-if="isLoading" class="loading-spinner d-flex justify-content-center">
              <b-spinner class="align-self-center" />
            </div>
          </b-col>
          <b-col cols="9">
            <b-row no-gutters class="song-url-details">
              <b-col class="w-100">
                <b-row class="w-100">
                  <b-input
                    class="title text-truncate"
                    placeholder="New Song"
                    :class="{ deactivated: !parsedSong }"
                    v-model="songTitle"
                    :disabled="!parsedSong"
                  />
                </b-row>
                <b-row class="w-100">
                  <b-input
                    class="subtitle text-truncate"
                    placeholder="Artist"
                    :class="{ deactivated: !parsedSong }"
                    v-model="songArtist"
                    :disabled="!parsedSong"
                  />
                </b-row>
              </b-col>
            </b-row>
            <b-row no-gutters>
              <b-col cols="12">
                <InputGroup class="input-group" hint="Enter URL Here.. (Youtube or Spotify)" @update="parseURL" />
              </b-col>
            </b-row>
          </b-col>
        </b-row>
      </b-container>
      <div class="mt-3 warning" v-if="!isLoggedIn">* Requires to be logged in to respective services</div>
      <b-button class="close-button ml-3" @click="close">Close</b-button>
      <b-button class="create-button" :disabled="!addButtonEnabled" @click="addToLibrary">Add</b-button>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import InputGroup from '../generic/InputGroup.vue'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import { vxm } from '@/mainWindow/store'
import { v4 } from 'uuid'
import { mixins } from 'vue-facing-decorator'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import RemoteSong from '@/utils/ui/mixins/remoteSongMixin'
import ProviderMixin from '@/utils/ui/mixins/ProviderMixin'
import { ProviderScopes } from '@/utils/commonConstants'

@Component({
  components: {
    SongDefault,
    InputGroup
  }
})
export default class SongFromUrlModal extends mixins(ImgLoader, RemoteSong, ProviderMixin) {
  @Prop({ default: 'SongFromURL' })
  id!: string

  forceEmptyImg = false

  parsedSong: Song | null = null

  songTitle = ''
  songArtist = ''

  isLoading = false

  addButtonEnabled = false

  private refreshCallback?: () => void

  handleImageError() {
    this.forceEmptyImg = true
  }

  isLoggedIn = false

  async parseURL(url: string) {
    if (url) {
      this.isLoading = true
      this.addButtonEnabled = false

      this.forceEmptyImg = false

      this.parsedSong = (await window.FileUtils.scanSingleSong(url)).song

      if (!this.parsedSong) {
        const providers = this.getProvidersByScope(ProviderScopes.SONG_FROM_URL)
        for (const p of providers) {
          console.debug('matching url to', p, p.matchSongUrl(url))
          if (p.matchSongUrl(url)) {
            try {
              this.parsedSong = (await p.getSongDetails(url)) ?? null
              if (this.parsedSong) {
                break
              }
            } catch (e) {
              console.error(e)
            }
          }
        }
      }

      if (!this.parsedSong) {
        this.parsedSong = (await this.parseStream(url)) ?? null
      }

      if (this.parsedSong) {
        this.songTitle = this.parsedSong.title ?? ''
        this.songArtist = this.parsedSong.artists?.map((val) => val.artist_name).join(', ') ?? ''
        this.addButtonEnabled = true
      } else {
        this.songTitle = ''
        this.songArtist = ''
      }
    } else {
      this.addButtonEnabled = false
      this.songTitle = ''
      this.songArtist = ''
      this.parsedSong = null
    }

    this.isLoading = false
  }

  private async parseStream(url: string): Promise<Song | undefined> {
    return new Promise<Song | undefined>((resolve) => {
      const audio = new Audio()
      audio.onloadedmetadata = () => {
        const song: Song = {
          _id: v4(),
          title: 'Song from URL',
          duration: audio.duration,
          date_added: Date.now(),
          playbackUrl: url,
          type: 'URL'
        }

        resolve(song)
      }

      audio.onerror = () => resolve(undefined)
      audio.src = url
    })
  }

  private getArtists(artists: string) {
    const split = artists.split(',')
    const ret: Artists[] = []
    for (const s of split) {
      ret.push({ artist_id: '', artist_name: s })
    }
    return ret
  }

  addToLibrary() {
    if (this.parsedSong) {
      this.addSongsToLibrary({
        ...this.parsedSong,
        title: this.songTitle,
        artists: this.getArtists(this.songArtist)
      })

      this.refreshCallback && this.refreshCallback()
      this.close()
    }
  }

  close() {
    this.parsedSong = null
    this.songTitle = ''
    this.songArtist = ''
    this.$bvModal.hide(this.id)
  }

  mounted() {
    bus.on(EventBus.SHOW_SONG_FROM_URL_MODAL, (refreshCallback: () => void) => {
      this.refreshCallback = refreshCallback
      this.forceEmptyImg = false
      this.addButtonEnabled = false
      this.isLoggedIn = vxm.providers.youtubeProvider.loggedIn && vxm.providers.spotifyProvider.loggedIn
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

.title.deactivated, .subtitle.deactivated
  color: var(--textSecondary)
  border-bottom: none !important
  margin-bottom: -10px

.topbar-container
  background: var(--primary)
  height: 70px
  padding-left: calc(261px + 30px + 7.5px)

.song-url-cover
  width: 157px
  height: 157px
  object-fit: cover
  border-radius: 16px

.song-url-desc
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

.song-url-details
  margin-top: -10px
  margin-left: 40px
  max-width: 100%

.songs-count
  font-size: 14px
  text-align: start

.edit-icon
  width: 15px
  height: 15px
  min-width: 15px
  min-height: 15px
  margin-left: 15px
  margin-top: 5px

.song-url-title
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

.warning
  color: #EB2525

.loading-spinner
  position: absolute
  left:  0
  top: 0
  width: 100%
  height: 100%
  background: rgba(0, 0, 0, 0.4)
  border-radius: 16px
</style>
