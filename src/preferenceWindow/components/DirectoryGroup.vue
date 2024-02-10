<!-- 
  DirectoryGroup.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="path-container w-100">
    <b-row no-gutters align-v="center">
      <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" />
      <b-col
        cols="auto"
        align-self="start"
        class="new-directories ml-auto justify-content-center"
        v-if="showRefreshIcon"
      >
        <RefreshIcon title="Rescan directories for music" @click="emitRefresh" class="refresh-icon button-grow" />
      </b-col>
      <b-col cols="auto" :class="`new-directories ${showRefreshIcon ? 'ml-4' : 'ml-auto'}`">
        <div class="add-directories-button" v-if="!bottomButton" @click="openFileBrowser">
          {{ $t('settings.paths.addFolder') }}
        </div>
      </b-col>
    </b-row>
    <b-row
      no-gutters
      :style="{ height: height * 51 + 'px' }"
      class="background w-100 mt-2 d-flex"
      v-if="Array.isArray(value)"
    >
      <b-row no-gutters class="mt-3 item w-100" v-for="(path, index) in value" :key="path.path">
        <b-col v-if="enableCheckbox" cols="auto" align-self="center" class="ml-4">
          <b-checkbox @change="togglePath(index)" :id="`path-${packageName}-${path.path}`" :checked="path.enabled" />
        </b-col>
        <b-col
          col
          md="8"
          lg="9"
          align-self="center"
          :class="{ 'no-checkbox-margin': !enableCheckbox, 'ml-3': enableCheckbox }"
          class="justify-content-start"
        >
          <div class="item-text text-truncate" :title="path.path">{{ path.path }}</div>
        </b-col>
        <b-col cols="auto" align-self="center" class="ml-auto">
          <div class="remove-button w-100" @click="removePath(index)">{{ $t('settings.paths.remove') }}</div>
        </b-col>
      </b-row>
    </b-row>
    <b-row>
      <b-col cols="auto" align-self="center" class="new-directories mt-3">
        <div class="add-directories-button" v-if="bottomButton" @click="openFileBrowser">
          {{ $t('settings.paths.addFolder') }}
        </div>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
type DirectoryGroupValue = {
  path: string
  enabled: boolean
}[]

import { Component, Prop, mixins } from 'vue-facing-decorator'
import Tooltip from '@/commonComponents/Tooltip.vue'
import { ExtensionPreferenceMixin } from '../mixins/extensionPreferenceMixin'
import PreferenceHeader from './PreferenceHeader.vue'
import RefreshIcon from '@/icons/RefreshIcon.vue'
@Component({
  components: {
    Tooltip,
    PreferenceHeader,
    RefreshIcon
  },
  emits: ['refresh']
})
export default class DirectoryGroup extends mixins(ExtensionPreferenceMixin) {
  @Prop({ default: 5 })
  height!: number

  @Prop({ default: true })
  enableCheckbox!: boolean

  @Prop({ default: false })
  bottomButton!: boolean

  @Prop({ default: false })
  isMainWindow!: boolean

  @Prop({ default: false })
  showRefreshIcon!: boolean

  @Prop()
  title!: string

  @Prop()
  tooltip!: string

  declare value: DirectoryGroupValue

  togglePath(index: number) {
    if (this.value && index >= 0) {
      const path = this.value[index]
      this.value[index].enabled = (
        document.getElementById(`path-${this.packageName}-${path.path}`) as HTMLInputElement
      ).checked
      this.onInputChange(this.value)
    }
  }

  emitRefresh() {
    this.$emit('refresh')
  }

  removePath(index: number) {
    if (this.value && index >= 0) {
      this.value.splice(index, 1)
      this.onInputChange(this.value)
    }
  }

  openFileBrowser() {
    window.WindowUtils.openFileBrowser(this.isMainWindow, false).then((data) => {
      if (!data.canceled && this.value) {
        for (const path of data.filePaths) {
          this.value.push({ path, enabled: true })
        }
        this.onInputChange(this.value)
      }
    })
  }
}
</script>

<style lang="sass">
.custom-control-input:checked + .custom-control-label::before
  background-color: transparent
  border-color: var(--textPrimary)

.custom-control-input:indeterminate ~ .custom-control-label
  background-image: none
  box-shadow: none

.custom-control-input:focus ~ .custom-control-label::before
  outline: var(--textPrimary) !important
  border: 1px solid var(--textPrimary) !important
  box-shadow: 0 0 1px 1px var(--textPrimary)

.custom-control-label
  &::before
    background-color: transparent
</style>

<style lang="sass" scoped>
.new-directories
  font-size: 16px
  color: var(--accent)
  &:hover
    cursor: pointer

.add-directories-button
  user-select: none

.background
  align-content: flex-start
  background-color: var(--tertiary)
  overflow-y: scroll
  overflow-x: hidden

  &::-webkit-scrollbar-track
    background: var(--tertiary)

.item
  height: 35px
  flex-wrap: nowrap

.item-text
  font-size: 18px
  color: var(--textSecondary)
  min-width: 0
  text-align: left

.remove-button
  color: #E62017
  user-select: none
  &:hover
    cursor: pointer

.no-checkbox-margin
  margin-left: 25px

.refresh-icon
  width: 16px
</style>
