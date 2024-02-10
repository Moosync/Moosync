<!-- 
  EditText.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container v-if="prefKey" fluid class="path-container w-100">
    <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" @tooltipClick="emitTooltipClick" />
    <b-row no-gutters class="background w-100 mt-2 d-flex">
      <b-row no-gutters class="mt-3 item w-100">
        <b-col cols="auto" align-self="center" class="flex-grow-1 justify-content-start">
          <b-input
            class="dropdown-input"
            :debounce="debounce"
            v-model="value"
            @update="onInputChange"
            list="datalist"
          ></b-input>
          <datalist id="datalist">
            <option v-for="option of datalist" :key="option" :value="option">{{ option }}</option>
          </datalist>
        </b-col>
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
export default class AutoFillEditText extends mixins(ExtensionPreferenceMixin) {
  @Prop()
  title!: string

  @Prop()
  tooltip!: string

  @Prop()
  datalist!: string[]

  @Prop({ default: 500 })
  debounce!: number

  emitTooltipClick() {
    this.$emit('tooltipClick')
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

.dropdown-input
  background-color: var(--tertiary)
  border: none
  color: var(--textPrimary)
  -webkit-background-clip: text !important
  &:focus
    box-shadow: none
  &:-internal-autofill-selected
    -webkit-box-shadow: 0 0 0px 1000px var(--tertiary) inset
    -webkit-text-fill-color: var(--textPrimary)
    background-color: var(--tertiary) !important
    color: var(--textPrimary) !important
    appearance: auto !important
</style>
