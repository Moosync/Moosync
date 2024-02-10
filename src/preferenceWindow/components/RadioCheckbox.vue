<!-- 
  CheckboxGroup.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="path-container w-100" v-if="prefKey && Array.isArray(value)">
    <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" />
    <b-row no-gutters class="item w-100" v-for="checkbox in defaultValue" :key="checkbox.key">
      <b-col cols="auto" align-self="center">
        <b-checkbox @change="toggleCheck(checkbox.key)" :value="checkbox.key" v-model="activeKey" />
      </b-col>
      <b-col col md="8" lg="9" align-self="center" class="ml-3 justify-content-start">
        <div class="item-text text-truncate" :title="checkbox.title">{{ checkbox.title }}</div>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, Prop, mixins } from 'vue-facing-decorator'
import { ExtensionPreferenceMixin } from '../mixins/extensionPreferenceMixin'
import PreferenceHeader from './PreferenceHeader.vue'
@Component({
  components: {
    PreferenceHeader
  }
})
export default class RadioCheckbox extends mixins(ExtensionPreferenceMixin) {
  @Prop()
  title!: string

  @Prop()
  tooltip!: string

  activeKey: string[] = []

  declare value: Checkbox[]
  declare defaultValue: Checkbox[]

  mounted() {
    this.postFetch = () => {
      this.value = this.defaultValue.map((val) => {
        return {
          title: val.title,
          key: val.key,
          enabled: this.value?.find((val2) => val.key === val2.key)?.enabled ?? val.enabled
        }
      })

      for (let i = 0; i < this.value.length; i++) {
        if (this.value[i].enabled) {
          this.activeKey = [this.value[i].key]
          break
        }
      }
    }
  }

  toggleCheck(key: string) {
    if (this.value) {
      this.value.forEach((val) => {
        val.enabled = val.key === key
      })
    }

    this.activeKey = [key]
    this.onInputChange(this.value)
  }

  private getCheckboxEnabled(key: string) {
    return key === this.activeKey[0]
  }
}
</script>

<style lang="sass" scoped>
.title
  font-size: 26px

.background
  align-content: flex-start
  overflow-y: scroll
  overflow-x: hidden

.item
  height: 35px
  flex-wrap: nowrap

.item-text
  font-size: 18px
  color: var(--textPrimary)
  min-width: 0
  text-align: left
</style>
