<!-- 
  index.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="h-100 w-100 parent">
    <CardRecycleScroller
      :title="$t('pages.genres')"
      :itemList="genresList"
      :titleKey="'genre_name'"
      :keyField="'genre_id'"
      :isSortAsc="isSortAsc"
      @click="gotoGenre"
    >
      <template #defaultCover>
        <SongDefault />
      </template>
    </CardRecycleScroller>
  </div>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import CardRecycleScroller from '@/mainWindow/components/generic/CardRecycleScroller.vue'
import { mixins } from 'vue-facing-decorator'
import RouterPushes from '@/utils/ui/mixins/RouterPushes'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import { vxm } from '@/mainWindow/store'

@Component({
  components: {
    CardRecycleScroller,
    SongDefault
  }
})
export default class Genres extends mixins(RouterPushes) {
  public genresList: Genre[] = []
  private async getGenres() {
    this.genresList = await window.SearchUtils.searchEntityByOptions({
      genre: {}
    })
  }

  get isSortAsc() {
    return vxm.themes.entitySortBy?.asc ?? true
  }

  mounted() {
    this.getGenres()
  }
}
</script>

<style lang="sass" scoped>
.title
  font-weight: bold
  font-size: 55px
  margin-left: 15px
  margin-bottom: 50px
  margin-top: 20px
</style>
