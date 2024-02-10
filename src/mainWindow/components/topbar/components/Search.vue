<!-- 
  Search.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="h-100 d-flex align-items-center search-container">
    <div class="w-100 searchbar-container" :class="showSearchResults ? 'half-border' : 'full-border'">
      <SearchIcon class="search-icon" />
      <b-form-input
        class="searchbar"
        :placeholder="$t('topbar.searchPlaceholder')"
        type="text"
        v-model="inputText"
        debounce="300"
        ref="inputfield"
        @update="onTextChange"
        @blur="handleInputFocus"
        @focus="handleInputFocus"
        @keyup.enter="openSearchPage"
      />
      <AltArrowIcon @click="openSearchPage" v-if="inputText !== ''" class="go-arrow button-grow" />
    </div>
    <div class="search-results d-flex" :class="showSearchResults ? 'search-visible' : 'search-invisible'">
      <div v-if="results && results.length !== 0" class="w-100">
        <RecycleScroller
          class="scroller"
          :items="results"
          :item-size="83"
          key-field="_id"
          v-slot="{ item, index }"
          :direction="'vertical'"
        >
          <SingleSearchResult
            class="single-result"
            :title="item.title"
            :subtitle="item.artists ? item.artists.map((val: Artists) => val.artist_name).join(', ') : ''"
            :coverImg="getImgSrc(getValidImageLow(item))"
            :divider="index != results.length - 1"
            :id="index"
            @imgClick="handleClick"
          />
        </RecycleScroller>
      </div>
      <div class="w-100 text-center" v-else>{{ $t('topbar.noResultsFound') }}</div>
    </div>
  </div>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import SearchIcon from '@/icons/SearchIcon.vue'
import AltArrowIcon from '@/icons/AltArrowIcon.vue'

import SingleSearchResult from '@/mainWindow/components/generic/SingleSearchResult.vue'
import { mixins } from 'vue-facing-decorator'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'

@Component({
  components: {
    SearchIcon,
    SingleSearchResult,
    AltArrowIcon
  }
})
export default class Sidebar extends mixins(PlayerControls, ImgLoader) {
  showSearchResults = false
  results: Song[] = []
  inputText = ''

  handleInputFocus(event: FocusEvent) {
    switch (event.type) {
      case 'blur':
        this.showSearchResults = false
        break
      case 'focus':
        this.showSearchResults = this.results.length > 0 ? true : false
        break
    }
  }

  handleClick(index: number) {
    this.playTop([this.results[index]])
  }

  openSearchPage() {
    this.$router
      .push({
        name: 'search',
        query: {
          search_term: `${this.inputText}`,
          timestamp: Date.now().toString()
        }
      })
      .catch((e) => console.error(e))
    this.showSearchResults = false
  }
  async onTextChange(value: string) {
    if (value) {
      value = `%${value}%`
      this.showSearchResults = true
      this.results = await window.SearchUtils.searchSongsByOptions({
        album: {
          album_name: value
        },
        artist: {
          artist_name: value
        },
        genre: {
          genre_name: value
        },
        playlist: {
          playlist_name: value
        },
        song: {
          title: value,
          path: value
        }
      })
    } else {
      this.showSearchResults = false
      this.results = []
    }
  }
}
</script>

<style lang="sass" scoped>
.searchbar
  color: var(--textPrimary) !important
  background: rgba(0, 0, 0, 0)
  border: none
  height: 24px
  margin-top: -12px
  width: calc(100% - 24px - 18px - 15px - 30px)
  position: absolute
  transition: background 0.3s cubic-bezier(0.39, 0.58, 0.57, 1), border-radius 1000ms
  text-align: left
  top: 50%
  box-shadow: none
  margin-left: calc(24px + 18px)
  &::-webkit-input-placeholder
    color: var(--textSecondary)
  &:focus
    background: rgba(0, 0, 0, 0) !important
    outline: 0

.full-border
  border-radius: 58px

.half-border
  border-radius: 18px 18px 0 0
  box-shadow: 0px -2px 17px 0px rgb(0 0 0 / 27%) !important

.search-icon
  position: absolute
  height: 24px
  top: 50%
  left: 0
  margin-top: -12px
  margin-left: 15px

.go-arrow
  position: absolute
  height: 20px
  top: 50%
  right: 0
  margin-top: -12px
  margin-right: 20px

.search-results
  position: absolute
  top: 50px
  padding: 0.375rem 0.75rem
  width: 100%
  background: var(--secondary)
  border-radius: 0 0 18px 18px
  box-shadow: -1px 12px 40px -5px rgb(0 0 0 / 50%)
  max-height: 60vh
  overflow: hidden

.search-invisible
  visibility: hidden
  opacity: 0
  transition: visibility 0s linear 300ms, opacity 300ms

.search-visible
  visibility: visible
  opacity: 1
  transition: visibility 0s linear 0s, opacity 300ms

.searchbar-container
  height: 50px
  background: var(--secondary)
  position: relative

.search-container
  position: relative
</style>
