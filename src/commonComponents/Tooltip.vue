<!-- 
  Tooltip.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <svg v-b-tooltip.hover.right :title="text" :id="tooltipId" width="19" height="19" viewBox="0 0 19 19" fill="none"
    xmlns="http://www.w3.org/2000/svg">
    <circle cx="9.5" cy="9.5" r="9.5" fill="var(--textSecondary)" />
    <path
      d="M8.912 12.056V11.592C8.912 11.048 9.01333 10.5413 9.216 10.072C9.41867 9.592 9.776 9.05333 10.288 8.456C10.6933 7.98667 10.976 7.58667 11.136 7.256C11.296 6.92533 11.376 6.57333 11.376 6.2C11.376 5.752 11.216 5.4 10.896 5.144C10.576 4.87733 10.1333 4.744 9.568 4.744C8.98133 4.744 8.43733 4.85067 7.936 5.064C7.43467 5.27733 6.96 5.59733 6.512 6.024L6.032 5C6.448 4.57333 6.976 4.232 7.616 3.976C8.256 3.70933 8.928 3.576 9.632 3.576C10.5493 3.576 11.2907 3.81067 11.856 4.28C12.4213 4.73867 12.704 5.34667 12.704 6.104C12.704 6.60533 12.592 7.07467 12.368 7.512C12.1547 7.94933 11.776 8.456 11.232 9.032C10.7307 9.56533 10.3733 10.0347 10.16 10.44C9.95733 10.8347 9.82933 11.24 9.776 11.656L9.744 12.056H8.912ZM8.512 15V13.368H10.144V15H8.512Z"
      fill="var(--textPrimary)" />
  </svg>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'

@Component({})
export default class Tooltip extends Vue {
  @Prop({ default: 'tooltip' })
  tooltipId!: string

  @Prop({ default: 'This is a tooltip' })
  text!: string
}
</script>
