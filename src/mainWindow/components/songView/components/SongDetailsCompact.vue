<!-- 
  SongDetailsCompact.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid :class="`h-100 ${scrollable ? 'scrollable' : ''}`">
    <b-row no-gutters>
      <b-col class="position-relative">
        <div class="image-container w-100" @click="emitClick">
          <div class="embed-responsive embed-responsive-1by1">
            <div class="embed-responsive-item">
              <transition
                name="custom-fade"
                enter-active-class="animate__animated animate__fadeIn"
                leave-active-class="animate__animated animate__fadeOut animate__faster"
              >
                <b-img
                  @dragstart="dragFile"
                  class="h-100 w-100 albumart"
                  v-if="computedImg"
                  :src="computedImg"
                  :key="computedImg"
                  @error="handleImageError"
                  referrerPolicy="no-referrer"
                />
                <SongDefault
                  class="albumart w-100"
                  v-else-if="defaultDetails?.defaultCover !== FAVORITES_PLAYLIST_ID"
                />
                <FavPlaylistIcon class="albumart w-100" v-else />
              </transition>
              <div class="play-button d-flex justify-content-center" v-if="showPlayHoverButton">
                <Play2 class="align-self-center" />
              </div>

              <div v-if="isLoading" class="loading-spinner d-flex justify-content-center">
                <b-spinner class="align-self-center" />
              </div>

              <div v-if="cardHoverText" :class="`hoverText ${pinHoverText ? 'visible-always' : ''}`">
                <PinIcon :filled="pinHoverText" @click="pinHoverText = !pinHoverText" class="pin-icon button-grow" />
                <div class="black-overlay"></div>
                <div v-html="parsedCardHoverText"></div>
              </div>
            </div>
          </div>

          <div class="song-info-container" :style="{ color: `${forceWhiteText ? '#fff' : 'var(--textPrimary)'}` }">
            <b-row class="d-flex">
              <b-col class="song-title text-truncate">
                {{ title }}
              </b-col>

              <b-col
                cols="auto"
                class="align-self-center ml-auto show-lyrics"
                v-if="isShowLyricsActive !== 0"
                :style="{ color: isShowLyricsActive === 2 ? 'var(--accent)' : 'var(--textPrimary)' }"
                @click="onShowLyricsClicked"
              >
                {{ isShowLyricsActive === 2 ? 'Show lyrics' : 'Show video' }}
              </b-col>
            </b-row>
            <div class="song-subtitle text-truncate" :title="subtitle" v-if="subtitle">{{ subtitle }}</div>
            <div class="song-timestamp" :title="subSubTitle" v-if="showSubSubTitle && subSubTitle">
              {{ subSubTitle }}
            </div>
          </div>
        </div>
      </b-col>
    </b-row>
    <b-row no-gutters class="flex-fill mt-2">
      <b-col>
        <div v-if="buttonGroup.enableContainer" class="button-group d-flex">
          <PlainPlay v-if="!isJukeboxModeActive" :title="$t('buttons.playSingle', { title })" @click="playAll" />
          <AddToQueue :title="$t('buttons.addToQueue', { title })" @click="addToQueue" />
          <AddToLibrary
            :title="$t('buttons.addToLibrary', { title })"
            @click="addToLibrary"
            v-if="buttonGroup.enableLibraryStore"
          />
          <RandomIcon v-if="buttonGroup.playRandom" :title="$t('buttons.playRandom')" @click="playRandom" />
          <FetchAllIcon v-if="buttonGroup.fetchAll" :title="$t('buttons.fetchAll')" @click="fetchAll" />
        </div>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import FileMixin from '@/utils/ui/mixins/FileMixin'

import { mixins, Component } from 'vue-facing-decorator'
import { Prop, Watch } from 'vue-facing-decorator'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import { convertDuration } from '@/utils/common'
import PlainPlay from '@/icons/PlainPlayIcon.vue'
import AddToLibrary from '@/icons/AddToLibraryIcon.vue'
import AddToQueue from '@/icons/AddToQueueIcon.vue'
import PinIcon from '@/icons/PinIcon.vue'
import RandomIcon from '@/icons/RandomIcon.vue'
import JukeboxMixin from '@/utils/ui/mixins/JukeboxMixin'
import Play2 from '@/icons/PlayIcon2.vue'
import FetchAllIcon from '@/icons/FetchAllIcon.vue'
import { FAVORITES_PLAYLIST_ID } from '@/utils/commonConstants'
import FavPlaylistIcon from '@/icons/FavPlaylistIcon.vue'

@Component({
  components: {
    SongDefault,
    PlainPlay,
    AddToLibrary,
    AddToQueue,
    PinIcon,
    RandomIcon,
    Play2,
    FetchAllIcon,
    FavPlaylistIcon
  },
  emits: ['playAll', 'addToQueue', 'addToLibrary', 'playRandom', 'fetchAll', 'toggleLyrics', 'click']
})
export default class SongDetailsCompact extends mixins(ImgLoader, FileMixin, JukeboxMixin) {
  @Prop({ default: null })
  private currentSong!: Song | null | undefined

  subtitle: string = this.getConcatedSubtitle()

  FAVORITES_PLAYLIST_ID = FAVORITES_PLAYLIST_ID

  @Prop({ default: () => null })
  defaultDetails!: SongDetailDefaults | null

  @Prop({ default: () => undefined })
  private forceCover!: string

  private forceShowDefaultImage = false

