<!-- 
  Details.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->
<template>
  <b-row no-gutters class="w-100" align-v="center" @contextmenu="$emit('contextmenu', $event)">
    <b-col cols="auto">
      <b-img fluid ref="cover" class="coverimg" v-if="imgSrc && !forceEmptyImg" :src="imgSrc" alt="cover art"
        @error="handlerImageError($event, handleError)" @dragstart="dragFile" referrerPolicy="no-referrer" />
      <SongDefault v-else class="coverimg" />
    </b-col>
    <b-col class="text-truncate">
      <b-row align-h="start" align-v="center">
        <b-col cols="auto" class="w-100 d-flex">
          <div id="musicbar-title" :title="title" class="text song-title text-truncate mr-2" @click="onTitleClick">
            {{ title }}
          </div>

          <YoutubeIcon v-if="iconType === 'YOUTUBE'" :color="'#E62017'" :filled="true" :dropShadow="true"
            class="provider-icon" />
          <SpotifyIcon v-if="iconType === 'SPOTIFY'" :color="'#1ED760'" :filled="true" :dropShadow="true"
            class="provider-icon" />

          <inline-svg class="provider-icon" v-if="iconURL && iconType === 'URL' && iconURL.endsWith('svg')"
            :src="iconURL" />
          <img referrerPolicy="no-referrer" v-if="iconURL && iconType === 'URL' && !iconURL.endsWith('svg')"
            :src="iconURL" alt="provider icon" class="provider-icon" />
        </b-col>
      </b-row>
      <b-row no-gutters>
        <b-col class="d-flex">
          <div v-for="(artist, index) of artists" :key="index" :title="artist.artist_name"
            class="text song-subtitle text-truncate" :class="index !== 0 ? 'ml-1' : ''" @click="onSubtitleClick(artist)">
            {{ artist.artist_name }}{{ index !== artists.length - 1 ? ',' : '' }}
          </div>
        </b-col>
      </b-row>

      <b-popover id="clipboard-popover" :show.sync="showPopover" target="musicbar-title" triggers="click blur"
        placement="top">
        Copied!
      </b-popover>
    </b-col>
  </b-row>
</template>

<script lang="ts">
import { mixins } from 'vue-facing-decorator'
import { Component, Prop, Watch } from 'vue-facing-decorator'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import ImageLoader from '@/utils/ui/mixins/ImageLoader'
import ErrorHandler from '@/utils/ui/mixins/errorHandler'
import Timestamp from '@/mainWindow/components/musicbar/components/Timestamp.vue'
import FileMixin from '@/utils/ui/mixins/FileMixin'
import RouterPushes from '@/utils/ui/mixins/RouterPushes'
import SpotifyIcon from '../../../../icons/SpotifyIcon.vue'
import YoutubeIcon from '../../../../icons/YoutubeIcon.vue'

@Component({
  components: {
    SongDefault,
    Timestamp,
    SpotifyIcon,
    YoutubeIcon
  },
  emits: ['contextmenu']
})
export default class MusicBar extends mixins(ImageLoader, ErrorHandler, FileMixin, RouterPushes) {
  @Prop({ default: '-' })
  title!: string

  @Prop({ default: () => [] })
  artists!: Artists[]

  @Prop({ default: '' })
  imgSrc!: string

  showPopover = false

  forceEmptyImg = false

  @Prop({ default: '' })
  iconType!: string

  @Prop({ default: '' })
  iconURL!: string

  onTitleClick() {
    let str = this.artists.map((val) => val.artist_name).join(', ')
    if (str) {
      str += ' - '
    }
    str += this.title
    navigator.clipboard.writeText(str)
    this.showPopover = true
    setTimeout(() => (this.showPopover = false), 1500)
  }

  async onSubtitleClick(artist: Artists) {
    this.gotoArtist(artist)
  }

  handleError() {
    this.forceEmptyImg = true
  }

  @Watch('imgSrc') onImgSrcChange() {
    this.forceEmptyImg = false
  }
}
</script>

<style lang="sass" scoped>
.coverimg
  height: 56px
  width: 56px
  min-width: 56px
  margin-right: 15px
  border-radius: 10px
  object-fit: cover

.text
  text-align: left
  font-weight: normal
  line-height: 170.19%

.song-title
  cursor: pointer
  font-size: 19.1549px
  width: fit-content

.song-subtitle
  font-size: 14.2592px
  color: var(--textSecondary)
  width: fit-content
  cursor: pointer
  text-decoration: none
  &:hover
    text-decoration: underline
</style>
