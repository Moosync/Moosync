<template>
  <div class="position-relative">
    <div class="h-100 w-100 img-container">
      <transition
        name="custom-fade"
        enter-active-class="animate__animated animate__fadeIn"
        leave-active-class="animate__animated animate__fadeOut animate__faster"
      >
        <video
          v-if="showSpotifyCanvas && spotifyCanvas"
          class="bg-img w-100 h-100"
          :src="spotifyCanvas"
          :key="spotifyCanvas"
          autoplay
          loop
        />

        <b-img
          class="bg-img w-100 h-100"
          v-else-if="computedImg"
          :src="computedImg"
          :key="computedImg"
          referrerPolicy="no-referrer"
        ></b-img>
      </transition>
      <div v-if="computedImg" class="dark-overlay" :style="{ top: !hasFrame ? '-28px' : '0px' }"></div>
    </div>
    <b-container fluid class="h-100 w-100">
      <b-row class="h-100">
        <b-col class="song-details h-100">
          <b-row no-gutters class="h-100">
            <b-col class="h-100">
              <b-row align-h="start" align-v="center">
                <b-col cols="auto" class="w-100 d-flex">
                  <div
                    id="musicbar-title"
                    :title="!!currentSong ? currentSong.title : '-'"
                    class="text song-title text-truncate mr-2"
                  >
                    {{ !!currentSong ? currentSong.title : '-' }}
                  </div>

                  <YoutubeIcon
                    v-if="iconType === 'YOUTUBE'"
                    :color="'#E62017'"
                    :filled="true"
                    :dropShadow="true"
                    class="provider-icon"
                  />
                  <SpotifyIcon
                    v-if="iconType === 'SPOTIFY'"
                    :color="'#1ED760'"
                    :filled="true"
                    :dropShadow="true"
                    class="provider-icon"
                  />

                  <inline-svg
                    class="provider-icon"
                    v-if="iconURL && iconType === 'URL' && iconURL.endsWith('svg')"
                    :src="iconURL"
                  />
                  <img
                    referrerPolicy="no-referrer"
                    v-if="iconURL && iconType === 'URL' && !iconURL.endsWith('svg')"
                    :src="iconURL"
                    alt="provider icon"
                    class="provider-icon"
                  />
                </b-col>
              </b-row>
              <b-row no-gutters>
                <b-col class="d-flex" v-if="currentSong">
                  <div
                    v-for="(artist, index) of currentSong.artists"
                    :key="index"
                    :title="artist.artist_name"
                    class="text song-subtitle text-truncate"
                    :class="index !== 0 ? 'ml-1' : ''"
                  >
                    {{ artist.artist_name }}{{ index !== (currentSong.artists?.length ?? 0) - 1 ? ',' : '' }}
                  </div>
                </b-col>
              </b-row>
              <b-row no-gutters class="lyrics-container" ref="lyrics-container" @scroll="onLyricsScroll">
                <b-col class="lyrics-holder" :style="{ 'background-image': lyricsGradient }">
                  <p class="lyrics" v-if="lyrics" v-html="lyrics"></p>
                </b-col>
              </b-row>
            </b-col>
          </b-row>
        </b-col>
      </b-row>
    </b-container>
    <div class="side-decoration"></div>
  </div>
</template>

<script lang="ts">
import { vxm } from '@/mainWindow/store'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import { mixins } from 'vue-facing-decorator'
import { Component, Ref, Watch } from 'vue-facing-decorator'
import SpotifyIcon from '@/icons/SpotifyIcon.vue'
import YoutubeIcon from '@/icons/YoutubeIcon.vue'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'

@Component({
  components: {
    SpotifyIcon,
    YoutubeIcon
  }
})
export default class App extends mixins(ImgLoader) {
  hasFrame = false
  forceDefaultImg = false

  iconType = ''
  iconURL = ''

  private scrollTop = 0
  private scrollHeight = 0
  private lyricsHeight = 0

