<!-- 
  ExtraControls.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-row align-h="end" align-v="center" no-gutters>
    <b-col
      v-if="!isJukeboxModeActive"
      cols="auto"
      class="slider-container d-flex"
      :style="{ opacity: `${showVolume ? '1' : ''}`, visibility: `${showVolume ? 'visible' : 'hidden'}` }"
      @mouseenter="handleSliderMouseEnter"
      @mouseleave="handleSliderMouseLeave"
    >
      <input
        type="range"
        min="0"
        max="100"
        class="slider w-100 align-self-center"
        v-bind:style="{
          background: ComputedGradient
        }"
        id="myRange"
        aria-label="volume"
        v-model="volume"
        @mousewheel="handleScrollEvent"
      />
    </b-col>
    <b-col cols="auto" v-if="!isJukeboxModeActive">
      <VolumeIcon
        class="volume-icon align-self-center"
        @click="muteToggle"
        :cut="volume == 0"
        @mouseover="handleVolumeIconMouseEnter"
        @mouseleave="handleVolumeIconMouseLeave"
        @mousewheel="handleScrollEvent"
      />
    </b-col>
    <b-col cols="auto" class="expand-icon ml-3" :class="{ open: sliderOpen }" @click="emitToggleSlider">
      <ExpandIcon />
    </b-col>
  </b-row>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import VolumeIcon from '@/icons/VolumeIcon.vue'
import ExpandIcon from '@/icons/ExpandIcon.vue'
import Timestamp from '@/mainWindow/components/musicbar/components/Timestamp.vue'
import { bus } from '@/mainWindow/main'
import { mixins } from 'vue-facing-decorator'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import JukeboxMixin from '@/utils/ui/mixins/JukeboxMixin'

@Component({
  components: {
    VolumeIcon,
    ExpandIcon,
    Timestamp
  }
})
export default class ExtraControls extends mixins(PlayerControls, JukeboxMixin) {
  sliderOpen = false

  volumeIconHover = false
  showVolume = false

  emitToggleSlider() {
    bus.emit('onToggleSlider')
  }

  mounted() {
    bus.on('onToggleSlider', (val: boolean) => {
      if (typeof val !== 'undefined') {
        this.sliderOpen = val
      } else {
        this.sliderOpen = !this.sliderOpen
      }

      bus.emit('onToggleSliderWindow', this.sliderOpen)
    })
  }

  get ComputedGradient(): string {
    return `linear-gradient(90deg, var(--accent) 0%, var(--accent) ${this.volume}%, var(--textSecondary) 0%)`
  }

  handleVolumeIconMouseEnter() {
    this.volumeIconHover = true
    this.showVolume = true
  }

  private leaveTimeout: ReturnType<typeof setTimeout> | undefined

  handleVolumeIconMouseLeave() {
    this.volumeIconHover = false

    this.leaveTimeout = setTimeout(() => {
      this.showVolume = false
    }, 150)
  }

  handleSliderMouseEnter() {
    if (this.volumeIconHover) {
      this.showVolume = true
    }
    this.leaveTimeout && clearTimeout(this.leaveTimeout)
  }

  handleSliderMouseLeave() {
    this.showVolume = false
    this.leaveTimeout && clearTimeout(this.leaveTimeout)
  }

  handleScrollEvent(e: WheelEvent) {
    if (e.deltaY < 0) {
      this.volume += 3
    } else {
      this.volume -= 3
    }
  }
}
</script>

<style lang="sass" scoped>
.slider-container
  padding-right: 20px

.slider
  right: 0
  -webkit-appearance: none
  height: 2px
  outline: none
  visibility: visible

.slider::-webkit-slider-thumb
  -webkit-appearance: none
  appearance: none
  width: 12px
  height: 12px
  border-radius: 50%
  background: var(--accent)

.slider::-ms-fill-upper
  background-color: var(--primary)

.volume-icon
  width: 22px
  &:hover
    .slider-container
      opacity: 1
      display: block

.expand-icon
  display: block
  height: 27px
  width: 18px
  transition: transform 0.2s linear

.open
  transform: rotate(180deg)

.test
  min-width: 0

@media only screen and (max-width : 565px)


@media only screen and (max-width : 800px)
  .expand-icon
    display: none

  .slider-container
    right: -232px !important

@media only screen and (max-width : 992px)
  .slider-container
    right: -200px
    bottom: 85px
    height: 40px
    width: 175px
    max-width: 175px
    position: absolute
    background: var(--tertiary)
    border-radius: 16px
    padding-left: 15px !important
    transform: rotate(-90deg)
    transform-origin: 0px 88px
    transition: opacity 0.2s ease-in-out
    z-index: 9999
    opacity: 0
</style>
