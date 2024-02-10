<!-- 
  EditText.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container v-if="prefKey" fluid class="path-container w-100">
    <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" @tooltipClick="emitTooltipClick" />
    <b-row no-gutters :class="`${type !== 'range' ? 'background mt-2' : ''} w-100  d-flex`">
      <b-row no-gutters :class="`${type !== 'range' ? 'mt-3 item' : 'mb-2'} w-100`">
        <b-col cols="auto" align-self="center" class="ml-4 folder-icon"> </b-col>
        <b-col cols="auto" align-self="center" class="flex-grow-1 justify-content-start">
          <input :type="type" v-model="value" id="ext-input" class="ext-input w-100"
            :class="`${type === 'range' ? 'slider' : 'ext-input-hover'}`" v-bind:style="{
              background: `${type === 'range' ? computedGradient : 'inherit'}`
            }" @change="formatAndUpdate" />
        </b-col>
        <b-col class="range-text" v-if="showRangeText && type === 'range'">{{ value }}%</b-col>
        <b-col cols="auto" class="mr-4"></b-col>
      </b-row>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import PreferenceHeader from './PreferenceHeader.vue'
import { ExtensionPreferenceMixin } from '../mixins/extensionPreferenceMixin'

@Component({
  components: {
    PreferenceHeader
  },
  emits: ['tooltipClick']
})
export default class EditText extends mixins(ExtensionPreferenceMixin) {
  @Prop()
  title!: string

  @Prop()
  tooltip!: string

  @Prop({ default: 500 })
  debounce!: number

  @Prop({ default: null })
  maxValue!: number | null

  @Prop({ default: false })
  onlyNumber!: boolean

  @Prop({ default: false })
  showRangeText!: boolean

  declare value: string

  get computedGradient() {
    return `linear-gradient(90deg, var(--accent) 0%, var(--accent) ${Math.min(
      100,
      Math.max(parseInt(this.value ?? '0'), 0)
    )}%, var(--textSecondary) 0%)`
  }

  emitTooltipClick() {
    this.$emit('tooltipClick')
  }

  private debounceTimer: ReturnType<typeof setTimeout> | undefined = undefined
  formatAndUpdate() {
    const formatted = this.formatVal(this.value ?? '')
    this.value = formatted

    clearTimeout(this.debounceTimer)
    this.debounceTimer = setTimeout(() => this.onInputChange(this.value), this.maxValue ? 0 : this.debounce)
  }

  private formatVal(input: string) {
    let ret = input

    if (typeof ret === 'string') {
      if (this.maxValue) {
        ret = ret.substring(0, this.maxValue)
      }

      if (this.onlyNumber) {
        ret = ret.replace(/\D/g, '')
      }
    }
    return ret
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

.ext-input
  font-size: 16px
  max-width: 100%
  margin-bottom: 0px !important
  color: var(--textPrimary)
  background-color: transparent

  border-radius: 0
  padding: 0


.ext-input-hover
  border: 0
  border-bottom: 1px solid transparent
  &:hover
    border-bottom: 1px solid var(--accent)
  &:focus
    border-bottom: 1px solid var(--accent)
    outline: none
    -webkit-box-shadow: none

.slider
  right: 0
  -webkit-appearance: none
  height: 2px
  outline: none
  visibility: visible

.slider::-webkit-slider-thumb
  -webkit-appearance: none
  appearance: none
  width: 12px
  height: 12px
  border-radius: 50%
  background: var(--accent)

.slider::-ms-fill-upper
  background-color: var(--primary)

.range-text
  color: var(--textSecondary)
</style>
