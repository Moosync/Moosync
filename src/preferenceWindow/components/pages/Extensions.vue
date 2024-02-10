<!-- 
  Extensions.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="w-100 h-100">
    <b-container fluid class="h-100 w-100">
      <b-row no-gutters>
        <b-col cols="8" md="8" class="tabs-col">
          <ul role="tablist" class="nav nav-tabs">
            <li
              role="presentation"
              class="nav-item"
              v-for="(ext, index) of currentTabs"
              :key="ext.packageName"
              :style="{
                maxWidth: `min(${
                  computedTabAmount < offset ? 'calc(100% /' + computedTabAmount + ')' : '140px'
                }, calc(100% / ${computedTabAmount}))`
              }"
              @click="activeTab = index"
            >
              <a class="nav-link active ext-tab text-truncate" aria-controls="__BVID__41">{{ ext.name }}</a>
            </li>
          </ul>
        </b-col>
        <b-col cols="auto" align-self="center" class="d-flex ml-4" v-if="maxPage > 1">
          <PrevIcon class="mr-3" @click="prevPage"></PrevIcon>
          <div class="page-no">{{ page + 1 }} of {{ maxPage }}</div>
          <NextIcon class="ml-3" @click="nextPage"></NextIcon>
        </b-col>
      </b-row>
      <b-row no-gutters class="mt-4 content-row flex-grow-1">
        <b-col>
          <b-row no-gutters class="single-pref-row">
            <ExtensionGroup @extensionsChanged="onExtensionChanged" :extensions="extensions" />
          </b-row>
          <b-container fluid v-for="ext of extensionsWithPrefs" :key="ext.packageName" ref="extContent" class="mt-4">
            <b-row no-gutters>
              <div class="ext-title">{{ ext.name }}</div>
            </b-row>
            <b-row no-gutters v-for="pref of ext.preferences" :key="pref.key" class="single-pref-row mt-4">
              <component
                v-if="isComponentExists(pref.type)"
                :title="pref.title"
                :tooltip="pref.description"
                :key="`${ext.packageName}.${pref.key}`"
                :defaultValue="(pref as any).default ? (pref as any).default : (pref as any).items"
                :isExtension="true"
                :packageName="ext.packageName"
                :type="(pref as any).inputType || 'text'"
                :is="pref.type"
              />
            </b-row>
          </b-container>
        </b-col>
      </b-row>
    </b-container>
  </div>
</template>

<script lang="ts">
import { Component, Ref } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'
import EditText from '../EditText.vue'
import DirectoryGroup from '../DirectoryGroup.vue'
import FilePicker from '../FilePicker.vue'
import CheckboxGroup from '../CheckboxGroup.vue'
import PrevIcon from '@/icons/PrevIcon.vue'
import NextIcon from '@/icons/NextIcon.vue'
import ExtensionGroup from '../ExtensionGroup.vue'
import ProgressBar from '../Progressbar.vue'
import ButtonGroup from '../ButtonGroup.vue'
import TextField from '../TextField.vue'
import InfoField from '../InfoField.vue'
import { nextTick } from 'vue'

@Component({
  components: {
    EditText,
    DirectoryGroup,
    FilePicker,
    CheckboxGroup,
    ExtensionGroup,
    TextField,
    ProgressBar,
    ButtonGroup,
    PrevIcon,
    NextIcon,
    InfoField
  }
})
export default class Extensions extends Vue {
  extensions: ExtensionDetails[] = []
  page = 0
  offset = 5
  activeTab_ = 0

  @Ref('extContent')
  private content!: HTMLDivElement[]

  created() {
    this.fetchExtensions()
    window.ExtensionUtils.listenRequests(() => {
      this.fetchExtensions()
    })
  }

  get computedTabAmount() {
    if (this.maxPage === 1) {
      return this.extensionsWithPrefs.length
    }
    return this.offset
  }

  get extensionsWithPrefs() {
    return this.extensions.filter((val) => val.preferences.length > 0)
  }

  get activeTab() {
    return this.activeTab_
  }

  set activeTab(index: number) {
    this.activeTab_ = this.page * this.offset + index
    this.content[this.activeTab_].scrollIntoView({ behavior: 'smooth' })
  }

  private isActive(packageName: string) {
    return packageName === this.extensions[this.activeTab_].packageName
  }

  get maxPage() {
    return Math.ceil(this.extensionsWithPrefs.length / this.offset)
  }

  get currentTabs() {
    return this.extensionsWithPrefs.slice(this.page * this.offset, this.page * this.offset + this.offset)
  }

  nextPage() {
    if (this.page * this.offset + this.offset < this.extensions.length) this.page++
  }

  prevPage() {
    if (this.page > 0) this.page--
  }

  isComponentExists(type: string) {
    if (this.$options.components) return !!this.$options.components[type]
    return false
  }

  private openFileBrowser() {
    window.WindowUtils.openFileBrowser(false, true).then((data) => {
      if (!data.canceled) {
        window.ExtensionUtils.install(...data.filePaths).then((result) => {
          if (result.success) {
            nextTick().then(() => this.fetchExtensions())
          }
        })
      }
    })
  }

  private async fetchExtensions() {
    this.extensions = await window.ExtensionUtils.getAllExtensions()
    for (const e of this.extensions) {
      e.preferences = e.preferences.sort((a, b) => (a.index ?? Infinity) - (b.index ?? Infinity))
    }
  }

  onExtensionChanged() {
    return this.fetchExtensions()
  }
}
</script>

<style lang="sass">
.ext-tab
  user-select: none
  color: var(--textPrimary)
  width: 100% !important
  margin: 0
  &:hover
    color: var(--textPrimary)
    border-color: transparent !important
.nav-tabs
  width: 100% !important
  border-bottom: none

.nav-item
  background: var(--secondary)
  border-radius: 0px 26px 0px 0px
  border-bottom: 2px solid transparent
  cursor: pointer
  .active
    background: var(--secondary) !important
    color: var(--textPrimary) !important
    border-color: transparent !important
    border-radius: 0px 26px 0px 0px
    border: 1px transparent
    border-bottom: 2px solid transparent
  .manual-active
    border-bottom: 2px solid var(--textPrimary) !important
</style>

<style lang="sass" scoped>
*
  text-align: left
.pref-container
  margin-left: 0 !important

.add-extension-button
  font-size: 22px
  color: var(--textPrimary)

.page-no
  user-select: none

.tabs-col
  max-width: calc(140px * 5)

.content-row
  overflow-y: scroll

.ext-title
  font-size: 26px
  font-weight: normal

.single-pref-row
  margin-bottom: 15px
</style>
