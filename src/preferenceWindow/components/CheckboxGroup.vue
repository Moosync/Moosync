<!-- 
  CheckboxGroup.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="path-container w-100" v-if="prefKey && Array.isArray(value)">
    <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" />
    <b-row no-gutters class="item w-100" v-for="checkbox in value" :key="checkbox.key">
      <b-col cols="auto" align-self="center">
        <b-checkbox
          @change="toggleCheck(checkbox.key)"
          :ref="`checkbox-${packageName}-${checkbox.key}`"
          :checked="getCheckboxEnabled(checkbox.key)"
        />
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
import { BFormCheckbox } from 'bootstrap-vue'

@Component({
  components: {
    PreferenceHeader
  }
})
export default class CheckboxGroup extends mixins(ExtensionPreferenceMixin) {
  @Prop()
  title!: string

  @Prop()
  tooltip!: string

  declare defaultValue: Checkbox[]
  declare value: Checkbox[]

  created() {
    this.postFetch = () => {
      this.value = this.defaultValue.map((val) => {
        return {
          title: val.title,
          key: val.key,
          enabled: this.getCheckboxEnabled(val.key)
        }
      })
    }
  }

  toggleCheck(key: string) {
    const isChecked = (this.$refs[`checkbox-${this.packageName}-${key}`] as BFormCheckbox[])?.at(0)?.isChecked
    if (this.value) {
      const value = this.getValueByKey(key)
      if (value) {
        value.enabled = isChecked
      }

      this.onInputChange(this.value)
    }

    // ;(this.value as Checkbox[])[index].enabled = (
    // (this.$refs[`checkbox-${this.packageName}-${(this.value as Checkbox[])[index].key}`] as HTMLInputElement).checked)
    //   document.getElementById(
    //     `checkbox-${this.packageName}-${(this.value as Checkbox[])[index].key}`
    //   ) as HTMLInputElement
    // ).checked
    // this.onInputChange()
  }

  private getValueByKey(key: string) {
    return this.value?.find((val) => val.key === key)
  }

  getCheckboxEnabled(key: string) {
    return this.getValueByKey(key)?.enabled ?? false
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
