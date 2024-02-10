<!-- 
  Themes.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="w-100 h-100">
    <ContextMenu ref="contextMenu" v-click-outside="hideContextMenu" :menu-items="menu" />
    <b-container fluid>
      <b-row>
        <PreferenceHeader :title="$t('settings.themes.songView')" :tooltip="$t('settings.themes.songView_tooltip')"
          class="mb-3" />
      </b-row>
      <b-row no-gutters class="w-100"> </b-row>
      <b-row no-gutters class="w-100">
        <b-col cols="5" xl="3" class="p-2">
          <div class="theme-component-container">
            <ThemeComponentClassic @click="setSongView('classic')" :selected="isSongView('classic')" :id="getRandomID()"
              :colors="currentTheme" />
            {{ $t('settings.themes.songView_classic') }}
          </div>
        </b-col>
        <b-col cols="5" xl="3" class="p-2">
          <div class="theme-component-container">
            <ThemeComponentCompact @click="setSongView('compact')" :selected="isSongView('compact')" :id="getRandomID()"
              :colors="currentTheme" />
            {{ $t('settings.themes.songView_compact') }}
          </div>
        </b-col>
      </b-row>
      <b-row>
        <PreferenceHeader :title="$t('settings.themes.themes')" :tooltip="$t('settings.themes.themes_tooltip')"
          class="mt-5 mb-3" />
      </b-row>
      <b-row no-gutters class="w-100"> </b-row>
      <b-row no-gutters class="w-100">
        <b-col cols="5" xl="3" class="p-2">
          <div class="theme-component-container">
            <component :is="themesComponent" @click="setTheme('default')" @contextmenu="themeMenu($event, defaultTheme)"
              :selected="isThemeActive('default')" :id="getRandomID()" :colors="defaultTheme.theme" />
            <div class="title">{{ $t('settings.themes.themes_default') }}</div>
            <div class="author">Moosync</div>
          </div>
        </b-col>
        <b-col cols="5" xl="3" class="p-2" v-for="(value, key) in allThemes" :key="key">
          <div class="theme-component-container">
            <component :is="themesComponent" @click="setTheme(value.id)" :selected="isThemeActive(value.id)"
              :id="value.id" @contextmenu="themeMenu($event, value)" :colors="value.theme" />
            <div class="title">{{ value.name }}</div>
            <div class="author">
              {{ value.author }}
            </div>
          </div>
        </b-col>
        <b-col cols="5" xl="3" class="p-2">
          <div class="theme-component-container">
            <Add @click="openNewThemeModal" />
            {{ $t('settings.themes.createTheme') }}
          </div>
        </b-col>
      </b-row>
    </b-container>
    <DeleteModal v-if="themeToRemove" id="themeDeleteModal" :itemName="themeToRemove.name" @confirm="removeTheme" />
    <MultiButtonModal :slots="2" :show="showNewThemeModal" @click-1="createTheme" @click-2="importTheme">
      <template #1>
        <CreatePlaylistIcon />
      </template>
      <template #1-title>{{ $t('settings.themes.createTheme') }}</template>
      <template #2>
        <ImportThemeIcon />
      </template>
      <template #2-title>{{ $t('settings.themes.importTheme') }}</template>
    </MultiButtonModal>
  </div>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'
import ThemeComponentClassic from '../ThemeComponentClassic.vue'
import { v1 } from 'uuid'
import PreferenceHeader from '../PreferenceHeader.vue'
import ThemeComponentCompact from '../ThemeComponentCompact.vue'
import Add from '@/icons/AddThemeIcon.vue'
import DeleteModal from '@/commonComponents/ConfirmationModal.vue'
import MultiButtonModal from '../../../commonComponents/MultiButtonModal.vue'
import CreatePlaylistIcon from '@/icons/CreatePlaylistIcon.vue'
import ImportThemeIcon from '@/icons/ImportThemeIcon.vue'
import { bus } from '@/preferenceWindow/main'
import { MenuItem } from '../../../utils/ui/mixins/ContextMenuMixin'
import { ContextMenuInstance } from '@imengyu/vue3-context-menu'

@Component({
  components: {
    ThemeComponentClassic,
    ThemeComponentCompact,
    PreferenceHeader,
    DeleteModal,
    Add,
    MultiButtonModal,
    CreatePlaylistIcon,
    ImportThemeIcon
  }
})
export default class Themes extends Vue {
  allThemes: { [key: string]: ThemeDetails } = {}

