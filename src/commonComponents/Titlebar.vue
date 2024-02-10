<!-- 
  Titlebar.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container v-if="!hasFrame" fluid class="titlebar">
    <div class="titlebar-resize-handle top"></div>
    <div class="titlebar-resize-handle right"></div>
    <div class="titlebar-resize-handle left"></div>

    <b-row no-gutters align-h="between" v-if="showTitlebarIcons">
      <b-col cols="auto" class="h-100">
        <b-row v-if="windowType === 'main-window'" no-gutters align-v="center" class="logo-container">
          <b-col cols="auto">
            <Logo />
          </b-col>
          <b-col cols="auto" class="logo-title"> oosync </b-col>
        </b-row>
      </b-col>
      <b-col cols="auto" class="buttons-group" v-if="!isJukeboxModeActive">
        <b-row no-gutters>
          <b-col cols="auto">
            <div class="titlebar-buttons minimize-button" @click="onMinimize()">
              <svg width="13" height="2" viewBox="0 0 17 2" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path
                  d="M15.9375 0H1.0625C0.474805 0 0 0.446875 0 1C0 1.55313 0.474805 2 1.0625 2H15.9375C16.5252 2 17 1.55313 17 1C17 0.446875 16.5252 0 15.9375 0Z"
                  fill="var(--textPrimary)"
                />
              </svg>
            </div>
          </b-col>

          <b-col cols="auto">
            <div @click="onMaximize()" class="titlebar-buttons maximize-button">
              <svg
                v-if="isMaximized"
                width="13"
                height="10"
                viewBox="0 0 15 12"
                fill="none"
                xmlns="http://www.w3.org/2000/svg"
              >
                <path
                  d="M13.75 0H5C4.66848 0 4.35054 0.126428 4.11612 0.351472C3.8817 0.576516 3.75 0.88174 3.75 1.2V2.4H5V1.2H13.75V7.2H12.5V8.4H13.75C14.0815 8.4 14.3995 8.27357 14.6339 8.04853C14.8683 7.82348 15 7.51826 15 7.2V1.2C15 0.88174 14.8683 0.576516 14.6339 0.351472C14.3995 0.126428 14.0815 0 13.75 0V0Z"
                  fill="var(--textPrimary)"
                />
                <path
                  d="M10 3.6001H1.25C0.918479 3.6001 0.600537 3.72653 0.366116 3.95157C0.131696 4.17661 0 4.48184 0 4.8001V10.8001C0 11.1184 0.131696 11.4236 0.366116 11.6486C0.600537 11.8737 0.918479 12.0001 1.25 12.0001H10C10.3315 12.0001 10.6495 11.8737 10.8839 11.6486C11.1183 11.4236 11.25 11.1184 11.25 10.8001V4.8001C11.25 4.48184 11.1183 4.17661 10.8839 3.95157C10.6495 3.72653 10.3315 3.6001 10 3.6001ZM1.25 10.8001V4.8001H10V10.8001H1.25Z"
                  fill="var(--textPrimary)"
                />
              </svg>
              <svg v-else width="10" height="10" viewBox="0 0 10 10" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path
                  d="M0 1.60714C0 1.1809 0.169323 0.772119 0.470721 0.470721C0.772119 0.169323 1.1809 0 1.60714 0H8.39286C8.8191 0 9.22788 0.169323 9.52928 0.470721C9.83068 0.772119 10 1.1809 10 1.60714V8.39285C10 8.8191 9.83068 9.22788 9.52928 9.52928C9.22788 9.83067 8.8191 10 8.39286 10H1.60714C1.1809 10 0.772119 9.83067 0.470721 9.52928C0.169323 9.22788 0 8.8191 0 8.39285V1.60714ZM1.60714 1.07143C1.46506 1.07143 1.3288 1.12787 1.22834 1.22834C1.12787 1.3288 1.07143 1.46506 1.07143 1.60714V8.39285C1.07143 8.68857 1.31143 8.92857 1.60714 8.92857H8.39286C8.53494 8.92857 8.6712 8.87213 8.77166 8.77166C8.87213 8.6712 8.92857 8.53493 8.92857 8.39285V1.60714C8.92857 1.46506 8.87213 1.3288 8.77166 1.22834C8.6712 1.12787 8.53494 1.07143 8.39286 1.07143H1.60714Z"
                  fill="var(--textPrimary)"
                />
              </svg>
            </div>
          </b-col>

          <b-col cols="auto">
            <div class="titlebar-buttons close-button" @click="onClose()">
              <svg width="11" height="11" viewBox="0 0 11 11" fill="none" xmlns="http://www.w3.org/2000/svg">
                <path
                  d="M7.0264 5.85281L10.6063 2.28075C10.763 2.12396 10.8511 1.91131 10.8511 1.68957C10.8511 1.46783 10.763 1.25518 10.6063 1.09839C10.4495 0.9416 10.2369 0.853516 10.0152 0.853516C9.79348 0.853516 9.58086 0.9416 9.42409 1.09839L5.85254 4.67877L2.28099 1.09839C2.12422 0.9416 1.9116 0.853516 1.6899 0.853516C1.46819 0.853516 1.25557 0.9416 1.0988 1.09839C0.942031 1.25518 0.85396 1.46783 0.85396 1.68957C0.85396 1.91131 0.942031 2.12396 1.0988 2.28075L4.67867 5.85281L1.0988 9.42486C1.02077 9.50227 0.958833 9.59436 0.916566 9.69582C0.8743 9.79729 0.852539 9.90612 0.852539 10.016C0.852539 10.126 0.8743 10.2348 0.916566 10.3363C0.958833 10.4377 1.02077 10.5298 1.0988 10.6072C1.17619 10.6853 1.26827 10.7472 1.36972 10.7895C1.47118 10.8318 1.57999 10.8535 1.6899 10.8535C1.7998 10.8535 1.90862 10.8318 2.01007 10.7895C2.11152 10.7472 2.2036 10.6853 2.28099 10.6072L5.85254 7.02684L9.42409 10.6072C9.50148 10.6853 9.59356 10.7472 9.69501 10.7895C9.79646 10.8318 9.90528 10.8535 10.0152 10.8535C10.1251 10.8535 10.2339 10.8318 10.3354 10.7895C10.4368 10.7472 10.5289 10.6853 10.6063 10.6072C10.6843 10.5298 10.7462 10.4377 10.7885 10.3363C10.8308 10.2348 10.8525 10.126 10.8525 10.016C10.8525 9.90612 10.8308 9.79729 10.7885 9.69582C10.7462 9.59436 10.6843 9.50227 10.6063 9.42486L7.0264 5.85281Z"
                  fill="var(--textPrimary)"
                />
              </svg>
            </div>
          </b-col>
        </b-row>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'
