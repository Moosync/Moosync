<!-- 
  SongList.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="d-flex h-100 w-100">
    <b-container fluid>
      <b-row no-gutters>
        <div ref="headers" class="wrapper w-100 headers">
          <template v-for="(field, index) of extrafields" :key="`box-${field.key}`">
            <div
              :title="field.label ? field.label : field.key"
              :style="{ width: columnWidths[index] + '%' }"
              class="box text-truncate"
            >
              {{ field.label ? field.label : field.key }}
            </div>
            <div
              v-if="index !== extrafields.length - 1"
              :key="`handler-${field.key}`"
              :id="field.key"
              class="handler"
              @mousedown="mouseDown($event, field.key)"
            ></div>
          </template>
        </div>
      </b-row>
      <b-row no-gutters class="recycler-row">
        <RecycleScroller
          class="scroller w-100 h-100"
          :items="songList"
          :item-size="64"
          key-field="_id"
          :direction="'vertical'"
          v-click-outside="clearSelection"
          @scroll-end="onScrollEnd"
        >
          <template v-slot="{ item, index }">
            <div class="wrapper w-100 field-content" :class="{ selectedItem: selected.includes(index) }">
              <div
                v-for="(field, i1) of extrafields"
                :key="`${item._id}-${field.key}`"
                class="box text-truncate"
                :style="{ width: columnWidths[i1] + '%' }"
                :title="(getFieldTitle(field.key, item, index) as string)"
                @dblclick="onRowDoubleClicked(item)"
                @click="onRowSelected(index)"
                @contextmenu="onRowContext($event, item)"
              >
                <div
                  :class="field.key === 'album_name' ? 'col-content' : ''"
                  v-if="typeof getFieldData(field.key, item, index) === 'string'"
                  @click="onTextClick(field.key, item)"
                >
                  {{ getFieldData(field.key, item, index) }}
                </div>
                <div class="d-flex" v-if="typeof getFieldData(field.key, item, index) === 'object'">
                  <div
                    v-for="(artist, i) in getFieldData(field.key, item, index)"
                    :key="i"
                    @click="onTextClick(field.key, artist as Artists)"
                    :class="field.key === 'artist_name' ? 'col-content' : ''"
                    class="ml-1"
                  >
                    {{ (artist as Artists).artist_name }}{{ index !== item.artists.length - 1 ? ',' : '' }}
                  </div>
                </div>
              </div>
            </div>
          </template>
        </RecycleScroller>
      </b-row>
    </b-container>
  </div>
</template>

<script lang="ts">
import SongListMixin from '@/utils/ui/mixins/SongListMixin'
import { mixins } from 'vue-facing-decorator'
import { Component, Prop, Ref } from 'vue-facing-decorator'
import { convertProxy } from '@/utils/ui/common'

@Component({
  emits: [
    'onRowContext',
    'onRowDoubleClicked',
    'onRowPlayNowClicked',
    'onArtistClicked',
    'onAlbumClicked',
    'onScrollEnd'
  ]
})
export default class SongList extends mixins(SongListMixin) {
  private refreshKey = false

  @Prop({ default: {} })
  extrafields!: [{ key: TableFields; label?: string }]

  @Prop({ default: false })
  isLoading!: boolean

  @Ref('headers')
  private headers!: HTMLDivElement

  getFieldData(field: TableFields, song: Song, index: number) {
    switch (field) {
      case 'index':
        return (index + 1).toString()
      case 'title':
        return song.title
      case 'album_name':
        return song.album?.album_name
      case 'artist_name':
        return song.artists
    }
  }

  getFieldTitle(field: TableFields, song: Song, index: number) {
    if (field !== 'artist_name') return this.getFieldData(field, song, index)
    else return song.artists?.map((val) => val.artist_name).join(', ')
  }

  getAlbumName(data: Song) {
    if (data.album && data.album.album_name) return data.album.album_name
    return '-'
  }

  mounted() {
    this.computeDefaultWidths()
    this.generateHandlerMap()
    this.setupMouseEvents()
  }