  pinHoverText = false

  @Prop({
    default: () => {
      return {
        enableContainer: false,
        enableLibraryStore: false
      }
    }
  })
  buttonGroup!: SongDetailButtons

  @Prop({ default: false })
  forceWhiteText!: boolean

  @Prop({ default: '' })
  cardHoverText!: string
  parsedCardHoverText = ''

  @Prop({ default: 0 })
  isShowLyricsActive!: number

  @Prop({ default: false })
  isLoading!: boolean

  @Prop({ default: true })
  showSubSubTitle!: boolean

  @Prop({ default: true })
  scrollable!: boolean

  @Prop({ default: false })
  showPlayHoverButton!: boolean

  handleImageError() {
    this.forceShowDefaultImage = true
  }

  get computedImg() {
    if (!this.forceShowDefaultImage) {
      return (
        this.forceCover ?? this.getImgSrc(this.getValidImageHigh(this.currentSong) ?? this.defaultDetails?.defaultCover)
      )
    }
    return undefined
  }

  @Watch('cardHoverText', { immediate: true })
  private onCardHoverTextChange() {
    this.parsedCardHoverText = this.cardHoverText.replaceAll('\n', '</br>')
  }

  @Watch('defaultDetails')
  @Watch('currentSong')
  onSongchange() {
    this.subtitle = this.getConcatedSubtitle()
    this.forceShowDefaultImage = false
  }

  get subSubTitle() {
    return (
      (this.currentSong && convertDuration(this.currentSong.duration)) ?? this.defaultDetails?.defaultSubSubtitle ?? ''
    )
  }

  get title() {
    return this.currentSong?.title ?? this.defaultDetails?.defaultTitle ?? ''
  }

  private isArtistAlbumNotEmpty() {
    return !!(this.currentSong?.artists && this.currentSong.artists.length > 0 && this.currentSong?.album?.album_name)
  }

  getParsedSubtitle() {
    if (this.currentSong && (this.currentSong.artists?.length || this.currentSong.album?.album_name)) {
      return (
        ((this.currentSong?.artists && this.currentSong?.artists?.map((val) => val.artist_name).join(', ')) ?? '') +
        (this.isArtistAlbumNotEmpty() ? ' - ' : '') +
        ((this.currentSong?.album && this.currentSong.album.album_name) ?? '')
      )
    }
  }

  getConcatedSubtitle() {
    return this.getParsedSubtitle() ?? this.defaultDetails?.defaultSubtitle ?? ''
  }

  playAll() {
    this.$emit('playAll')
  }

  addToQueue() {
    this.$emit('addToQueue')
  }

  addToLibrary() {
    this.$emit('addToLibrary')
  }

  onShowLyricsClicked() {
    this.pinHoverText = true
    this.$emit('toggleLyrics')
  }

  playRandom() {
    this.$emit('playRandom')
  }

  emitClick(event: MouseEvent) {
    this.$emit('click', event)
  }

  fetchAll() {
    this.$emit('fetchAll')
  }
}
</script>

<style lang="sass" scoped>
.albumart
  border-radius: 28px
  -webkit-user-select: none
  user-select: none
  object-fit: cover

.image-container
  position: relative
  text-shadow: 0 0 white

.song-info-container
  text-align: left
  margin-top: 15px
  .song-title
    font-weight: bold
    font-size: 24px
  .song-subtitle
    font-weight: 250
    font-size: 18px
  .song-timestamp
    font-weight: 600
    font-size: 16px

.main-container
  position: absolute
  left: 0
  top: 0
  padding-top: 72px
  padding-bottom: 30px

.queue-container
  overflow-y: scroll

.right-container
  margin-left: 5rem

.bg-img
  height: 100vh
  width: 100vw
  object-fit: cover
  filter: blur(10px)
  position: absolute
  top: 0
  left: 0
  z-index: -9999

.animate__animated.animate__fadeOut
  --animate-duration: 0.3s

.scrollable
  overflow-y: scroll
  color: transparent
  transition: color 0.3s ease
  &:hover
    color: white

.hoverText
  position: absolute
  color: white
  width: 100%
  height: 100%
  top: 0
  left: 0
  border-radius: 28px
  opacity: 0
  overflow-y: overlay
  transition: opacity 0.2s ease-in-out
  text-align: left
  padding: 30px 25px 30px 25px
  background: rgba(0, 0, 0, 0.7)
  line-height: 1.5
  pre
    color: white
    font-family: 'Nunito Sans'
    font-size: 18px
    font-weight: normal
    white-space: pre-wrap
  &:hover
    opacity: 1
    backdrop-filter: blur(5px)

.hoverText.visible-always
  opacity: 1
  backdrop-filter: blur(5px)

.pin-icon
  position: absolute
  top: 0
  right: 30px
  width: 25px

.show-lyrics
  font-weight: 400
  font-size: 17px
  cursor: pointer

.loading-spinner
  position: absolute
  left:  0
  top: 0
  width: 100%
  height: 100%
  background: rgba(0, 0, 0, 0.4)
  border-radius: 16px
  span
    color: white

.play-button
  width: calc(100%)
  height: calc(100%)
  background: rgba(0, 0, 0, 0.6)
  position: absolute
  top: 0
  border-radius: 28px
  cursor: pointer

.play-button
  opacity: 0
  transition: opacity 0.2s ease
  &:hover
    opacity: 1

  svg
    width: 80px
    height: 80px
</style>
