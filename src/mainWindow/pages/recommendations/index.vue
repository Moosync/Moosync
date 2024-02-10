<!-- 
  index.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="h-100 w-100 parent">
    <b-container class="recommendations-container" fluid>
      <b-row no-gutters class="page-title">{{ $t('pages.explore') }}</b-row>

      <b-row no-gutters>
        <b-col class="d-flex total-listen-time">
          {{ $t('explore.you_listened_to') }}
          <div :class="`${index === 0 ? 'ml-2 mr-2' : 'mr-2'} total-listen-time-item`"
            v-for="(i, index) in totalListenTime" :key="i">
            {{ i }}
          </div>
          {{ $t('explore.of_music') }}
        </b-col>
      </b-row>
      <b-row no-gutters v-if="analyticsSongs.length > 0" class="analytics">
        <b-col>
          <b-row no-gutters>
            <b-col class="big-song">
              <SongDetailsCompact :currentSong="analyticsSongs[0]" :showSubSubTitle="false" :scrollable="false"
                :showPlayHoverButton="true" @click="playTop([analyticsSongs[0]])" />
            </b-col>
            <b-col cols="auto">
              <div class="played-for">{{ $t('explore.played_for') }}</div>
              <div v-for="(item, index) in maxPlayTimeFormatted" :key="index" class="d-flex">
                <span v-for="(i, index) in item" :key="i"
                  :class="`${index === 0 ? 'playtime' : 'playtime-suffix'} big-playtime`">
                  {{ i }}
                </span>
              </div>
            </b-col>
          </b-row>
        </b-col>
        <b-col cols="3" class="small-song-first">
          <SmallSongItem v-for="item in firstFourAnalytics" :item="item" :key="item._id" />
        </b-col>
        <b-col cols="3" class="small-song-second">
          <SmallSongItem v-for="item in secondFourAnalytics" :item="item" :key="item._id" />
        </b-col>
      </b-row>

      <b-row v-for="p of providers" :key="p.key">
        <b-col v-if="hasRecommendations(p.key) || loadingMap[p.key]">
          <b-row align-v="center" class="mt-3">
            <b-col cols="auto" class="provider-title">{{ $t('explore.hot_from') }} {{ p.Title }}</b-col>
            <b-col cols="2" class="d-flex button-group mt-1" v-if="hasRecommendations(p.key)">
              <PlainPlay v-if="!isJukeboxModeActive" :title="$t('buttons.playSingle', { title: p.Title })"
                @click="playAll(p.key)" />
              <AddToQueue :title="$t('buttons.addToQueue', { title: p.Title })" @click="addToQueue(p.key)" />
              <AddToLibrary :title="$t('buttons.addToLibrary', { title: p.Title })" @click="addToLibrary(p.key)" />
            </b-col>
            <b-col cols="auto" v-if="loadingMap[p.key]">
              <div class="loading-spinner d-flex justify-content-center">
                <b-spinner class="align-self-center" />
              </div>
            </b-col>
          </b-row>
          <b-row class="slider-row">
            <b-col v-if="hasRecommendations(p.key)">
              <CardCarousel :songList="recommendationList[p.key]" />
            </b-col>
          </b-row>
        </b-col>
      </b-row>
    </b-container>
  </div>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import RouterPushes from '@/utils/ui/mixins/RouterPushes'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import CardView from '../../components/generic/CardView.vue'
import CardCarousel from '../../components/generic/CardCarousel.vue'
import ProviderMixin from '@/utils/ui/mixins/ProviderMixin'
import { ProviderScopes } from '@/utils/commonConstants'
import AddToQueue from '@/icons/AddToQueueIcon.vue'
import PlainPlay from '@/icons/PlainPlayIcon.vue'
import AddToLibrary from '@/icons/AddToLibraryIcon.vue'
import JukeboxMixin from '@/utils/ui/mixins/JukeboxMixin'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import SongDetailsCompact from '@/mainWindow/components/songView/components/SongDetailsCompact.vue'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import SmallSongItem from '@/mainWindow/components/generic/SmallSongItem.vue'
import { convertDuration } from '@/utils/common'

