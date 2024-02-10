<!-- 
  Preferences.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div id="app">
    <Titlebar windowType="preference-window" />
    <Sidebar />
    <div class="logo-title version">v{{ version }}</div>

    <div class="main-content">
      <router-view v-slot="{ Component }" class="animate_absolute">
        <transition
          appear
          name="custom-slide-fade"
          enter-active-class="animate__animated animate__slideInLeft animate__fast"
          leave-active-class="animate__animated animate__slideOutRight animate__fast"
        >
          <component :is="Component" />
        </transition>
      </router-view>
    </div>
  </div>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import Titlebar from '@/commonComponents/Titlebar.vue'
import { mixins } from 'vue-facing-decorator'
import ThemeHandler from '@/utils/ui/mixins/ThemeHandler'
import Sidebar from '@/preferenceWindow/components/Sidebar.vue'
import { bus } from './main'
import { useI18n } from 'vue-i18n'

@Component({
  components: {
    Titlebar,
    Sidebar
  },
  setup: () => {
    const { t, locale } = useI18n()
    return { t, locale }
  }
})
export default class App extends mixins(ThemeHandler) {
  mounted() {
    this.getLanguage()
    bus.on('themeChanged', this.fetchThemeFromID)
    this.registerDevTools()
    this.listenArgs()
  }

  private async getLanguage() {
    const langs = await window.PreferenceUtils.loadSelective<Checkbox[]>('system_language')
    const active = (langs ?? []).find((val) => val.enabled)
    if (active) {
      this.$i18n.locale = active?.key
    }
  }

  private listenArgs() {
    window.WindowUtils.listenArgs((args) => {
      if (args && (args as { page: string }).page) {
        this.$router.push((args as { page: string }).page)
      }
    })
  }

  get version() {
    return process.env.MOOSYNC_VERSION
  }

  private registerDevTools() {
    document.addEventListener('keydown', function (e) {
      if (e.code === 'F12') {
        window.WindowUtils.toggleDevTools(false)
      } else if (e.code === 'F5') {
        location.reload()
      }
    })
  }

  private closeWindow() {
    window.WindowUtils.closeWindow(false)
  }
}
</script>

<style>
#app {
  font-family: 'Nunito Sans';
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
  text-align: center;
  background: var(--primary);
  color: var(--textPrimary);
  width: 100%;
  height: 100%;
  /* margin-top: 60px */
}

body {
  background-color: var(--primary) !important;
  color: var(--textPrimary) !important;
}

.footer-buttons {
  position: absolute;
  bottom: 0;
  right: 0;
}
</style>

<style lang="sass">
.slide-fade-enter-active
  transition: all .3s ease

.slide-fade-leave-active
  transition: all .2s ease
.slide-fade-enter, .slide-fade-leave-to
  transform: translateY(100px)
  opacity: 0

*::-webkit-scrollbar,
*::-webkit-scrollbar-thumb
  width: 26px
  border-radius: 13px
  background-clip: padding-box
  border: 10px solid transparent

*::-webkit-scrollbar-thumb
  box-shadow: inset 0 0 0 10px
  min-height: 40px

*::-webkit-scrollbar-track
  background: transparent
</style>

<style lang="sass" scoped>
.main-content
  position: absolute
  left: calc(261px + 30px)
  height: calc(100% - 30px - 70px)
  top: calc(70px)
  right: 0
  bottom: calc(30px)
  overflow-y: scroll
  overflow-x: hidden
  z-index: -4
  transition: 0.2s

.logo-title
  position: absolute
  bottom: 0
  margin-left: 4px
  font-family: Poppins
  font-style: normal
  font-weight: 600
  font-size: 14px
  line-height: 167.19%
  letter-spacing: 0.105em
  text-align: left
  color: var(--textSecondary)
</style>
