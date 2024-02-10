<!-- 
  AddToLibraryIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    class="button-grow"
    width="20"
    height="24"
    viewBox="0 0 20 24"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <title>{{ title }}</title>
    <path
      d="M18.75 8.25H13.75V0.75H6.25V8.25H1.25L10 18.25L18.75 8.25ZM0 20.75H20V23.25H0V20.75Z"
      fill="var(--accent)"
    />
  </svg>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'

@Component({
  emits: ['click']
})
export default class PlainPlay extends Vue {
  @Prop({ default: '' })
  title!: string
}
</script>

<style lang="sass" scoped>
*
  cursor: pointer
</style>
