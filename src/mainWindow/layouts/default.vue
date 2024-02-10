<!-- 
  default.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="appContainer">
    <TopBar :showRefreshIcon="showRefreshIcon" class="topbar" :class="{ 'is-open': isSidebarOpen }" />
    <Sidebar class="sidebar" />
    <MusicBar class="musicbar" />
    <MiniPlayer class="mini-player" />

    <div class="main-content" :class="{ 'is-open': isSidebarOpen }">
      <router-view
        v-slot="{ Component }"
        :enableRefresh="enableRefreshIcon"
        :key="refreshPage.toString()"
        class="animate_absolute"
      >
        <transition
          appear
          name="custom-slide-fade"
          enter-active-class="animate__animated animate__slideInLeft animate__fast"
          leave-active-class="animate__animated animate__slideOutRight animate__fast"
        >
          <component :is="Component" />
        </transition>
      </router-view>

      <!-- <transition
        appear
        name="custom-slide-fade"
        enter-active-class="animate__animated animate__slideInLeft animate__fast"
        leave-active-class="animate__animated animate__slideOutRight animate__fast"
      >
        <router-view
          :enableRefresh="enableRefreshIcon"
          :key="refreshPage.toString()"
          class="animate_absolute"
        ></router-view>
      </transition> -->
    </div>
  </div>
</template>

<script lang="ts">
import MusicBar from '@/mainWindow/components/musicbar/Musicbar.vue'
import Sidebar from '@/mainWindow/components/sidebar/Sidebar.vue'
import TopBar from '@/mainWindow/components/topbar/TopBar.vue'
import { Component, Watch } from 'vue-facing-decorator'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import { mixins } from 'vue-facing-decorator'
import { vxm } from '../store/index'
import MiniPlayer from '@/mainWindow/components/miniplayer/MiniPlayer.vue'

@Component({
  components: {
    Sidebar,
    TopBar,
    MusicBar,
    MiniPlayer
  }
})
export default class DefaultLayout extends mixins(ContextMenuMixin) {
  refreshPage = false
  showRefreshIcon = false

  get isSidebarOpen() {
    return vxm.themes.sidebarOpen
  }

  enableRefreshIcon() {
    this.showRefreshIcon = true
  }

  @Watch('$route')
  onRouteChange() {
    window.WindowUtils.clearRSS()
  }

  private listenRefreshPage() {
    vxm.themes.$watch('_refreshPage', (refresh) => {
      if (refresh) {
        this.refreshPage = !this.refreshPage
        vxm.themes.refreshPage = false
      }
    })
  }

  mounted() {
    this.listenRefreshPage()
    this.$router.beforeEach((to, from, next) => {
      this.showRefreshIcon = false
      next()
    })
  }
}
</script>

<style lang="sass" scoped>
.musicbar
  position: fixed
  z-index: 4

.sidebar
  position: relative
  z-index: 3

.topbar
  position: fixed
  z-index: 2
  left: calc(70px + 30px + 7.5px)
  width: calc(100% - 70px - 30px - 7.5px)
  transition: 0.2s
  &.is-open
    width: calc(100% - 261px - 30px - 7.5px)
    left: calc(261px + 30px + 7.5px)

.main-content
  position: absolute
  left: calc(70px)
  top: calc(70px + 18px + 4px)
  right: 0
  bottom: calc(6rem + 30px)
  height: calc(100% - (6rem + 30px) - 70px - 11px)
  overflow-y: hidden
  overflow-x: hidden
  z-index: 1
  transition: 0.2s
  padding-right: 20px
  &.is-open
    left: calc(261px)

.mini-player
  display: none
  height: calc(100% - 6rem - 15px)

.main-content
  display: flex

.sidebar
  display: block

.topbar
  display: flex


@media only screen and (max-width : 800px)
  .main-content
    display: none

  .sidebar
    display: none

  .topbar
    display: none

  .mini-player
    display: block
</style>
