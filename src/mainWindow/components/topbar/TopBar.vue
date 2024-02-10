<!-- 
  TopBar.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="topbar-container align-items-center">
    <b-container fluid class="d-flex">
      <b-row align-h="start" class="flex-grow-1">
        <b-col cols="auto" class="my-auto"> <Navigation /> </b-col>
        <b-col> <Search /> </b-col>
        <b-col cols="auto" class="pr-5 ml-auto my-auto icons-bar d-flex">
          <b-row class="flex-grow-1">
            <b-col cols="auto" v-if="showRefreshIcon">
              <Refresh @click="refreshPage" class="refresh-icon button-grow" />
            </b-col>
            <!-- <b-col cols="auto"> <Notifications /> </b-col> -->
            <b-col v-if="!isJukeboxModeActive" cols="auto">
              <Accounts />
            </b-col>
            <b-col v-if="!isJukeboxModeActive" cols="auto">
              <Gear id="settings" class="gear-icon" @click="openSettings" />
            </b-col>
            <b-col cols="auto">
              <JukeboxIcon
                v-if="showJukeboxIcon"
                :isActive="isJukeboxModeActive"
                class="jukebox-icon button-grow"
                @click="toggleJukeboxMode"
              />
            </b-col>
            <b-col cols="auto" v-if="showUpdateIcon"> <Update class="update-icon button-grow" /></b-col>
          </b-row>
        </b-col>
      </b-row>
    </b-container>
  </div>
</template>

<script lang="ts">
import Navigation from '@/mainWindow/components/topbar/components/Navigation.vue'
import Search from '@/mainWindow/components/topbar/components/Search.vue'
import { Component, Prop } from 'vue-facing-decorator'
import Accounts from '@/mainWindow/components/topbar/components/Accounts.vue'
import Notifications from '@/mainWindow/components/topbar/components/Notifications.vue'
import Refresh from '@/icons/RefreshIcon.vue'
import Update from '@/mainWindow/components/topbar/components/Update.vue'

import Gear from '@/icons/GearIcon.vue'
import { EventBus } from '@/utils/preload/ipc/constants'
import { bus } from '../../main'
import JukeboxIcon from '@/icons/JukeboxIcon.vue'
import { vxm } from '@/mainWindow/store'
import JukeboxMixin from '@/utils/ui/mixins/JukeboxMixin'
import { mixins } from 'vue-facing-decorator'

@Component({
  components: {
    Search,
    Navigation,
    Accounts,
    Notifications,
    Gear,
    Refresh,
    Update,
    JukeboxIcon
  }
})
export default class TopBar extends mixins(JukeboxMixin) {
  @Prop({ default: false })
  showRefreshIcon!: boolean

  showJukeboxIcon = false

  get showUpdateIcon() {
    return vxm.themes.isUpdateAvailable
  }

  openSettings() {
    window.WindowUtils.openWindow(false)
  }

  refreshPage() {
    bus.emit(EventBus.REFRESH_PAGE)
  }

  private async handleJukeboxIcon() {
    const setJukeboxIconVisibility = (systemPrefs: Checkbox[]) => {
      this.showJukeboxIcon = systemPrefs.find((val) => val.key === 'jukebox_mode_toggle')?.enabled ?? false

      // Disable jukebox mode if icon isn't supposed to be shown
      if (!this.showJukeboxIcon) {
        vxm.themes.jukeboxMode = false
      }
    }

    setJukeboxIconVisibility(
      (await window.PreferenceUtils.loadSelective<Checkbox[]>('system', false, [])) as Checkbox[]
    )
    window.PreferenceUtils.listenPreferenceChanged('system', true, (_, value: Checkbox[]) =>
      setJukeboxIconVisibility(value)
    )

    const optionalJukeboxItems = await window.PreferenceUtils.loadSelective<Checkbox[]>('jukebox_optional_fields')
    vxm.themes.jukeboxOptionalFields = optionalJukeboxItems ?? []

    window.PreferenceUtils.listenPreferenceChanged<Checkbox[]>('jukebox_optional_fields', true, (_, value) => {
      vxm.themes.jukeboxOptionalFields = value
    })
  }

  created() {
    this.handleJukeboxIcon()
  }

  async toggleJukeboxMode() {
    if (vxm.themes.jukeboxMode) {
      const pin = await window.Store.getSecure('jukebox_pin')
      if (pin) {
        bus.emit(EventBus.SHOW_PIN_ENTRY_MODAL, pin.length, (input: string) => {
          if (pin === input) {
            vxm.themes.jukeboxMode = false
            return true
          }

          return false
        })
        return
      }
      vxm.themes.jukeboxMode = false
      return
    }

    vxm.themes.jukeboxMode = true
  }
}
</script>

<style lang="sass" scoped>
.topbar-container
  background: var(--primary)
  height: 70px

.gear-icon
  height: 26px
  width: 26px
  margin-left: 10px

.update-icon
  height: 26px
  width: 26px
  margin-left: 10px

.icons-bar
  margin-right: 30px

.refresh-icon
  height: 22px
  width: 22px

.jukebox-icon
  height: 22px
  width: 22px
</style>
