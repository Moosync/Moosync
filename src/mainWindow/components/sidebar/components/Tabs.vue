<!-- 
  Tabs.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="d-flex flex-column">
    <router-link
      v-for="item in navigationTabs"
      v-bind:key="item.link"
      :to="{ path: item.link }"
      custom
      v-slot="{ navigate, isActive }"
    >
      <div
        class="d-flex button-bar"
        v-on:click="(...args: unknown[]) => getOnClick(item, navigate, ...args)"
        v-bind:class="{ 'button-active': item.custom ? false : isActive }"
      >
        <div
          class="whitebar"
          v-bind:class="{
            'whitebar-active': item.custom ? false : isActive
          }"
          v-if="item.custom ? false : isActive"
        ></div>
        <div
          class="d-flex align-items-center icon-transition icon-padding-open w-100"
          v-bind:class="{
            'icon-active': item.custom ? false : isActive
          }"
        >
          <div class="icon">
            <component :active="item.custom ? false : isActive" v-bind:is="item.component"></component>
          </div>
          <transition name="text-delay">
            <div
              class="text-padding text-format"
              v-if="isOpen"
              v-bind:class="{
                'text-active': item.custom ? false : isActive
              }"
            >
              {{ item.title }}
            </div>
          </transition>
        </div>
      </div>
    </router-link>
  </div>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'
import Playlists from '@/icons/PlaylistsIcon.vue'
import AllSongs from '@/icons/AllSongsIcon.vue'
import Artists from '@/icons/ArtistsIcon.vue'
import Genre from '@/icons/GenreIcon.vue'
import Albums from '@/icons/AlbumsIcon.vue'
import Toggle from '@/icons/ToggleIcon.vue'
import Rooms from '@/icons/RoomsIcon.vue'
import Explore from '@/icons/ExploreIcon.vue'
import Queue from '@/icons/QueueIcon.vue'
import { bus } from '@/mainWindow/main'

@Component({
  components: {
    Playlists,
    AllSongs,
    Artists,
    Genre,
    Albums,
    Toggle,
    Rooms,
    Explore,
    Queue
  }
})
export default class Sidebar extends Vue {
  private get componentNames() {
    return [
      { component: 'Queue', title: this.$t('sidebar.tabs.queue'), link: './', custom: this.openQueue.bind(this) },
      { component: 'AllSongs', title: this.$t('sidebar.tabs.allSongs'), link: '/songs' },
      { component: 'Playlists', title: this.$t('sidebar.tabs.playlists'), link: '/playlists' },
      { component: 'Albums', title: this.$t('sidebar.tabs.albums'), link: '/albums' },
      { component: 'Artists', title: this.$t('sidebar.tabs.artists'), link: '/artists' },
      { component: 'Genre', title: this.$t('sidebar.tabs.genre'), link: '/genre' },
      { component: 'Explore', title: this.$t('sidebar.tabs.explore'), link: '/recommendations' }
    ]
  }

  private get showExplore() {
    return true
  }

  get navigationTabs() {
    return this.componentNames.filter((val) => typeof val.link === 'string')
  }

  get methodTabs() {
    return this.componentNames.filter((val) => typeof val.link === 'function')
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  getOnClick(item: (typeof this.componentNames)[0], navigate: (...args: any[]) => void, ...args: any[]) {
    if (item.custom) {
      item.custom()
      return
    }
    navigate(...args)
  }

  private openQueue() {
    bus.emit('onToggleSlider', true)
  }

  @Prop({ default: true })
  isOpen!: boolean
}
</script>

<style lang="sass" scoped>
.icon
  width: 38px
  height: 38px
  display: flex
  align-items: center

.icon-padding-open
  padding: 0.25rem 0rem 0.25rem 1.8rem

.icon-padding-closed
  padding: 0.5rem 0rem 0.25rem 1rem

.icon-transition
  transition: 0.1s

.icon-padding-open.icon-transition:hover
  margin-left: 0.6rem

.text-padding
  padding-left: 2rem
  user-select: none

.text-format
  color: var(--textPrimary)

.text-active
  color: var(--accent)
  font-weight: 700

.button-bar
  margin-top: 1.25rem
  vertical-align: middle

.whitebar
  width: 3px
  height: auto
  background: var(--textPrimary)

.whitebar-active
  background: var(--accent)

.button-active
  background: linear-gradient(270deg, rgba(var(--secondary-rgb), 0) 0%, rgba(var(--accent-rgb), 0.22) 100%)

.icon-active
  padding-left: calc(1.8rem - 3px)

.text-delay-enter-active
  display: none
  transition-delay: 0.08s
</style>
