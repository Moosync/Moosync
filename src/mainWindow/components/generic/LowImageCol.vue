<!-- 
  LowImageCol.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div cols="auto" class="img-container justify-content-around ms-auto" @click="emitClick">
    <img
      referrerPolicy="no-referrer"
      v-if="!forceEmptyImg"
      ref="cover"
      class="coverimg me-auto d-flex align-items-center"
      alt="cover img"
      :style="{ height, width }"
      :src="getImgSrc(src)"
      @error="handleCoverError"
    />
    <SongDefault :style="{ height, width }" v-else class="coverimg me-auto d-flex align-items-center" />
    <div class="play-button d-flex justify-content-center" v-if="showPlayHoverButton">
      <Play2 class="align-self-center" />
    </div>

    <div v-if="showEqualizer" class="equalizer-bg d-flex justify-content-center">
      <AnimatedEqualizer :isRunning="isSongPlaying" class="animated-playing" />
    </div>
  </div>
</template>

<script lang="ts">
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import { mixins, Component } from 'vue-facing-decorator'
import Play2 from '@/icons/PlayIcon2.vue'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import { Prop, Watch } from 'vue-facing-decorator'
import AnimatedEqualizer from '@/icons/AnimatedEqualizerIcon.vue'

@Component({
  components: {
    Play2,
    SongDefault,
    AnimatedEqualizer
  },
  emits: ['imgClicked']
})
export default class LowImageCol extends mixins(ImgLoader) {
  @Prop({ default: '' })
  src!: string

  @Prop({ default: '' })
  height!: string

  @Prop({ default: '' })
  width!: string

  @Prop({ default: false })
  showEqualizer!: boolean

  @Prop({ default: false })
  isSongPlaying!: boolean

  @Prop({ default: true })
  showPlayHoverButton!: boolean

  forceEmptyImg = false

  @Watch('src')
  onSrcChange() {
    this.forceEmptyImg = false
  }

  handleCoverError() {
    this.forceEmptyImg = true
  }

  emitClick(event: MouseEvent) {
    this.$emit('imgClicked', event)
  }
}
</script>

<style lang="sass" scoped>
.img-container
  position: relative
  margin-right: 20px
  .coverimg
    border-radius: 10px

.play-button
  width: calc(80px - (12px * 2))
  height: calc(80px - (12px * 2))
  background: rgba(0, 0, 0, 0.6)
  position: absolute
  top: 0
  border-radius: 10px
  cursor: pointer

.play-button
  opacity: 0
  transition: opacity 0.2s ease
  &:hover
    opacity: 1

.coverimg
  object-fit: cover

.animated-playing
  padding-top: 14px
  padding-bottom: 14px


.equalizer-bg
  width: calc(80px - (12px * 2))
  height: calc(80px - (12px * 2))
  background: rgba(0, 0, 0, 0.6)
  position: absolute
  top: 0
  border-radius: 10px
</style>
