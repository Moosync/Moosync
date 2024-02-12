<!-- 
  index.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="h-100 w-100 parent" @contextmenu="contextHandler">
    <CardRecycleScroller :title="$t('pages.albums')" :itemList="filteredAlbumList" :titleKey="'album_name'"
      :imageKey="'album_coverPath_high'" :keyField="'album_id'" @click="gotoAlbum"
      @CardContextMenu="(event: MouseEvent, album: Album) => singleItemContextHandler(event, album)"
      @generalContextMenu="contextHandler" :isSortAsc="isSortAsc">
      <template #defaultCover>
        <AlbumDefault />
      </template>
    </CardRecycleScroller>
  </div>
</template>

<script lang="ts">
import CardRecycleScroller from '@/mainWindow/components/generic/CardRecycleScroller.vue'
import { mixins } from 'vue-facing-decorator'
import { Component } from 'vue-facing-decorator'
import RouterPushes from '@/utils/ui/mixins/RouterPushes'
import AlbumDefault from '@/icons/AlbumDefaultIcon.vue'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import { vxm } from '@/mainWindow/store'

@Component({
  components: {
    CardRecycleScroller,
    AlbumDefault
  }
})
export default class Albums extends mixins(RouterPushes, ContextMenuMixin) {
  private albumList: Album[] = []

  private async getAlbums() {
    this.albumList = await window.SearchUtils.searchEntityByOptions({
      album: {}
    })
    this.sort()
  }

  get filteredAlbumList() {
    return this.albumList.filter((x) => {
      return x.album_name !== null
    })
  }

  get isSortAsc() {
    return vxm.themes.entitySortBy?.asc ?? true
  }

  private setSort(options: NormalSortOptions) {
    vxm.themes.entitySortBy = options
  }

  private sort() {
    this.albumList.sort((a, b) => {
      switch (vxm.themes.entitySortBy.type) {
        default:
        case 'name':
          return (
            (vxm.themes.entitySortBy.asc
              ? a.album_name?.localeCompare(b.album_name ?? '')
              : b.album_name?.localeCompare(a.album_name ?? '')) ?? 0
          )
      }
    })
  }

  public singleItemContextHandler(event: MouseEvent, album: Album) {
    this.getContextMenu(event, {
      type: 'ALBUM',
      args: {
        album,
        refreshCallback: this.getAlbums
      }
    })
  }

  public contextHandler(event: MouseEvent) {
    this.getContextMenu(event, {
      type: 'ENTITY_SORT',
      args: {
        sortOptions: {
          callback: this.setSort,
          current: vxm.themes.entitySortBy
        }
      }
    })
  }

  mounted() {
    this.getAlbums()
    vxm.themes.$watch('entitySortBy', this.sort)
  }
}
</script>
