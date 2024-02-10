<!-- 
  Sidebar.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="sidebar-container">
    <b-sidebar
      no-close-on-esc
      :width="isOpen ? '261px' : '70px'"
      visible
      id="sidebar"
      no-header-close
      no-close-on-route-change
      :sidebar-class="`gradient ${hasFrame ? 'sidebar-top-low-spacing' : 'sidebar-top-high-spacing'}`"
    >
      <template #header>
        <div class="d-flex w-100 mt-3 justify-content-between">
          <Toggle class="toggle" @click="toggleOpen()" />
          <!-- <Rooms class="rooms-button" id="rooms" v-if="showRoomsButton" />
          <b-popover
            v-if="showRoomsButton"
            :target="`rooms`"
            placement="rightbottom"
            title="Rooms"
            triggers="focus"
            :content="`Placement hello`"
          >
            <div>
              <b-tabs content-class="mt-3">
                <b-tab title="Join" active>
                  <b-form-input
                    v-model="roomInput"
                    :formatter="formatter"
                    class="inputtext"
                    placeholder="Enter room ID"
                    aria-label="room id"
                  />
                  <button v-on:click="joinRoom">Join room</button>
                  <h3>{{ roomID }}</h3>
                </b-tab>
                <b-tab title="Create">
                  <button v-on:click="createRoom">Create Room</button>
                  <h3>{{ roomID }}</h3>
                </b-tab>
              </b-tabs>
              <div></div>
            </div>
          </b-popover> -->
        </div>
      </template>
      <template #default>
        <div class="extra-margin-top">
          <Tabs :isOpen="isOpen" />
        </div>
      </template>
    </b-sidebar>
  </div>
</template>

<script lang="ts">
import Rooms from '@/icons/RoomsIcon.vue'
import Toggle from '@/icons/ToggleIcon.vue'
import Tabs from '@/mainWindow/components/sidebar/components/Tabs.vue'
import { PeerMode } from '@/mainWindow/store/syncState'
import { Component, Vue } from 'vue-facing-decorator'
import { vxm } from '@/mainWindow/store'
import { bus } from '@/mainWindow/main'

@Component({
  components: {
    Toggle,
    Rooms,
    Tabs
  },
  emits: ['toggleOpen']
})
export default class Sidebar extends Vue {
  private roomInput = ''
  private showRoomsButton = true
  hasFrame = false

  get roomID() {
    return vxm.sync.roomID
  }

  get isOpen() {
    return vxm.themes.sidebarOpen
  }

  set isOpen(val: boolean) {
    vxm.themes.sidebarOpen = val
  }

  async created() {
    this.hasFrame = await window.WindowUtils.hasFrame()
  }

  toggleOpen() {
    this.isOpen = !this.isOpen

    // Delay showing of rooms button since it makes the toggle button smaller while sidebar size is transitioning
    if (!this.showRoomsButton) setTimeout(() => (this.showRoomsButton = true), 100)
    else this.showRoomsButton = false

    this.$emit('toggleOpen', this.isOpen)
  }

  public formatter(value: string) {
    return value.toUpperCase()
  }

  private setWatcher() {
    vxm.sync.setMode(PeerMode.WATCHER)
  }

  private setBroadcaster() {
    vxm.sync.setMode(PeerMode.BROADCASTER)
  }

  private joinRoom() {
    bus.emit('join-room', this.roomInput)
  }

  private createRoom() {
    bus.emit('create-room')
  }
}
</script>

<style lang="sass" scoped>
.toggle
  margin-left: 8px

.icon-spacing
  margin-right: 26px
  padding-left: 6px

.extra-margin-top
  margin-top: 1rem

.rooms-button
  width: 30px
  height: 30px
</style>