  showNewThemeModal = false

  private async getAllThemes() {
    this.allThemes = (await window.ThemeUtils.getAllThemes()) ?? {}
  }

  private activeTheme = 'default'
  private activeView: songMenu = 'compact'

  menu: MenuItem[] = []

  get themesComponent() {
    return this.activeView === 'compact' ? 'ThemeComponentCompact' : 'ThemeComponentClassic'
  }

  get currentTheme() {
    return this.allThemes[this.activeTheme]?.theme ?? this.defaultTheme.theme
  }

  isThemeActive(themeId: string) {
    return themeId === this.activeTheme
  }

  isSongView(id: songMenu) {
    return id === this.activeView
  }

  private editTheme(theme: ThemeDetails) {
    this.$router.push({
      name: 'new_theme',
      query: {
        currentTheme: theme.id
      }
    })
  }

  themeToRemove: ThemeDetails | null = null

  private contextMenu: ContextMenuInstance | undefined

  themeMenu(event: MouseEvent, theme: ThemeDetails) {
    this.menu = []
    if (theme.id !== 'system_default' && theme.id !== 'default') {
      this.themeToRemove = theme
      this.menu.push({
        label: 'Delete',
        onClick: () => {
          this.$bvModal.show('themeDeleteModal')
        }
      })
      this.menu.push({
        label: 'Edit',
        onClick: () => {
          this.editTheme(theme)
        }
      })
    }
    this.menu.push({
      label: 'Copy to clipboard',
      onClick: () => {
        navigator.clipboard.writeText(JSON.stringify(theme))
      }
    })
    if (theme.id !== 'default') {
      this.menu.push({
        label: 'Export theme',
        onClick: () => {
          window.ThemeUtils.packTheme(theme.id)
        }
      })
    }

    this.contextMenu = this.$contextmenu({
      x: event.x,
      y: event.y,
      customClass: 'context-menu',
      items: this.menu,
    })
  }

  hideContextMenu() {
    this.contextMenu?.closeMenu()
  }

  async removeTheme() {
    const currentTheme = await window.ThemeUtils.getActiveTheme()
    if (currentTheme.id === this.themeToRemove?.id) {
      await this.setTheme('default')
    }

    this.themeToRemove && (await window.ThemeUtils.removeTheme(this.themeToRemove?.id))
    this.getAllThemes()
  }

  get defaultTheme(): ThemeDetails {
    return {
      id: 'default',
      name: 'Default',
      author: 'Moosync',
      theme: {
        primary: '#212121',
        secondary: '#282828',
        tertiary: '#151515',
        textPrimary: '#ffffff',
        textSecondary: '#565656',
        textInverse: '#000000',
        accent: '#65CB88',
        divider: 'rgba(79, 79, 79, 0.67)'
      }
    }
  }

  getRandomID() {
    return v1()
  }

  async setTheme(id: string) {
    await window.ThemeUtils.setActiveTheme(id)
    this.activeTheme = id
    bus.emit('themeChanged')
  }

  async setSongView(id: songMenu) {
    await window.ThemeUtils.setSongView(id)
    this.activeView = id
  }

  createTheme() {
    this.$router.push({
      name: 'new_theme'
    })
  }

  async importTheme() {
    const resp = await window.WindowUtils.openFileBrowser(false, true, [
      {
        name: 'Moosync theme (.mstx)',
        extensions: ['mstx']
      }
    ])

    for (const filePath of resp.filePaths ?? []) {
      await window.ThemeUtils.importTheme(filePath)
    }

    this.getAllThemes()
  }

  openNewThemeModal() {
    this.showNewThemeModal = !this.showNewThemeModal
  }

  async created() {
    this.activeTheme = (await window.ThemeUtils.getActiveTheme())?.id ?? 'default'
    this.activeView = (await window.ThemeUtils.getSongView()) ?? 'compact'
    await this.getAllThemes()
  }
}
</script>

<style lang="sass">
.context-menu
  position: fixed !important
  background: var(--secondary)
  ul li
    &:hover
      background: var(--accent)
      color: var(--textInverse) !important
</style>

<style lang="sass" scoped>
.path-selector
  max-width: 750px

.title, .author
  text-align: left

.title
  font-size: 16px
  font-weight: 700

.author
  font-size: 14px

.import-button
  font-size: 16px
  color: var(--accent)
  &:hover
    cursor: pointer
</style>