  beforeUnmount() {
    this.destroyMouseEvents()
  }

  private handlerMap: {
    [key: string]: {
      handler: HTMLDivElement
      next: HTMLDivElement
      prev: HTMLDivElement
      prevWidth: number
      nextWidth: number
      startPos: number
    }
  } = {}

  private activeHandlerKey?: string

  columnWidths: number[] = []

  private async computeDefaultWidths() {
    await this.loadWidths()
    if (this.columnWidths.length === 0) {
      for (const index in this.extrafields) {
        this.columnWidths[index] = 100 / this.extrafields.length - 8
      }
    }
  }

  private generateHandlerMap() {
    for (const field of this.extrafields) {
      const handler = document.querySelector(`#${field.key}`) as HTMLDivElement | null
      if (handler) {
        this.handlerMap[field.key] = {
          handler: handler,
          next: handler.nextElementSibling as HTMLDivElement,
          prev: handler.previousElementSibling as HTMLDivElement,
          prevWidth: 0,
          nextWidth: 0,
          startPos: 0
        }
      }
    }
  }

  mouseDown(e: MouseEvent, id: string) {
    const handlerData = this.handlerMap[id]
    handlerData.startPos = e.pageX
    handlerData.prevWidth = handlerData.prev.offsetWidth
    handlerData.nextWidth = handlerData.next.offsetWidth
    this.activeHandlerKey = id
  }

  private getNextField(key: string) {
    const index = this.extrafields.findIndex((val) => val.key === key)
    if (index !== -1) {
      return index + 1
    }
  }

  private mouseMove(e: MouseEvent) {
    if (!this.activeHandlerKey) {
      return
    }

    const handlerData = this.handlerMap[this.activeHandlerKey]
    const pointerRelativeXpos = e.pageX - handlerData.startPos

    const minWidth = 3

    const prevWidthP = ((handlerData.prevWidth + pointerRelativeXpos) / this.headers.offsetWidth) * 100
    const nextWidthP = ((handlerData.nextWidth - pointerRelativeXpos) / this.headers.offsetWidth) * 100

    if (nextWidthP < minWidth || prevWidthP < minWidth) {
      return
    }

    const rightI = this.getNextField(this.activeHandlerKey)

    if (rightI) {
      this.columnWidths[rightI - 1] = prevWidthP + 1
      this.columnWidths[rightI] = nextWidthP + 1
    }
  }

  private mouseUp() {
    this.activeHandlerKey = undefined
    this.saveWidths()
  }

  private setupMouseEvents() {
    document.addEventListener('mousemove', this.mouseMove)
    document.addEventListener('mouseup', this.mouseUp)
  }

  private destroyMouseEvents() {
    document.removeEventListener('mousemove', this.mouseMove)
    document.removeEventListener('mouseup', this.mouseUp)
  }

  private saveWidths() {
    return window.PreferenceUtils.saveSelective('UI.columnHeaders.widths', convertProxy(this.columnWidths), false)
  }

  private async loadWidths() {
    this.columnWidths =
      ((await window.PreferenceUtils.loadSelective('UI.columnHeaders.widths', false)) as number[]) ?? []
  }

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

  async onTextClick(key: TableFields, item: Song | Artists) {
    if (key === 'artist_name') {
      this.$emit('onArtistClicked', item)
    } else if (key === 'album_name' && typeof item !== 'string') {
      this.$emit('onAlbumClicked', (item as Song).album)
    }
  }

  private sortContent(): void {
    // TODO: Sort content without b-table sort since we have table resizers
  }

  // For some reason table isn't rerendered on window size change through maximize and minimize functions
  private rerenderTable() {
    this.refreshKey = !this.refreshKey
  }

  onScrollEnd(e: Event) {
    this.$emit('onScrollEnd', e)
  }
}
</script>

<style lang="sass">
.col-content
  text-decoration: none
  cursor: pointer
  &:hover
    text-decoration: underline
</style>
