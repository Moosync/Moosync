<!-- 
  SmallSongItem.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-row no-gutters class="single-song-item">
    <LowImageCol
      class="img-col"
      height="56px"
      width="56px"
      :src="getValidImageLow(item)"
      @imgClicked="playTop([item])"
    />
    <b-col cols="5" class="ml-2 mr-2" align-self="center">
      <b-row no-gutters align-v="center">
        <b-col cols="auto" class="d-flex">
          <div class="title text-truncate mr-2">
            {{ item.title }}
          </div>

          <IconHandler :item="item" />
        </b-col>
      </b-row>
      <b-row no-gutters class="flex-nowrap">
        <div
          v-for="(artist, index) in item.artists"
          :key="index"
          class="subtitle text-truncate"
          :class="index !== 0 ? 'ml-1' : ''"
        >
          <span> {{ artist.artist_name }}{{ index !== (item.artists?.length ?? 0) - 1 ? ',' : '' }}</span>
        </div>
      </b-row>
    </b-col>
    <b-col class="align-self-center" cols>
      <span
        v-for="(i, index) in formattedDuration(item.duration)"
        :key="index"
        :class="`${index === 0 ? 'playtime' : 'playtime-suffix'}`"
      >
        {{ i }}
      </span>
    </b-col>
  </b-row>
</template>

<script lang="ts">
import { convertDuration } from '@/utils/common'
import ImageLoader from '@/utils/ui/mixins/ImageLoader'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import { mixins } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'
import IconHandler from './IconHandler.vue'
import LowImageCol from './LowImageCol.vue'

@Component({
  components: {
    IconHandler,
    LowImageCol
  }
})
export default class SmallSongItem extends mixins(ImageLoader, PlayerControls) {
  @Prop({ default: () => ({}) })
  item!: Song

  formattedDuration(duration: number) {
    const formatted = convertDuration(duration)
    switch (formatted.split(':').length) {
      case 2:
        return [formatted, 'Mins']
      case 3:
        return [formatted, 'Hours']
      default:
        return [formatted, 'Secs']
    }
  }
}
</script>

<style lang="sass" scoped>
.title
  color: var(--textPrimary)
  font-weight: bold
  font-size: 16px

.playtime
  color: var(--accent)
  font-weight: 700
  margin-right: 3px

.playtime-suffix
  font-weight: 700
</style>