  get spotifyCanvas() {
    return vxm.themes.currentSpotifyCanvas
  }

  get showSpotifyCanvas() {
    return vxm.themes.showSpotifyCanvas
  }

  @Ref('lyrics-container')
  private lyricsContainer?: HTMLDivElement

  private lyricsRaw?: string = ''

  get lyrics() {
    return this.lyricsRaw?.replaceAll('\n', '<br/>').trim()
  }

  get currentSong() {
    return vxm.player.currentSong
  }

  get computedImg() {
    this.forceDefaultImg = false
    return this.getImgSrc(this.getValidImageHigh(this.currentSong))
  }

  async created() {
    this.hasFrame = await window.WindowUtils.hasFrame()
    this.iconType = (await this.getIconType()) ?? ''
    this.listenLyricsChanged()
  }

  mounted() {
    this.onLyricsScroll()
  }

  private async getIconType() {
    this.iconURL = ''

    if (this.currentSong) {
      if (this.currentSong.icon) {
        this.iconURL = 'media://' + this.currentSong.icon
        return 'URL'
      }

      if (this.currentSong.providerExtension) {
        const icon = await window.ExtensionUtils.getExtensionIcon(this.currentSong.providerExtension)
        if (icon) {
          this.iconURL = 'media://' + icon
          return 'URL'
        }
      }

      return this.currentSong.type
    }

    return ''
  }

  @Watch('currentSong')
  private async onCurrentSongChange() {
    this.iconType = (await this.getIconType()) ?? ''
    this.lyricsContainer?.scrollTo({ top: 0, behavior: 'smooth' })
  }

  // TODO: Better gradient calculations
  get lyricsGradient() {
    const adder = (600 - this.lyricsHeight) / 30
    return `linear-gradient(transparent ${this.scrollTop > 0 ? '12%' : '0%'}, currentColor ${
      this.scrollTop > 0 ? 30 + adder + '%' : '0%'
    }, currentColor ${this.scrollTop < this.scrollHeight ? 70 - adder + '%' : '100%'}, transparent ${
      this.scrollTop < this.scrollHeight ? '86%' : '100%'
    })`
  }

  onLyricsScroll() {
    this.scrollTop = this.lyricsContainer?.scrollTop ?? 0
    this.scrollHeight =
      (this.lyricsContainer && this.lyricsContainer.scrollHeight - this.lyricsContainer.clientHeight) ?? 0
    this.lyricsHeight = this.lyricsContainer?.clientHeight ?? 0
  }

  listenLyricsChanged() {
    this.lyricsRaw = this.currentSong?.lyrics
    bus.on(EventBus.REFRESH_LYRICS, (lyrics: string) => {
      this.lyricsRaw = lyrics
    })
  }
}
</script>

<style lang="sass" scoped>
.img-container
  position: absolute

.bg-img
  border-radius: 0px !important
  filter: blur(10px)
  object-fit: cover

.dark-overlay
  height: calc(100% + 28px + 5px + 3px)
  width: 100vw
  position: absolute
  left: 0
  background: rgba(0,0,0,.55)

.song-details
  text-align: left
  margin-top: 30px
  padding-left: 30px

.text
  text-align: left
  font-weight: normal
  line-height: 170.19%

.song-title
  font-weight: 700
  font-size: 21.434px
  width: fit-content

.song-subtitle
  font-weight: 300
  font-size: 16.0755px
  width: fit-content
  text-decoration: none
  &:hover
    text-decoration: underline

.lyrics-holder
  font-weight: 700
  font-size: 24px
  -webkit-background-clip: text
  background-attachment: fixed
  p
    color: transparent

.lyrics
  word-break: break-word
  line-height: 1.2

.lyrics-container
  overflow-y: auto
  overflow-x: hidden
  margin-top: 20px
  margin-right: -20px !important
  height: calc(100% - 80px - 15px - 20px)

.side-decoration
  position: absolute
  top: 7.8rem
  left: 0
  background: var(--accent)
  width: 8px
  height: 20vh
</style>
