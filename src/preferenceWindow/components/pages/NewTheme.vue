<!-- 
  NewTheme.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="h-100 w-100">
    <b-row no-gutters> </b-row>
    <b-row no-gutters>
      <b-col class="h-100">
        <b-row no-gutters class="metadata mb-3">
          <b-input v-model="title" class="theme-title" maxlength="20"
            :placeholder="$t('settings.themes.newTheme.name')" />
          <b-input v-model="author" class="theme-title" maxlength="20"
            :placeholder="$t('settings.themes.newTheme.author')" />
        </b-row>
        <b-row no-gutters>
          <b-col cols="6" class="preview-col">
            <b-row no-gutters class="preview mb-5">
              <ThemeComponentClassic class="h-100" :colors="customTheme" :id="getRandomID()"
                @colorClick="toggleColorPicker" />
            </b-row>
            <b-row no-gutters class="preview">
              <ThemeComponentCompact class="h-100" :colors="customTheme" :id="getRandomID()"
                @colorClick="toggleColorPicker" />
            </b-row>
          </b-col>
          <b-col cols="auto" class="color-col ml-5">
            <PreferenceHeader :title="$t('settings.themes.newTheme.colors')"
              :tooltip="$t('settings.themes.newTheme.colors_tooltip')" />
            <table>
              <ColorPicker v-for="item in themeKeys" :key="item" :ref="item" :defColor="customTheme[item]"
                :title="getThemeTitle(item)" @colorChange="(color: string) => onColorChange(item, color)" />
            </table>
          </b-col>
        </b-row>
        <PreferenceHeader :title="$t('settings.themes.newTheme.css')" tooltip="Optional" class="mt-4" />
        <b-row no-gutters class="background w-100 mt-2 d-flex">
          <b-row no-gutters class="mt-3 item">
            <b-col cols="auto" align-self="center" class="ml-4 folder-icon">
              <FolderIcon @click="openFileBrowser" />
            </b-col>
            <b-col :id="popoverTarget" cols="auto" align-self="center" :title="customTheme.customCSS"
              class="ml-3 justify-content-start" @click="copy">
              <div class="item-text text-truncate">{{ customTheme.customCSS }}</div>
            </b-col>
            <b-popover id="clipboard-popover" :show.sync="showPopover" :target="popoverTarget" triggers="click blur"
              placement="top">
              Copied!
            </b-popover>
          </b-row>
          <b-col cols="auto" align-self="center" class="ml-4 cross-icon">
            <CrossIcon color="#E62017" @click="clearCustomCSS" />
          </b-col>
        </b-row>
        <b-row class="mt-2">
          <CheckboxGroup key="css_options" :defaultValue="cssOptions" :onValueChange="cssOptionsChanged" />
        </b-row>
        <b-row class="mt-5 mr-4" align-h="end">
          <b-button class="cancel-button mr-4" @click="dismiss">{{ $t('buttons.cancel') }}</b-button>
          <b-button class="confirm-button" @click="saveTheme">{{ $t('buttons.save') }}</b-button>
        </b-row>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-facing-decorator'
import ThemeComponentClassic from '../ThemeComponentClassic.vue'
import { v1, v4 } from 'uuid'
import PreferenceHeader from '../PreferenceHeader.vue'
import ThemeComponentCompact from '../ThemeComponentCompact.vue'
import ColorPicker from '../ColorPicker.vue'
import NavBack from '@/icons/NavBackIcon.vue'
import FolderIcon from '@/icons/FolderIcon.vue'
import { bus } from '@/preferenceWindow/main'
import { convertProxy } from '../../../utils/ui/common'
import CrossIcon from '../../../icons/CrossIcon.vue';
import CheckboxGroup from '../CheckboxGroup.vue';

@Component({
  components: {
    ThemeComponentClassic,
    ThemeComponentCompact,
    PreferenceHeader,
    ColorPicker,
    NavBack,
    FolderIcon,
    CrossIcon,
    CheckboxGroup
  }
})
export default class NewTheme extends Vue {
  customTheme: ThemeItem = this.defaultTheme

  title = ''
  author = ''
  private currentThemeID = ''

  popoverTarget = v4()
  showPopover = false
  private popoverTimeout: ReturnType<typeof setTimeout> | undefined

  private lastActivePath: string | undefined

  themeKeys: ThemeKey[] = [
    'primary',
    'secondary',
    'tertiary',
    'textPrimary',
    'textSecondary',
    'textInverse',
    'accent',
    'divider'
  ]

  get defaultTheme() {
    return {
      primary: '#212121',
      secondary: '#282828',
      tertiary: '#151515',
      textPrimary: '#ffffff',
      textSecondary: '#565656',
      textInverse: '#000000',
      accent: '#65CB88',
      divider: 'rgba(79, 79, 79, 0.67)',
      customCSS: ''
    }
  }

  getThemeTitle(key: string) {
    return this.$t(`settings.themes.${key}`)
  }

  toggleColorPicker(type: ThemeKey) {
    // eslint-disable-next-line @typescript-eslint/no-extra-semi
    ; (this.$refs[type] as ColorPicker[])[0]?.toggleColorPicker()
  }

  onColorChange(attr: ThemeKey, color: string) {
    this.customTheme[attr] = color
  }

  getRandomID() {
    return v1()
  }

  mounted() {
    window.NotifierUtils.onFileChanged(async (path) => {
      if (path === this.lastActivePath)
        await window.ThemeUtils.setTempTheme(convertProxy(this.generateThemeMetadata(), true))
    })

    this.listenPathChanges()
  }

