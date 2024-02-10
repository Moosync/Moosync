<!-- 
  QueueIcon.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    width="18"
    height="18"
    viewBox="0 0 24 21"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <title>Queue</title>
    <path
      d="M6 0.666504H24V2.6665H6V0.666504ZM6 6.6665H24V8.6665H6V6.6665ZM11 12.6665H24V14.6665H11V12.6665ZM6 18.6665H24V20.6665H6V18.6665ZM0 8.6665L7 13.6665L0 18.6665V8.6665Z"
      :fill="active ? 'var(--accent)' : 'var(--textPrimary)'"
    />
  </svg>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'

@Component({
  emits: ['click']
})
export default class Playlists extends Vue {
  @Prop({ default: false })
  active!: boolean
}
</script>
