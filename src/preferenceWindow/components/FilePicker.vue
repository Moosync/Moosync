<!-- 
  FilePicker.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="path-container w-100">
    <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" />
    <b-row no-gutters class="background w-100 mt-2 d-flex">
      <b-row no-gutters class="mt-3 item w-100">
        <b-col cols="auto" align-self="center" class="ml-4 folder-icon">
          <FolderIcon @click="openFileBrowser" />
        </b-col>
        <b-col
          :id="popoverTarget"
          cols="auto"
          align-self="center"
          :title="value"
          class="ml-3 justify-content-start"
          @click="copy"
        >
          <div class="item-text text-truncate">{{ value }}</div>
        </b-col>
        <b-popover
          id="clipboard-popover"
          :show.sync="showPopover"
          :target="popoverTarget"
          triggers="click blur"
          placement="top"
        >
          Copied!
        </b-popover>
      </b-row>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, mixins, Prop } from 'vue-facing-decorator'
import PreferenceHeader from './PreferenceHeader.vue'
import { ExtensionPreferenceMixin } from '../mixins/extensionPreferenceMixin'
import FolderIcon from '@/icons/FolderIcon.vue'
import { v4 } from 'uuid'

@Component({
  components: { PreferenceHeader, FolderIcon }
})
export default class FilePicker extends mixins(ExtensionPreferenceMixin) {
  declare value: string

  @Prop()
  title!: string

  @Prop()
  tooltip!: string

  popoverTarget = v4()
  showPopover = false
  private popoverTimeout: ReturnType<typeof setTimeout> | undefined

  openFileBrowser() {
    window.WindowUtils.openFileBrowser(false, false).then((data) => {
      if (!data.canceled && data.filePaths.length > 0) {
        this.value = data.filePaths[0]
        this.onInputChange(this.value)
      }
    })
  }

  copy() {
    if (this.popoverTimeout) {
      clearTimeout(this.popoverTimeout)
      this.popoverTimeout = undefined
    }

    navigator.clipboard.writeText(this.value ?? '')
    this.showPopover = true
    this.popoverTimeout = setTimeout(() => {
      this.showPopover = false
    }, 1000)
  }
}
</script>

<style lang="sass" scoped>
.title
  font-size: 26px

.background
  align-content: flex-start
  background-color: var(--tertiary)
  height: 65px
  overflow: hidden

.item
  height: 35px
  flex-wrap: nowrap

.item-text
  font-size: 18px
  color: var(--textSecondary)
  min-width: 0
  text-align: left

.folder-icon
  &:hover
    cursor: pointer
</style>
