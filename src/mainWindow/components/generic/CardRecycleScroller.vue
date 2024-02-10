<!-- 
  CardRecycleScroller.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="container-with-title w-100">
    <b-row no-gutters class="page-title">
      <b-col cols="auto">{{ title }} </b-col>
      <b-col class="align-self-center">
        <TabCarousel @onSortClicked="sortMenuHandler" @onSearchChange="onSearchChange" :isSortAsc="isSortAsc" />
      </b-col>
    </b-row>
    <b-row class="recycle-row" ref="scrollerRow">
      <RecycleScroller class="scroller w-100 h-100" :items="filteredItems" :item-size="itemWidth"
        :itemSecondarySize="itemWidth" :key-field="keyField" :grid-items="itemsInRow" :buffer="100"
        :direction="'vertical'" @resize="onScrollerResize">
        <template v-slot="{ item }">
          <CardView :title="item[titleKey]" :imgSrc="item[imageKey]" :maxWidth="`${itemWidth - 50}px`"
            @click="emitClick(item)" @CardContextMenu="(event: unknown, ...args: unknown[]) =>  emitContextMenu(event, item, ...args)">
            <template #defaultCover>
              <slot ref="defaultCover" name="defaultCover" />
            </template>
          </CardView>
        </template>
      </RecycleScroller>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import CardView from '@/mainWindow/components/generic/CardView.vue'
import { Component, Prop, Ref } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'
import TabCarousel from '../../components/generic/TabCarousel.vue'

@Component({
  components: {
    CardView,
    TabCarousel
  },
  emits: ['generalContextMenu', 'CardContextMenu', 'click']
})
export default class CardRecycleScroller extends Vue {
  @Prop({ default: () => [] })
  public itemList!: Record<string, string>[]

  @Prop({ default: '' })
  public keyField!: string

  @Prop({ default: '' })
  public titleKey!: string

  @Prop({ default: '' })
  public imageKey!: string

  @Prop({ default: '' })
  public title!: string

  @Prop({ default: true })
  public isSortAsc!: boolean

  @Ref('scrollerRow')
  private scrollerRow!: HTMLDivElement

  get filteredItems() {
    return this.itemList.filter((val) => val[this.titleKey].toLowerCase().includes(this.searchText))
  }

  private searchText = ''
  onSearchChange(searchText: string) {
    this.searchText = searchText ?? ''
  }

  sortMenuHandler(event: MouseEvent) {
    this.$emit('generalContextMenu', event)
  }

  private totalWidth = 0
  public itemWidth = 250
  public itemsInRow = 1

  public emitClick(item: unknown) {
    this.$emit('click', item)
  }

  public emitContextMenu(item: unknown, ...args: unknown[]) {
    this.$emit('CardContextMenu', item, ...args)
  }

  mounted() {
    this.totalWidth = this.scrollerRow.clientWidth - 30 // 30 for scrollbar width
  }

  public onScrollerResize() {
    const minItemWidth = 220
    this.itemsInRow = Math.floor(this.totalWidth / minItemWidth)
    this.itemWidth = this.totalWidth / this.itemsInRow
    this.totalWidth = this.scrollerRow.clientWidth
  }
}
</script>

<style lang="sass" scoped>
.recycle-row
  height: calc(100% - 50px - 20px - 55px - 30px) !important

.container-with-title
  height: calc(100% - 20px) !important
</style>
