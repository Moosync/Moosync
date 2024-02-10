<!-- 
  NextTrackIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    class="button-grow"
    width="24"
    height="16"
    viewBox="0 0 24 16"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <path
      d="M1.80574 1.34454C1.59713 1.23814 1.34822 1.24785 1.14853 1.37016C0.948842 1.49247 0.827102 1.7098 0.827102 1.94397V14.0561C0.827102 14.2903 0.948846 14.5076 1.14854 14.6299C1.34823 14.7523 1.59715 14.762 1.80575 14.6556L9.81366 10.5709V14.0561C9.81366 14.2903 9.9354 14.5076 10.1351 14.6299C10.3348 14.7523 10.5837 14.762 10.7923 14.6556L22.6656 8.59926C22.8909 8.48434 23.0327 8.25276 23.0327 7.99983C23.0327 7.7469 22.8909 7.51533 22.6656 7.40041L10.7923 1.34454C10.5837 1.23814 10.3348 1.24785 10.1351 1.37016C9.9354 1.49247 9.81366 1.7098 9.81366 1.94397V5.42892L1.80574 1.34454Z"
      :stroke="disabled ? 'var(--textSecondary)' : 'var(--accent)'"
      stroke-width="1.3458"
      stroke-linecap="round"
      stroke-linejoin="round"
    />
  </svg>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'

@Component({
  emits: ['click']
})
export default class NextTrack extends Vue {
  @Prop({ default: false })
  disabled!: boolean
}
</script>

<style lang="sass" scoped>
*
  cursor: pointer
</style>
