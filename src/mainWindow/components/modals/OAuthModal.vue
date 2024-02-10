<!-- 
  OAuthModal.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal no-close-on-backdrop centered size="sm" :id="id" :ref="id" hide-footer hide-header>
    <b-container class="response-container">
      <b-row no-gutters class="d-flex">
        <b-col class="title" cols="auto">Logging in to</b-col>
        <b-col class="title ml-1" cols="auto" :style="{ color: textColor }">{{ title }}</b-col>
      </b-row>
      <div v-if="!alternative">
        <b-row>
          <b-col class="mt-4 waiting">{{ desc ? desc : 'Waiting for response from your browser...' }}</b-col>
        </b-row>
        <b-row>
          <b-col class="d-flex justify-content-center">
            <div
              @click="openBrowser"
              class="start-button button-grow mt-4 d-flex justify-content-center align-items-center"
              v-if="url"
            >
              Open browser
            </div>
          </b-col>
        </b-row>
        <b-row>
          <b-col class="d-flex justify-content-center">
            <div
              @click="submitCode"
              class="start-button button-grow mt-4 d-flex justify-content-center align-items-center"
              v-if="manualClick"
            >
              Submit
            </div>
          </b-col>
        </b-row>
        <b-row v-if="url">
          <b-col class="not-working-text mt-3" @click="alternative = true"> Having trouble? </b-col>
        </b-row>
      </div>
      <div v-if="alternative && url">
        <b-row>
          <b-col class="mt-4 waiting">Paste this link in your browser...</b-col>
        </b-row>
        <b-row>
          <b-col>
            <b-input class="ext-input mt-3" @click="copyToClipboard" v-model="url" readonly />
          </b-col>
        </b-row>
        <b-row>
          <b-col class="mt-4 waiting">Then enter the code shown after the login process has completed</b-col>
        </b-row>
        <b-row>
          <b-col>
            <b-input class="ext-input mt-3 code-input" v-model="oauthCode" placeholder="Code" />
          </b-col>
        </b-row>
        <b-row>
          <b-col class="d-flex justify-content-center">
            <div
              @click="submitCode"
              class="start-button button-grow mt-4 d-flex justify-content-center align-items-center"
            >
              Submit
            </div>
          </b-col>
        </b-row>
      </div>
    </b-container>
    <CrossIcon @click="close" class="close-icon button-grow" />
  </b-modal>
</template>

<script lang="ts">
import { EventBus } from '@/utils/preload/ipc/constants'
import { Component, Prop, Vue } from 'vue-facing-decorator'
import { bus } from '@/mainWindow/main'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import CrossIcon from '@/icons/CrossIcon.vue'
import InputGroup from '../generic/InputGroup.vue'

@Component({
  components: {
    SongDefault,
    CrossIcon,
    InputGroup
  }
})
export default class OAuthModal extends Vue {
  @Prop({ default: 'OAuthModal' })
  id!: string

  textColor = ''
  title = ''
  url = ''
  oauthCode = ''
  desc = ''
  manualClick = false
  oauthPath?: string

  private showing = false

  alternative = true

  openBrowser() {
    if (!this.url.startsWith('http')) {
      this.url = 'https://' + this.url
    }
    window.WindowUtils.openExternal(this.url)
  }

  close() {
    bus.emit(EventBus.HIDE_OAUTH_MODAL)
  }

  copyToClipboard() {
    navigator.clipboard.writeText(this.url)
  }

  submitEmpty() {
    window.WindowUtils.triggerOAuthCallback(this.oauthCode)
  }

  submitCode() {
    if (this.oauthPath) {
      if (!this.oauthCode.startsWith('moosync://')) {
        this.oauthCode = 'moosync://' + this.oauthPath + this.oauthCode
      }
      window.WindowUtils.triggerOAuthCallback(this.oauthCode)
    }
  }

  mounted() {
    bus.on(EventBus.SHOW_OAUTH_MODAL, (data: ExtendedLoginModalData) => {
      if (!this.showing) {
        this.alternative = false
        this.title = data.providerName
        this.textColor = data.providerColor
        this.desc = data.text ?? ''
        this.url = data.url ?? ''
        this.manualClick = data.manualClick ?? false
        this.oauthPath = data.oauthPath
        this.$bvModal.show(this.id)
        this.showing = true
      }
    })

    bus.on(EventBus.HIDE_OAUTH_MODAL, () => {
      if (this.showing) {
        this.$bvModal.hide(this.id)
        this.title = ''
        this.textColor = ''
        this.url = ''
        this.desc = ''
        this.manualClick = false
        this.oauthPath = undefined
        this.oauthCode = ''
        this.showing = false
      }
    })
  }
}
</script>

<style lang="sass" scoped>
.title
  font-size: 20px
  font-weight: 700

.waiting
  font-size: 16px
  font-weight: 700

.response-container
  padding: 10px

.close-icon
  position: absolute
  top: 20px
  right: 20px
  width: 14px
  height: 14px

.start-button
  width: 135px
  height: 36px
  border: 1px solid #65CB88
  border-radius: 6px
  font-size: 16px

.ext-input
  font-size: 16px
  max-width: 100%
  color: var(--textSecondary)
  background-color: var(--tertiary)
  border: 0
  border-bottom: 1px solid transparent
  border-radius: 0
  padding: 20px 15px 20px  15px
  &::placeholder
    color: var(--textSecondary)

.code-input
  color: var(--textPrimary)

.not-working-text
  color: var(--textSecondary)
  font-size: 14px
  transition: all 0.1s ease-in
  cursor: pointer
  &:hover
    color: var(--accent)
</style>