@Component({
  components: {
    CardView,
    CardCarousel,
    PlainPlay,
    AddToLibrary,
    AddToQueue,
    SongDetailsCompact,
    SmallSongItem
  }
})
export default class Albums extends mixins(
  RouterPushes,
  ContextMenuMixin,
  ProviderMixin,
  JukeboxMixin,
  PlayerControls,
  ImgLoader
) {
  get providers() {
    return this.fetchProviders()
  }

  private totalTime = 0

  get totalListenTime() {
    if (this.totalTime > 60) {
      return [Math.round(this.totalTime / 60), 'Minutes']
    } else {
      return [this.totalTime, 'Seconds']
    }
  }

  get maxPlayTimeFormatted() {
    const formatted = convertDuration(this.analyticsSongs[0].playTime)
    const split = formatted.split(':').reverse()

    const ret = []
    for (let i = 0; i < split.length; i++) {
      let suffix = ''
      switch (i) {
        case 0:
          suffix = 'Secs'
          break
        case 1:
          suffix = 'Mins'
          break
        case 2:
          suffix = 'Hours'
          break
      }

      ret.push([split[i], suffix])
    }

    return ret.reverse()
  }

  recommendationList: Record<string, Song[]> = {}
  loadingMap: Record<string, boolean> = {}

  get firstFourAnalytics() {
    return this.analyticsSongs.slice(1, 5)
  }

  get secondFourAnalytics() {
    return this.analyticsSongs.slice(5, 9)
  }

  private fetchProviders() {
    const providers = this.getProvidersByScope(ProviderScopes.RECOMMENDATIONS)
    return providers
  }

  private fetchRecomsFromProviders() {
    for (const val of this.providers) {
      this.getResults(val.key, val.getRecommendations())
    }
  }

  hasRecommendations(key: string) {
    return this.recommendationList[key] && this.recommendationList[key].length > 0
  }

  addToQueue(key: string) {
    this.queueSong(this.recommendationList[key] ?? [])
  }

  playAll(key: string) {
    this.playTop(this.recommendationList[key] ?? [])
  }

  addToLibrary(key: string) {
    this.addSongsToLibrary(...(this.recommendationList[key] ?? []))
  }

  analyticsSongs: (Song & { playTime: number; playCount: number })[] = []

  async created() {
    const playCounts = await window.SearchUtils.getPlayCount()
    const values = Object.entries(playCounts)
      .map((val) => ({ ...val[1], id: val[0] }))
      .sort((a, b) => b.playTime - a.playTime)

    this.totalTime = values.reduce((prev, curr) => prev + curr.playTime, 0)

    const renderFirst = 9

    const fetchedSongs: (Song & (typeof playCounts)[0])[] = []
    for (let i = 0; i < values.length; i++) {
      let song: Song | undefined = (
        await window.SearchUtils.searchSongsByOptions({
          song: {
            _id: values[i].id
          }
        })
      )[0]

      if (!song) {
        for (const p of this.getAllProviders()) {
          if (p.matchEntityId(values[i].id)) {
            song = await p.getSongById(values[i].id)
          }
        }
      }

      if (!song) continue

      fetchedSongs.push({
        ...song,
        playTime: values[i].playTime,
        playCount: values[i].playCount
      })

      if (fetchedSongs.length === renderFirst) break
    }

    this.analyticsSongs.push(...fetchedSongs)
  }

  mounted() {
    this.fetchRecomsFromProviders()
    this.onProvidersChanged(this.fetchRecomsFromProviders)
  }

  private async getResults(key: string, gen: AsyncGenerator<Song[]>) {
    this.loadingMap[key] = true
    for await (const s of gen) {
      if (!this.recommendationList[key]) {
        this.recommendationList[key] = []
      }
      this.recommendationList[key].push(...s)
      this.recommendationList = Object.assign({}, this.recommendationList)
    }
    this.loadingMap[key] = false
  }
}
</script>

<style lang="sass" scoped>
.title
  font-weight: bold
  font-size: 55px
  text-align: left

.provider-title
  font-weight: bold
  font-size: 26px
  text-align: left

.recommendations-container
  margin-bottom: 50px
  margin-top: 20px

.slider-row
  margin-top: 15px

.big-song
  min-width: 200px
  max-width: 400px
  margin-right: 25px

.played-for
  font-size: 30px
  margin-bottom: 15px
  font-weight: 700

.single-song-item
  margin-bottom: 30px

.playtime, .total-listen-time-item
  color: var(--accent)
  font-weight: 700
  margin-right: 3px

.playtime-suffix
  font-weight: 700

.big-playtime
  font-size: 30px

.small-song-first
  margin-right: 50px
  margin-left: 50px
  display: block

.small-song-second
  margin-right: 50px
  display: block

.analytics
  margin-bottom: 30px

.total-listen-time
  font-size: 28px
  margin-bottom: 50px
  margin-left: 15px

@media only screen and (max-width : 1557px)
  .small-song-second
    display: none

  .small-song-first
    margin-left: 30px
    flex: 0 0 40%
    max-width: 40%

@media only screen and (max-width : 1188px)
  .small-song-first
    display: none
</style>
