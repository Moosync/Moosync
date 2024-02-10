<!-- 
  IconButton.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div
    @mouseover="hover = true"
    @mouseleave="hover = false"
    @click="$emit('click', $event)"
    class="button-bg d-flex ripple w-100"
    :style="{ backgroundColor: bgColor }"
  >
    <div class="d-flex w-100 h-100">
      <div class="icon-wrapper d-flex my-auto">
        <slot name="icon"></slot>
      </div>
      <div class="title-wrapper flex-grow-1 my-auto text-truncate">
        {{ hover && hoverText ? hoverText : title }}
      </div>
    </div>
  </div>
</template>

<script lang="ts">
import ImageLoader from '@/utils/ui/mixins/ImageLoader'
import { mixins } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'
import Youtube from '@/icons/YoutubeIcon.vue'

@Component({
  components: {
    Youtube
  },
  emits: ['click']
})
export default class IconButton extends mixins(ImageLoader) {
  @Prop({ default: '' })
  title!: string

  @Prop({ default: '' })
  bgColor!: string

  @Prop({ default: undefined })
  hoverText: string | undefined

  hover = false
}
</script>

<style lang="sass" scoped>
.button-bg
  border-radius: 5px
  width: 187px
  height: 40px
  cursor: pointer

.title-wrapper
  font-weight: bold
  font-size: 14px
  color: white
  text-align: center
  user-select: none
  margin-right: 25px
  margin-left: 8px

.icon-wrapper
  width: 26px
  margin-left: 15px
  > svg
    height: 100%
    width: 100%

.ripple
  position: relative
  overflow: hidden
  transform: translate3d(0, 0, 0)

  &:after
    content: ''
    display: block
    position: absolute
    width: 100%
    height: 100%
    top: 0
    left: 0
    pointer-events: none
    background-image: radial-gradient(circle, #000 10%, transparent 20.01%)
    background-repeat: no-repeat
    background-position: 50%
    transform: scale(10, 10)
    opacity: 0
    transition: transform 0.5s, opacity 1s

  &:active:after
    transform: scale(0, 0)
    opacity: 0.2
    transition: 0s
</style>