import Logo from '@/icons/LogoIcon.vue'
@Component({
  components: {
    Logo
  }
})
export default class Sidebar extends Vue {
  @Prop({ default: 'main-window' })
  windowType!: 'main-window' | 'preference-window'

  @Prop({ default: false })
  isJukeboxModeActive!: boolean

  isMaximized = false
  resizedFinished: NodeJS.Timeout | undefined

  hasFrame = false
  showTitlebarIcons = false

  onMinimize() {
    window.WindowUtils.minWindow(this.windowType == 'main-window')
  }

  async onMaximize() {
    this.isMaximized = await window.WindowUtils.maxWindow(this.windowType == 'main-window')
  }

  onClose() {
    window.WindowUtils.closeWindow(this.windowType == 'main-window')
  }

  async created() {
    this.hasFrame = await window.WindowUtils.hasFrame()
    this.showTitlebarIcons = await window.WindowUtils.showTitlebarIcons()
  }

  mounted() {
    window.addEventListener('resize', () => {
      if (this.resizedFinished) clearTimeout(this.resizedFinished)
      this.resizedFinished = setTimeout(async () => {
        this.isMaximized = await window.WindowUtils.isWindowMaximized(this.windowType == 'main-window')
      }, 250)
    })
  }
}
</script>

<style lang="sass" scoped>
.titlebar
  width: 100%
  height: 31px
  -webkit-app-region: drag
  padding-left: 0 !important
  padding-right: 0 !important
  .titlebar-resize-handle
    position: absolute
    top: 0
    left: 0
    -webkit-app-region: no-drag
    &.top
      width: 100%
      height: 3px

    &.right
      left: auto
      right: 0
      width: 3px
      height: 18px

    &.left
      width: 3px
      height: 18px

.titlebar-buttons
  padding-left: 17px
  padding-right: 17px
  -webkit-app-region: no-drag

.close-button
  &:hover
    background: rgba(255, 0, 0, 0.73)

.maximize-button, .minimize-button
  &:hover
    background: rgba(255, 255, 255, 0.1)

.logo-container
  margin-top: 4px
  margin-bottom: 4px
  margin-left: 10px

.logo-title
  margin-left: 4px
  font-family: Poppins
  font-style: normal
  font-weight: 600
  font-size: 13px
  line-height: 167.19%
  letter-spacing: 0.105em
  color: var(--textSecondary)

.buttons-group
  z-index: 999
</style>
