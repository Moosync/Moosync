<!-- 
  PreferenceHeader.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-row no-gutters>
    <b-col v-if="title" cols="auto" align-self="center" class="title d-flex preference-title">
      {{ title }}
    </b-col>
    <b-col cols="auto" class="ml-2">
      <Tooltip v-if="tooltip" :text="tooltip" @click="emitTooltipClick" />
    </b-col>
  </b-row>
</template>

<script lang="ts">
import Tooltip from '@/commonComponents/Tooltip.vue'
import { Component, Prop, Vue } from 'vue-facing-decorator'

@Component({
  components: { Tooltip },
  emits: ['tooltipClick']
})
export default class PreferenceHeader extends Vue {
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
.preference-title
  font-weight: normal
  font-size: 20px
  color: var(--textPrimary)

.tooltip-pos
  margin-top: 2px
</style>
