<!-- 
  CardCarousel.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="carousel-container w-100">
    <RecycleScroller
      class="scroller-horizontal w-100 h-100"
      :items="songList"
      :item-size="230"
      key-field="_id"
      :direction="'horizontal'"
    >
      <template v-slot="{ item }">
        <div :key="item._id" @click="playSong(item)" class="card-container">
          <CardView
            :title="item.title"
            :subtitle="item.artists ? item.artists.map((val: Artists) => val.artist_name).join(', ') : ''"
            :imgSrc="getValidImageHigh(item)"
            @CardContextMenu="showContextMenu($event, item)"
          >
            <template #defaultCover> <SongDefault /></template>
            <template #overlay>
              <div class="play-icon">
                <Play2 /></div
            ></template>
          </CardView>
        </div>
      </template>
    </RecycleScroller>
  </div>
</template>

<script lang="ts">
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import { mixins } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'
import CardView from '../../components/generic/CardView.vue'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import Play2 from '@/icons/PlayIcon2.vue'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'

@Component({
  components: {
    CardView,
    SongDefault,
    Play2
  }
})
export default class CardCarousel extends mixins(ImgLoader, PlayerControls, ContextMenuMixin) {
  @Prop({ default: () => [] })
  songList!: Song[]

  playSong(song: Song) {
    this.playTop([song])
  }

  showContextMenu(event: MouseEvent, song: Song) {
    this.getContextMenu(event, {
      type: 'SONGS',
      args: {
        isRemote: true,
        songs: [song],
        refreshCallback: () => {
          // Empty Callback
        }
      }
    })
  }
}
</script>

<style lang="sass">
.play-icon > svg
  width: 40px
  height: 40px

.card-container
  width: 200px
  min-width: 200px
  margin-right: 30px

.carousel-container
  height: 300px
</style>

<style lang="sass">
.scroller-horizontal::-webkit-scrollbar
    min-height: 28px !important

.scroller-horizontal::-webkit-scrollbar-thumb
    min-width: 50px
</style>
