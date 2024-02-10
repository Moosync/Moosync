<!-- 
  SingleSearchResult.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="single-result-container" @contextmenu.prevent="emitContextMenu($event)">
    <b-row align-h="around">
      <b-col cols="1" class="img-container justify-content-around ms-auto" @click="emitImgClick">
        <img
          referrerPolicy="no-referrer"
          v-if="!forceEmptyImg"
          ref="cover"
          class="coverimg me-auto d-flex align-items-center"
          alt="cover img"
          :src="getImgSrc(coverImg)"
          @error="handleCoverError"
        />
        <SongDefault v-else class="coverimg me-auto d-flex align-items-center" />
        <div class="play-button d-flex justify-content-center" v-if="playable">
          <Play2 class="align-self-center" />
        </div>
      </b-col>
      <b-col class="text-container text-truncate my-auto">
        <b-link class="song-title text-truncate" @click="emitTitleClick">{{ title }}</b-link>
        <div class="song-subtitle text-truncate">{{ subtitle }}</div>
      </b-col>
    </b-row>
    <b-row v-if="divider" class="divider-row d-flex no-gutters">
      <div class="divider" />
    </b-row>
    <b-row v-else class="no-gutters">
      <div class="placeholder" />
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, Prop, Watch } from 'vue-facing-decorator'
import Play2 from '@/icons/PlayIcon2.vue'
import { mixins } from 'vue-facing-decorator'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'

@Component({
  components: {
    Play2,
    SongDefault
  },
  emits: ['imgClick', 'titleClick', 'onContextMenu']
})
export default class SingleSearchResult extends mixins(PlayerControls, ImgLoader) {
  @Prop({ default: '' })
  coverImg!: string

  @Prop({ default: '' })
  title!: string

  @Prop({ default: '' })
  subtitle!: string

  @Prop({ default: false })
  divider!: boolean

  @Prop({ default: null })
  id!: string | null

  @Prop({ default: true })
  playable!: boolean

  forceEmptyImg = false

  handleCoverError() {
    this.forceEmptyImg = true
  }

  @Watch('coverImg') onCoverChange() {
    this.forceEmptyImg = false
  }

  emitImgClick() {
    if (this.playable) this.$emit('imgClick', this.id)
  }

  emitTitleClick() {
    this.$emit('titleClick', this.id)
  }

  emitContextMenu(event: Event) {
    this.$emit('onContextMenu', event, this.id)
  }
}
</script>

<style lang="sass" scoped>
.result-container
  margin-left: 40px

.img-container
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

.play-icon
  z-index: 10

.coverimg
  height: 56px
  width: 56px
  object-fit: cover

.text-container
  position: relative
  text-align: left
  font-weight: normal

.song-title
  font-size: 16px
  color: var(--textPrimary)

.song-subtitle
  font-size: 14px
  color: var(--textSecondary)

.divider
  border-bottom: 1px solid var(--divider) !important
  width: 100%

.single-result-container
  margin-top: 13px

.placeholder
  height: 13px

.divider-row
  margin-top: 13px
</style>
