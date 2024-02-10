<!-- 
  SongListCompact.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="d-flex h-100 w-100">
    <b-container fluid>
      <TabCarousel class="tab-carousel" v-bind="$attrs" :items="optionalProviders" :isSortAsc="isSortAsc" />
      <b-row v-if="isLoading">
        <b-col class="mb-2">
          <b-spinner>{{ $t('loading') }}</b-spinner>
        </b-col>
      </b-row>
      <b-row no-gutters class="h-100">
        <RecycleScroller :class="`scroller w-100  ${isLoading ? 'low-height' : 'full-height'}`" :items="songList"
          :item-size="94" key-field="_id" :direction="'vertical'" @scroll-end="onScrollEnd"
          v-click-outside="clearSelection">
          <template v-slot="{ item, index }">
            <SongListCompactItem :item="item" :index="index" :selected="selected" @onRowDoubleClicked="onRowDoubleClicked"
              @onRowSelected="onRowSelected" @onRowContext="onRowContext" @onPlayNowClicked="onPlayNowClicked"
              @onArtistClicked="onArtistClicked" />
          </template>
        </RecycleScroller>
      </b-row>
    </b-container>
  </div>
</template>

<script lang="ts">
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import SongListMixin from '@/utils/ui/mixins/SongListMixin'
import { mixins } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'
import SongListCompactItem from './SongListCompactItem.vue'
import EllipsisIcon from '@/icons/EllipsisIcon.vue'
import TabCarousel from '../../generic/TabCarousel.vue'
import { vxm } from '@/mainWindow/store'

@Component({
  components: {
    SongListCompactItem,
    EllipsisIcon,
    TabCarousel
  },
  emits: ['onRowContext', 'onRowDoubleClicked', 'onRowPlayNowClicked', 'onArtistClicked', 'onScrollEnd'],
  options: {
    compatConfig: {
      INSTANCE_LISTENERS: false
    }
  }
})
export default class SongListCompact extends mixins(ImgLoader, SongListMixin) {
  @Prop({ default: () => [] })
  optionalProviders!: TabCarouselItem[]

  @Prop({ default: false })
  isLoading!: boolean

  onRowContext(event: Event, item: Song) {
    this.$emit(
      'onRowContext',
      event,
      this.selected.length > 1 ? this.selected.map((val) => this.songList[val]) : [item]
    )
  }

  onRowDoubleClicked(item: Song) {
    this.$emit('onRowDoubleClicked', item)
  }

  onPlayNowClicked(item: Song) {
    this.$emit('onRowPlayNowClicked', item)
  }

  onArtistClicked(item: Artists) {
    this.$emit('onArtistClicked', item)
  }

  onScrollEnd(e: Event) {
    this.$emit('onScrollEnd', e)
  }

  get isSortAsc() {
    return vxm.themes.songSortBy?.[0]?.asc ?? true
  }
}
</script>

<style lang="sass" scoped>
.scroller
  color: var(--textPrimary)
  transition: color 0.3s ease

.full-height
  height: calc(100% - 40px - 13px)

.tab-carousel
  margin-bottom: 15px
</style>
