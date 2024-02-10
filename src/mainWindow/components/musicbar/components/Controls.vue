<!-- 
  Controls.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-row align-v="center" align-h="center" no-gutters>
    <b-col class="col-button" @click="prevSongWrapper()" v-if="isSkipEnabled">
      <PrevTrack :disabled="!enableTrackControls" />
    </b-col>
    <b-col class="col-button" @click="toggleRepeat()" v-if="isRepeatEnabled">
      <RepeatOnceIcon v-if="repeat === 1" :filled="true" />
      <Repeat v-else :filled="repeat === 2" />
    </b-col>
    <b-col class="col-play-button" v-if="isLoading && !isJukeboxModeActive">
      <b-spinner label="Loading..."></b-spinner>
    </b-col>
    <b-col class="col-play-button" v-else-if="!isJukeboxModeActive" v-on:click="togglePlayerState()">
      <Play :play="playerState === 'PLAYING'" />
    </b-col>
    <b-col class="col-button" @click="nextSongWrapper()" v-if="isSkipEnabled">
      <NextTrack :disabled="!enableTrackControls" />
    </b-col>
    <b-col class="col-button" @click="shuffle()" v-if="isShuffleEnabled">
      <Shuffle :filled="true" />
    </b-col>
    <b-col class="col-button mr-1" @click="favoriteSong" v-if="!isJukeboxModeActive">
      <FavIcon :filled="isFavorite" />
    </b-col>
    <b-col cols="5" md="3" align-self="center" class="timestamp-container">
      <Timestamp class="timestamp" :duration="duration" :timestamp="timestamp" />
    </b-col>
  </b-row>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import PrevTrack from '@/icons/PrevTrackIcon.vue'
import NextTrack from '@/icons/NextTrackIcon.vue'
import Play from '@/icons/PlayIcon.vue'
import Repeat from '@/icons/RepeatIcon.vue'
import FavIcon from '@/icons/FavIcon.vue'
import Shuffle from '@/icons/ShuffleIcon.vue'
import { mixins } from 'vue-facing-decorator'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import { vxm } from '@/mainWindow/store'
import Timestamp from '@/mainWindow/components/musicbar/components/Timestamp.vue'
import JukeboxMixin from '@/utils/ui/mixins/JukeboxMixin'
import { FAVORITES_PLAYLIST_ID } from '@/utils/commonConstants'
import RepeatOnceIcon from '../../../../icons/RepeatOnceIcon.vue';
import { convertProxy } from '@/utils/ui/common'

@Component({
  components: {
    PrevTrack,
    NextTrack,
    Play,
    Repeat,
    RepeatOnceIcon,
    Shuffle,
    Timestamp,
    FavIcon
  }
})
export default class MusicBar extends mixins(PlayerControls, JukeboxMixin) {
  @Prop({ default: 0 })
  duration!: number

  @Prop({ default: 0 })
  timestamp!: number

  get playerState() {
    return vxm.player.playerState
  }

  get enableTrackControls() {
    return this.isSyncing ? vxm.sync.queueOrder.length > 1 : vxm.player.queueOrder.length > 1
  }

  nextSongWrapper() {
    if (this.enableTrackControls) this.nextSong()
  }

  prevSongWrapper() {
    if (this.enableTrackControls) this.prevSong()
  }

  isFavorite = false

  created() {
    vxm.player.$watch(
      'currentSong',
      async (song?: Song) => {
        if (song) {
          const s = await window.SearchUtils.searchSongsByOptions({
            song: {
              _id: song._id
            },
            playlist: {
              playlist_id: FAVORITES_PLAYLIST_ID
            },
            inclusive: true
          })

          this.isFavorite = s.length > 0
        }
      },
      { immediate: true, deep: false }
    )
  }

  async favoriteSong() {
    if (vxm.player.currentSong) {
      if (!this.isFavorite) {
        await window.DBUtils.addToPlaylist(FAVORITES_PLAYLIST_ID, convertProxy(vxm.player.currentSong))
      } else {
        await window.DBUtils.removeFromPlaylist(FAVORITES_PLAYLIST_ID, convertProxy(vxm.player.currentSong))
      }
      this.isFavorite = !this.isFavorite
    }
  }

  get isLoading() {
    return vxm.player.loading
  }
}
</script>

<style lang="sass" scoped>
.invisible
  min-width: 0%

.timestamp-container
  display: block

.fav-icon
  margin-right: 1.5rem

.col-button
  max-width: calc(26px + 1.5rem)

.col-play-button
  max-width: calc(42px + 1.5rem)

@media only screen and (max-width : 800px)
  .timestamp-container
    display: none

  .shuffle-icon
    margin-right: 0px
</style>
