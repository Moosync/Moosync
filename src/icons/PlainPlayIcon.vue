<!-- 
  PlainPlayIcon.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg
    class="button-grow"
    width="22"
    height="22"
    viewBox="0 0 20 20"
    fill="none"
    xmlns="http://www.w3.org/2000/svg"
    @click="$emit('click', $event)"
  >
    <title>{{ title }}</title>
    <path
      d="M1.2236 0.552783C1.06861 0.475289 0.884538 0.483573 0.737132 0.574676C0.589725 0.66578 0.5 0.826712 0.5 1V19C0.5 19.1733 0.589727 19.3342 0.737137 19.4253C0.884547 19.5164 1.06862 19.5247 1.22361 19.4472L19.2236 10.4469C19.393 10.3622 19.5 10.1891 19.5 9.99969C19.5 9.8103 19.393 9.63717 19.2236 9.55247L1.2236 0.552783Z"
      fill="var(--accent)"
      stroke="var(--accent)"
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
export default class PlainPlay extends Vue {
  @Prop({ default: '' })
  title!: string
}
</script>

<style lang="sass" scoped>
*
  cursor: pointer
</style>