  private generateThemeMetadata(): ThemeDetails {
    return {
      id: this.currentThemeID,
      name: this.title,
      author: this.author,
      theme: this.customTheme
    }
  }

  async dismiss() {
    await this.stopListeningPath()
    this.$router.push({
      name: 'themes'
    })
  }

  async saveTheme() {
    await window.ThemeUtils.saveTheme(convertProxy(this.generateThemeMetadata(), true))
    const currentTheme = await window.ThemeUtils.getActiveTheme()
    if (currentTheme.id === this.currentThemeID) {
      await window.ThemeUtils.setActiveTheme(this.currentThemeID)
      bus.emit('themeChanged')
    }
    this.dismiss()
  }

  private async parseClipboard() {
    const text = await navigator.clipboard.readText()
    try {
      const parsed: ThemeDetails = JSON.parse(text)
      if (parsed.name && parsed.author && parsed.theme) {
        for (const key of Object.keys(parsed.theme)) {
          if (this.themeKeys.includes(key as ThemeKey) && parsed.theme[key as keyof ThemeItem]) {
            const color = parsed.theme[key as keyof ThemeItem]
            if (color?.length === 7 && color?.startsWith('#')) {
              continue
            }
            parsed.theme[key as keyof ThemeItem] = this.defaultTheme[key as keyof ThemeItem]
          }
        }
        this.customTheme = parsed.theme
        this.title = parsed.name
        this.author = parsed.author
      }
    } catch (_) {
      return
    }
  }

  async created() {
    this.parseClipboard()
    this.currentThemeID = this.$route.query['currentTheme']?.toString() ?? ''
    if (this.currentThemeID) {
      const theme = await window.ThemeUtils.getTheme(this.currentThemeID)
      if (theme) {
        this.customTheme = theme.theme
        this.title = theme.name
        this.author = theme.author
      }
    } else {
      this.currentThemeID = v1()
    }
  }

  clearCustomCSS() {
    this.stopListeningPath()
    this.customTheme['customCSS'] = undefined
  }

  private async stopListeningPath() {
    if (this.lastActivePath)
      await window.NotifierUtils.watchFileChanges(this.lastActivePath, false, false)
  }

  cssOptionsChanged(options: Checkbox[]) {
    const shouldWatch = options[0].enabled
    if (this.customTheme['customCSS']) {
      this.stopListeningPath()
      window.NotifierUtils.watchFileChanges(this.customTheme['customCSS'], shouldWatch, false)
      this.lastActivePath = this.customTheme['customCSS']
      window.ThemeUtils.setTempTheme(convertProxy(this.generateThemeMetadata(), true))
    }
  }

  get cssOptions(): Checkbox[] {
    return [
      {
        'enabled': false,
        'key': 'watch_css_changes',
        'title': this.$t('settings.themes.newTheme.watch_css')
      }
    ]
  }

  private async listenPathChanges() {
    const cssOptions = await window.PreferenceUtils.loadSelective<Checkbox[]>('css_options.watch_css_changes') ?? this.cssOptions
    this.cssOptionsChanged(cssOptions)
  }

  async openFileBrowser() {
    const data = await window.WindowUtils.openFileBrowser(false, true, [
      {
        extensions: ['css'],
        name: 'CSS Stylesheets'
      }
    ])

    if (!data.canceled && data.filePaths.length > 0) {
      this.customTheme['customCSS'] = data.filePaths[0]
      await this.listenPathChanges()
    }
  }

  copy() {
    if (this.popoverTimeout) {
      clearTimeout(this.popoverTimeout)
      this.popoverTimeout = undefined
    }

    navigator.clipboard.writeText(this.customTheme.customCSS ?? '')
    this.showPopover = true
    this.popoverTimeout = setTimeout(() => {
      this.showPopover = false
    }, 1000)
  }
}
</script>

<style lang="sass" scoped>
.preview, .metadata
  min-width: 320px
  max-width: 600px

.preview-col
  @media (max-width: 996px)
    display: none !important

.preview-col
  max-width: 600px

.color-col
  @media (max-width: 996px)
    margin-left: 0 !important

.theme-title
  font-size: 18px
  max-width: 100%
  margin-bottom: 15px !important
  color: var(--textPrimary) !important
  background-color: transparent !important
  border: 0
  border-bottom: 1px solid var(--divider)
  border-radius: 0
  padding: 0
  &:hover
    border-bottom: 1px solid var(--accent)
  &:focus
    outline: none
    border-bottom: 1px solid var(--accent)
    -webkit-box-shadow: none

.background
  align-content: flex-start
  background-color: var(--tertiary)
  height: 65px
  overflow: hidden

.item
  height: 35px
  flex-wrap: nowrap
  width: 80%

.item-text
  font-size: 18px
  color: var(--textSecondary)
  min-width: 0
  text-align: left

.folder-icon
  &:hover
    cursor: pointer

.vacp-color-picker
  background-color: var(--secondary)
  color: var(--textPrimary)

.vacp-copy-button, .vacp-format-switch-button
    background-color: var(--primary)

.vacp-icon
  path
    fill: var(--textPrimary)

.vacp-color-input
  background-color: var(--primary)
  border: var(--vacp-border-width) solid var(--textSecondary)

.cross-icon
  width: 20px
  position: absolute
  right: 20px

</style>
