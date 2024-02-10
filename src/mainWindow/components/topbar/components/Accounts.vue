<!-- 
  Accounts.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div>
    <Person id="account" class="accounts-icon" @click="togglePopover" />
    <Transition>
      <div v-if="showAccountsPopover" triggers="click blur" class="accounts-popover custom-popover"
        v-click-outside="hidePopover">
        <div class="buttons" :key="`${forceRefresh}`" :id="`${forceRefresh}`">
          <div v-for="p in providers" :key="p.key">
            <IconButton v-if="p && p.provider.canLogin" :bgColor="p.provider.BgColor"
              :hoverText="p.provider.loggedIn ? $t('accounts.sign_out') : p.provider.Title"
              :title="p.username ? p.username : $t('accounts.connect')" @click="handleClick(p)">
              <template #icon>
                <component v-if="isIconComponent(p.provider.IconComponent)" :is="p.provider.IconComponent" />
                <inline-svg class="provider-icon" v-else-if="p.provider.IconComponent.endsWith('svg')"
                  :src="`media://${p.provider.IconComponent}`" />
                <img v-else referrerPolicy="no-referrer" :src="p.provider.IconComponent" alt="provider icon"
                  class="provider-icon" />
              </template>
            </IconButton>
          </div>
        </div>
      </div>
    </Transition>
    <ConfirmationModal keyword="log out from " :itemName="activeSignout ? activeSignout.provider.Title : ''"
      id="signoutModal" @confirm="signout" />
  </div>
</template>
<script lang="ts">
import IconButton from '@/mainWindow/components/generic/IconButton.vue'
import YoutubeIcon from '@/icons/YoutubeIcon.vue'
import SpotifyIcon from '@/icons/SpotifyIcon.vue'
import LastFMIcon from '@/icons/LastFMIcon.vue'
import Person from '@/icons/PersonIcon.vue'
import { Component } from 'vue-facing-decorator'
import ConfirmationModal from '@/commonComponents/ConfirmationModal.vue'
import { mixins } from 'vue-facing-decorator'
import AccountsMixin from '@/utils/ui/mixins/AccountsMixin'
import InvidiousIcon from '@/icons/InvidiousIcon.vue'
import PipedIcon from '@/icons/PipedIcon.vue'
import { vxm } from '@/mainWindow/store'

@Component({
  components: {
    IconButton,
    YoutubeIcon,
    InvidiousIcon,
    SpotifyIcon,
    LastFMIcon,
    PipedIcon,
    Person,
    ConfirmationModal
  }
})
export default class TopBar extends mixins(AccountsMixin) {
  activeSignout: Provider | null = null

  forceRefresh = 0

  showAccountsPopover = false

  isIconComponent(src: string) {
    switch (src) {
      case vxm.providers.youtubeProvider.IconComponent:
      case vxm.providers.spotifyProvider.IconComponent:
      case vxm.providers.lastfmProvider.IconComponent:
        return true
      default:
        return false
    }
  }

  async mounted() {
    this.signoutMethod = this.showSignoutModal
  }

  async signout() {
    if (this.activeSignout) {
      if (this.activeSignout) {
        this.activeSignout.provider.signOut()

        this.activeSignout['username'] = (await this.activeSignout.provider.getUserDetails()) ?? ''
        this.activeSignout = null
      }
    }
  }

  protected showSignoutModal(signout: Provider) {
    this.activeSignout = signout
    this.$bvModal.show('signoutModal')
  }

  togglePopover() {
    this.showAccountsPopover = !this.showAccountsPopover
  }

  hidePopover() {
    this.showAccountsPopover = false
  }
}
</script>

<style lang="sass" scoped>
.accounts-icon
  height: 22px
  width: 22px
  margin-left: 0.5rem

.buttons
  > div
    margin-bottom: 8px
    &:first-child
      margin-top: 15px

.custom-popover
  position: fixed
  top: 60px
  right: 45px
  padding: 5px 15px 10px 15px

.v-enter-active, .v-leave-active
  transition: opacity 0.3s ease


.v-enter-from, .v-leave-to
  opacity: 0
</style>
