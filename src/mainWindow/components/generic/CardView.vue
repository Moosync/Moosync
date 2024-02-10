<!-- 
  CardView.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div
    class="card mb-2 card-grow"
    @contextmenu="emitContext"
    :style="{ 'max-width': maxWidth }"
    @click="$emit('click', $event)"
  >
    <div class="card-img-top">
      <div class="icon-container" v-if="iconBgColor" :style="{ background: iconBgColor }">
        <slot name="icon" />
      </div>
      <div class="embed-responsive embed-responsive-1by1">
        <div class="embed-responsive-item img-container">
          <img
            referrerPolicy="no-referrer"
            v-if="imgSrc && !forceEmptyImg"
            :src="getImgSrc(imgSrc)"
            alt="Album Art"
            :class="[isOverlayExists ? 'overlay-base' : '']"
            class="img-fluid w-100 h-100"
            @error="handlerImageError($event as ErrorEvent, handleError)"
          />
          <div class="overlay me-auto justify-content-center d-flex align-items-center h-100 w-100">
            <slot name="overlay" />
          </div>
          <div class="default-icon" :class="[isOverlayExists ? 'overlay-base' : '']">
            <slot v-if="!imgSrc || forceEmptyImg" name="defaultCover" />
          </div>
        </div>
      </div>
    </div>
    <div class="card-body">
      <p class="card-title text-truncate" :title="title">{{ title }}</p>
      <p v-if="subtitle" class="subtitle text-truncate" :title="subtitle">{{ subtitle }}</p>
    </div>
  </div>
</template>

<script lang="ts">
import ImageLoader from '@/utils/ui/mixins/ImageLoader'
import { mixins } from 'vue-facing-decorator'
import { Component, Prop, Watch } from 'vue-facing-decorator'
import ErrorHandler from '@/utils/ui/mixins/errorHandler'
import Play2 from '@/icons/PlayIcon2.vue'

@Component({
  components: {
    Play2
  },
  emits: ['click', 'CardContextMenu']
})
export default class CardView extends mixins(ImageLoader, ErrorHandler) {
  @Prop({ default: '' })
  title!: string

  @Prop({ default: '' })
  subtitle!: string

  @Prop({ default: '' })
  private id!: string

  @Prop({ default: '' })
  imgSrc!: string

  @Prop({ default: '200px' })
  public maxWidth!: string

  @Prop({ default: '' })
  iconBgColor!: string

  forceEmptyImg = false

  get isOverlayExists(): boolean {
    return !!this.$slots.overlay
  }

  @Watch('imgSrc') onImgSrcChange() {
    this.forceEmptyImg = false
  }

  handleError() {
    this.forceEmptyImg = true
  }

  emitContext(event: Event) {
    event.stopPropagation()
    this.$emit('CardContextMenu', event)
  }
}
</script>

<style lang="sass" scoped>
.card
  padding: 8px
</style>
