<!-- 
  NewPlaylistModal.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal class="playlist-modal" centered size="lg" :id="id" :ref="id" hide-footer hide-header>
    <div class="modal-content-container">
      <b-container fluid class="p-0">
        <b-row no-gutters class="d-flex">
          <canvas crossorigin="anonymous" v-if="!forceEmptyImg" ref="canvas" width="800" height="800" id="playlist-cover"
            class="playlist-cover"></canvas>
          <SongDefault v-if="forceEmptyImg" class="playlist-cover" />
          <b-col class="playlist-details">
            <div class="d-flex">
              <b-input v-model="title" id="playlist-title" class="playlist-title" maxlength="20"
                placeholder="Playlist Name..." />
            </div>
            <p class="songs-count">{{ songCount }} {{ songCount == 1 ? 'Song' : 'Songs' }}</p>
          </b-col>
        </b-row>
        <b-row no-gutters>
          <b-form-textarea class="playlist-desc" id="playlist-desc" v-model="desc"
            :placeholder="$t('playlists.new_playlist.description_placeholder')" rows="3" max-rows="6"></b-form-textarea>
        </b-row>
      </b-container>
      <b-button class="create-button" @click="createPlaylist">{{ $t('playlists.new_playlist.create') }}</b-button>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { EventBus } from '@/utils/preload/ipc/constants'
import { Component, Prop, Ref } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import { bus } from '@/mainWindow/main'
import { vxm } from '@/mainWindow/store'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import RemoteSong from '@/utils/ui/mixins/remoteSongMixin'

@Component({
  components: {
    SongDefault
  }
})
export default class NewPlaylistModal extends mixins(ImgLoader, RemoteSong) {
  @Prop({ default: 'NewPlaylistModal' })
  id!: string

  title = 'New Playlist'
  desc = ''

  songs: Song[] = []
  songCount = 0

  forceEmptyImg = true

  showing = false

  private createCallback: (() => void) | undefined

  @Ref('canvas') private canvas!: HTMLCanvasElement

  private isDuplicatePlaylistName(): boolean {
    for (const playlist of Object.values(vxm.playlist.playlists)) {
      if (this.title === playlist) {
        return true
      }
    }
    return false
  }

  async createPlaylist() {
    let path
    if (this.canvas) {
      const data = this.canvas.toDataURL('image/png')
      path = await window.FileUtils.savePlaylistCover(data)
    }

    const playlist_id = await window.DBUtils.createPlaylist({
      playlist_name: this.title,
      playlist_coverPath: path,
      playlist_desc: this.desc
    })
    this.addToPlaylist(playlist_id, this.songs)

    this.$bvModal.hide(this.id)
    vxm.playlist.updated = true

    this.createCallback && this.createCallback()
  }

  private handleImageError() {
    this.forceEmptyImg = true
  }

  private getValidImages() {
    const mergableImages: string[] = []
    for (const song of this.songs) {
      const cover = this.getValidImageHigh(song)
      if (cover) mergableImages.push(cover)
    }
    return mergableImages.slice(0, 4)
  }

  private async createImage(src: string, quad: number, len: number, ctx: CanvasRenderingContext2D) {
    const img = new Image()
    img.onload = () => this.drawImage(quad, len, ctx, img)
    img.crossOrigin = ''
    img.src = this.getImgSrc(src)
  }

  private isExpandedImage(quad: number, len: number) {
    if (len === 3) {
      return quad === 1
    }

    if (len < 3) return true

    return false
  }

  private getImageSize(quad: number, len: number) {
    return this.isExpandedImage(quad, len) ? 800 : 400
  }

  private getCenter(width: number, height: number) {
    const dx = Math.max((width - height) / 2, 0)
    return dx
  }

  private drawImage(quad: number, len: number, ctx: CanvasRenderingContext2D, img: HTMLImageElement) {
    const size = this.getImageSize(quad, len)
    const dx = this.getCenter(img.naturalWidth, img.naturalHeight)
    switch (quad) {
      case 0:
        ctx.drawImage(img, dx, 0, img.naturalHeight, img.naturalHeight, 0, 0, size, size)
        break
      case 1:
        ctx.drawImage(img, dx, 0, img.naturalHeight, img.naturalHeight, 400, 0, size, size)
        break
      case 2:
        ctx.drawImage(img, dx, 0, img.naturalHeight, img.naturalHeight, 0, 400, size, size)
        break
      case 3:
        ctx.drawImage(img, dx, 0, img.naturalHeight, img.naturalHeight, 400, 400, size, size)
        break
    }
  }

  private drawWholeImage(ctx: CanvasRenderingContext2D, src: string) {
    const img = new Image()
    img.src = this.getImgSrc(src)
    img.onload = () => ctx.drawImage(img, 0, 0, 800, 800)
  }

  private async mergeImages() {
    const mergableImages = this.getValidImages()

    await this.$nextTick()
    if (mergableImages.length === 0) {
      this.forceEmptyImg = true
    } else {
      if (this.canvas) {
        const ctx = this.canvas.getContext('2d')
        if (ctx) {
          ctx.clearRect(0, 0, this.canvas.width, this.canvas.height)
          for (let i = 0; i < mergableImages.length; i++)
            this.createImage(mergableImages[i], i, mergableImages.length, ctx)
        }
      }
    }
  }

  private async addToPlaylist(playlist_id: string, songs: Song[]) {
    await window.DBUtils.addToPlaylist(playlist_id, ...songs)
  }

  mounted() {
    bus.on(EventBus.SHOW_NEW_PLAYLIST_MODAL, (songs: Song[], createCallback?: () => void) => {
      if (!this.showing) {
        this.songs = songs
        this.songCount = songs.length
        this.forceEmptyImg = false
        this.desc = ''
        this.title = 'New Playlist'

        this.createCallback = createCallback

        this.$nextTick(() => this.mergeImages())

        for (let i = 1; this.isDuplicatePlaylistName(); i++) {
          this.title = `New Playlist ${i}`
        }

        this.$bvModal.show(this.id)
      }
    })
  }
}
</script>

<style lang="sass" scoped>
.topbar-container
  background: var(--primary)
  height: 70px
  padding-left: calc(261px + 30px + 7.5px)

.playlist-cover
  width: 157px
  height: 157px
  border-radius: 16px

.playlist-desc
  width: 100%
  background-color: transparent
  color: var(--textPrimary)
  border: 0
  padding: 0
  padding-left: 10px
  margin-top: 15px
  border-radius: 0
  border-left: 1px solid var(--divider)
  &:focus
    -webkit-box-shadow: none
    box-shadow: none
  &:hover
    border-left: 1px solid var(--accent)

.playlist-details
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

.playlist-title
  font-size: 26px
  max-width: 100%
  margin-bottom: 15px !important
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

.create-button
  font-size: 16px
  color: var(--textInverse)
  background-color: var(--accent)
  border-radius: 6px
  float: right
  margin-bottom: 20px
  margin-top: 15px
  border: 0
</style>
