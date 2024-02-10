<!-- 
  AccountsSetup.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="p-0">
    <b-row>
      <b-col class="title-text mb-4"> Would you like to connect to any of these services? </b-col>
    </b-row>
    <b-row align-h="around">
      <b-col cols="auto">
        <YoutubeBig @click="handleClick(providers[0])" class="button-grow" :active="loggedInYoutube" />
      </b-col>
      <b-col cols="auto">
        <SpotifyBig class="button-grow" @click="handleClick(providers[1])" :active="loggedInSpotify" />
      </b-col>
      <b-col cols="auto">
        <LastFMBig class="button-grow" @click="handleClick(providers[2])" :active="loggedInLastFM" />
      </b-col>
    </b-row>
    <b-row class="mt-4">
      <b-col class="note">
        * Logging in to above services is optional. <br />
        * You may make use of the search feature to find and play songs from youtube without logging in
      </b-col>
    </b-row>
    <b-row no-gutters align-h="end" class="button-row mt-3">
      <b-col cols="auto" class="mr-3">
        <b-button @click="skip" class="close-button">Prev</b-button>
      </b-col>
      <b-col cols="auto">
        <b-button @click="next" class="create-button">Close</b-button>
      </b-col>
    </b-row>
    <ConfirmationModal
      keyword="signout from"
      :itemName="activeSignout?.provider.Title"
      id="signoutSetupModal"
      @confirm="signout"
    />
  </b-container>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import Logo from '@/icons/LogoIcon.vue'
import DirectoryGroup from '@/preferenceWindow/components/DirectoryGroup.vue'
import YoutubeBig from '@/icons/YoutubeBigIcon.vue'
import SpotifyBig from '@/icons/SpotifyBigIcon.vue'
import LastFMBig from '@/icons/LastFMBigIcon.vue'
import { mixins } from 'vue-facing-decorator'
import AccountsMixin from '@/utils/ui/mixins/AccountsMixin'
import ConfirmationModal from '../../../commonComponents/ConfirmationModal.vue'
import { vxm } from '@/mainWindow/store'

@Component({
  components: {
    Logo,
    DirectoryGroup,
    YoutubeBig,
    SpotifyBig,
    LastFMBig,
    ConfirmationModal
  },
  emits: ['next', 'prev']
})
export default class AccountsSetup extends mixins(AccountsMixin) {
  activeSignout: Provider | null = null

  async mounted() {
    this.signoutMethod = this.showSignoutModal
  }

  next() {
    this.$emit('next')
  }

  skip() {
    this.$emit('prev')
  }

  get loggedInYoutube() {
    return vxm.providers.youtubeProvider.loggedIn
  }

  get loggedInSpotify() {
    return vxm.providers.spotifyProvider.loggedIn
  }

  get loggedInLastFM() {
    return vxm.providers.lastfmProvider.loggedIn
  }

  async signout() {
    if (this.activeSignout) {
      if (this.activeSignout.provider) {
        await this.activeSignout.provider.signOut()

        this.activeSignout['username'] = (await this.activeSignout.provider.getUserDetails()) ?? ''
        this.activeSignout = null
      }
    }
  }

  protected showSignoutModal(signout: Provider) {
    this.activeSignout = signout
    this.$bvModal.show('signoutSetupModal')
  }
}
</script>

<style lang="sass">
.modal-dialog
  transition: all 0.4s cubic-bezier(0.71, -0.03, 0.32, 1.01), transform 0.3s ease-out !important
</style>

<style lang="sass" scoped>
.title-text
  margin-left: 15px
  font-size: 16px

.create-button, .close-button
  font-size: 16px
  font-weight: 400
  color: var(--textInverse)
  background-color: var(--accent)
  border-radius: 6px
  margin-bottom: 8px
  padding: 6px 20px 6px 20px
  margin-top: 15px
  border: 0

.close-button
  background-color: var(--textSecondary)
  color: var(--textPrimary)

.button-row
  margin-right: 15px

.note
  color: var(--accent)
</style>
