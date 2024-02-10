<!-- 
  EditText.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container v-if="prefKey" fluid class="w-100">
    <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" @tooltipClick="emitTooltipClick" />
    <b-row no-gutters class="w-100 d-flex">
      <b-row no-gutters class="m-1 item w-100">
        <b-col cols="auto" align-self="center" class="w-100 h-100">
          <pre class="info-field">{{ value }}</pre>
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
export default class InfoField extends mixins(ExtensionPreferenceMixin) {
  @Prop()
  title!: string

  @Prop()
  tooltip!: string

  emitTooltipClick() {
    this.$emit('tooltipClick')
  }
}
</script>

<style lang="sass" scoped>
.title
  font-size: 26px

.item
  height: calc(100% - 1rem)

.info-field
  margin-top: -15px
  color: var(--text-primary)
  white-space: pre-line
</style>
