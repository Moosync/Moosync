<!-- 
  Timestamp.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-row no-gutters align-v="center">
    <b-col cols="auto" class="timestamp">{{ formattedDuration(timestamp) }} </b-col>
    <b-col cols="auto" class="timestamp timestamp-extra">&nbsp;/ {{ formattedDuration(duration) }}</b-col>
  </b-row>
</template>

<script lang="ts">
import { Component, Vue, Prop } from 'vue-facing-decorator'
import { convertDuration } from '@/utils/common'

@Component({})
export default class Timestamp extends Vue {
  @Prop({ default: 0 })
  duration!: number

  @Prop({ default: 0 })
  timestamp!: number

  formattedDuration = convertDuration
}
</script>

<style lang="sass" scoped>
.timestamp
  color: var(--textSecondary)
  font-size: 16px
  overflow: hidden
</style>
