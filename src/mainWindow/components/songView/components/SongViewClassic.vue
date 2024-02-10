<!-- 
  SongViewClassic.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-row class="w-100 h-100">
    <b-col class="w-100 h-100">
      <b-row no-gutters align-v="center" class="details-background">
        <SongDetails
          class="details-container h-100"
          :defaultDetails="defaultDetails"
          :buttonGroup="detailsButtonGroup"
          :currentSong="currentSong"
          :optionalProviders="optionalProviders"
          v-bind="$attrs"
        />
      </b-row>
      <b-row no-gutters class="list-container">
        <SongList
          :songList="songList"
          :extrafields="[
            { key: 'index', label: 'Sr. No' },
            { key: 'title', label: 'Title' },
            { key: 'album_name', label: 'Album' },
            { key: 'artist_name', label: 'Artists' }
          ]"
          :isLoading="isLoading"
          v-bind="$attrs"
        />
      </b-row>
    </b-col>
  </b-row>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import SongList from './SongList.vue'
import SongDetails from './SongDetails.vue'
import ModelHelper from '@/utils/ui/mixins/ModelHelper'

@Component({
  components: {
    SongList,
    SongDetails
  },
  options: {
    compatConfig: {
      INSTANCE_LISTENERS: false
    }
  }
})
export default class SongViewClassic extends mixins(ImgLoader, ModelHelper) {
  @Prop({ default: () => [] })
  songList!: Song[]

  @Prop({ default: false })
  currentSong!: Song | undefined | null

  @Prop({ default: false })
  isLoading!: boolean

  @Prop({ default: () => [] })
  optionalProviders!: TabCarouselItem[]

  @Prop({
    default: () => {
      return { defaultTitle: '', defaultSubtitle: '', defaultCover: '' }
    }
  })
  defaultDetails!: SongDetailDefaults

  @Prop({
    default: () => {
      return {
        enableContainer: false,
        enableLibraryStore: false,
        playRandom: false
      }
    }
  })
  detailsButtonGroup!: SongDetailButtons
}
</script>

<style lang="sass" scoped>
.details-background
  height: 25%
  max-height: 200px
  margin-top: 15px
  margin-left: 15px
  margin-right: 15px
  border-radius: 28px
  background: var(--secondary)

.details-container
  width: 100%

.list-container
  height: 75%
</style>
