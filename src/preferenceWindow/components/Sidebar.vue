<!-- 
  Sidebar.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-sidebar
    no-close-on-esc
    width="261px"
    visible
    id="sidebar"
    no-header-close
    no-close-on-route-change
    sidebar-class="gradient sidebar-height"
  >
    <template #header>
      <div class="d-flex w-100 mt-3 justify-content-between"></div>
    </template>
    <template #default>
      <div class="extra-margin-top">
        <Tabs />
      </div>
    </template>
  </b-sidebar>
</template>

<script lang="ts">
import Tabs from '@/preferenceWindow/components/sidebar/Tabs.vue'
import { Component, Vue } from 'vue-facing-decorator'

@Component({
  components: {
    Tabs
  }
})
export default class Sidebar extends Vue {}
</script>

<style lang="sass">
.gradient
  background: linear-gradient(176.27deg, var(--secondary) 1.69%, var(--primary) 72.01%)
  border-radius: 0px 30px 0px 0px
  top: 32px !important

.b-sidebar
  transition: 0.2s

.sidebar-height
  height: calc(100% - (6.5rem + 32px))

.b-sidebar-body
  &::-webkit-scrollbar-track
    margin-top: 0
    background: transparent !important
</style>

<style lang="sass" scoped>
.toggle
  margin-left: 8px

.sidebar
  position: absolute
  overflow: hidden

.icon-spacing
  margin-right: 26px
  padding-left: 6px

.extra-margin-top
  margin-top: 1rem
</style>
