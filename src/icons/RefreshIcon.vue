<!-- 
  RefreshIcon.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg viewBox="0 0 16 20" fill="none" xmlns="http://www.w3.org/2000/svg" @click="$emit('click', $event)">
    <title>{{ title }}</title>
    <path
      d="M13.91 12.5C13.432 15.333 10.967 17.5 8 17.5C4.692 17.5 2 14.808 2 11.5C2 8.192 4.692 5.5 8 5.5H10.172L8.086 7.586L9.5 9L14 4.5L9.5 0L8.086 1.414L10.172 3.5H8C5.87827 3.5 3.84344 4.34285 2.34315 5.84315C0.842855 7.34344 0 9.37827 0 11.5C0 13.6217 0.842855 15.6566 2.34315 17.1569C3.84344 18.6571 5.87827 19.5 8 19.5C12.079 19.5 15.438 16.445 15.931 12.5H13.91Z"
      fill="var(--textPrimary)"
    />
  </svg>
</template>

<script lang="ts">
import { Vue, Component, Prop } from 'vue-facing-decorator'

@Component({
  emits: ['click']
})
export default class RefreshIcon extends Vue {
  @Prop({ default: '' })
  title!: string
}
</script>

<style lang="sass" scoped>
*
  cursor: pointer
</style>
