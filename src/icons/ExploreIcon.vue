<!-- 
  ExploreIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    width="18"
    height="18"
    viewBox="0 0 18 18"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <path
      d="M11.75 11.7501C9.4028 14.0973 5.5972 14.0973 3.24998 11.7501C0.902752 9.40285 0.902752 5.59725 3.24998 3.25002C5.5972 0.902801 9.4028 0.902801 11.75 3.25002C14.0972 5.59725 14.0972 9.40285 11.75 11.7501ZM11.75 11.7501L16.5 16.5"
      :stroke="active ? 'var(--accent)' : 'var(--textPrimary)'"
      stroke-width="2"
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
export default class Explore extends Vue {
  @Prop({ default: false })
  active!: boolean
}
</script>
