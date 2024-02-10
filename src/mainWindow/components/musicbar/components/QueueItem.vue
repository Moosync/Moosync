<!-- 
  QueueItem.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="item-container" @contextmenu="getItemContextMenu">
    <b-row class="item-row">
      <b-col cols="auto" class="img-container h-100 d-flex justify-content-start">
        <LowImageCol
          @imgClicked="playSong"
          height="56px"
          width="56px"
          :src="getValidImageLow(song)"
          :showEqualizer="current"
          :isSongPlaying="isSongPlaying"
        />
      </b-col>
      <b-col xl="8" lg="7" cols="5">
        <div class="d-flex">
          <div class="text-left song-title text-truncate">{{ song.title }}</div>
          <IconHandler :item="song" />
        </div>

        <div class="text-left song-subtitle text-truncate">
          {{ song.artists && song.artists.map((val) => val.artist_name).join(', ') }}
          {{ song.artists && song.artists.length > 0 && song.album && song.album.album_name ? ' - ' : '' }}
          {{ song.album && song.album.album_name }}
        </div>
      </b-col>
      <b-col cols="auto" class="text-right ml-auto d-flex align-items-center">
        <div class="ml-auto remove-button" @click="removeSong"><TrashIcon /></div>
      </b-col>
    </b-row>
    <!-- <div class="divider" /> -->
  </b-container>
</template>

<script lang="ts">
import { mixins } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import { vxm } from '@/mainWindow/store'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import TrashIcon from '@/icons/TrashIcon.vue'

import LowImageCol from '@/mainWindow/components/generic/LowImageCol.vue'
import IconHandler from '@/mainWindow/components/generic/IconHandler.vue'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import ErrorHandler from '@/utils/ui/mixins/errorHandler'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'

@Component({
  components: {
    SongDefault,
    IconHandler,
    TrashIcon,
    LowImageCol
  }
})
export default class QueueItem extends mixins(ImgLoader, PlayerControls, ContextMenuMixin, ErrorHandler) {
  @Prop({ default: '' })
  song!: Song

  @Prop({ default: false })
  current!: boolean

  @Prop({ default: -1 })
  index!: number

  get queueProvider() {
    return this.isSyncing ? vxm.sync : vxm.player
  }

  get isSongPlaying() {
    return vxm.player.playerState === 'PLAYING'
  }

  playSong() {
    this.playFromQueue(this.index)
  }

  removeSong() {
    bus.emit(EventBus.IGNORE_MUSIC_INFO_SCROLL)
    this.removeFromQueue(this.index)
  }

  private sortQueue(options: SongSortOptions[]) {
    vxm.themes.queueSortBy = options
  }

  getItemContextMenu(event: MouseEvent) {
    this.getContextMenu(event, {
      type: 'QUEUE_ITEM',
      args: {
        isRemote: this.song.type === 'YOUTUBE' || this.song.type === 'SPOTIFY',
        song: this.song,
        songIndex: this.index,
        refreshCallback: () => this.removeSong(),
        sortOptions: {
          callback: this.sortQueue,
          current: vxm.themes.queueSortBy
        }
      }
    })
  }
}
</script>

<style lang="sass" scoped>
.item-container
  position: relative
  height: 87px

.image
  object-fit: cover
  width: 55px
  height: 55px
  border-radius: 10px

.song-title
  font-weight: 600
  font-size: 16px
  min-width: 0

.song-subtitle
  font-weight: 250
  font-size: 14px
  min-width: 0

.item-row
  height: 80px
  padding: 12px !important

.divider
  margin-top: 8px
  border-bottom: 1px solid var(--divider) !important
  width: 100%

.remove-button
  color: var(--accent)
  cursor: pointer
  padding: 10px
  svg
    width: 22px
    height: 22px

.play-button, .now-playing
  width: calc(80px - (12px * 2))
  height: calc(80px - (12px * 2))
  background: rgba(0, 0, 0, 0.6)
  position: absolute
  border-radius: 10px

.play-button
  opacity: 0
  transition: opacity 0.2s ease
  &:hover
    opacity: 1

.provider-icon
  align-self: center
  margin-left: 10px
  min-width: 20px
  min-width: 20px
  height: 20px
  width: 20px

.img-container
  min-width: calc(56px + 12px)

.text-content
  min-width: 0%
</style>
