<!-- 
  InputGroup.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="path-container w-100">
    <b-row no-gutters class="background w-100 mt-2 d-flex">
      <b-col cols="auto" align-self="center" class="h-100 ml-3 mr-3">
        <Link />
      </b-col>
      <b-col cols="auto" align-self="center" class="flex-grow-1 justify-content-start">
        <b-input :placeholder="hint" id="ext-input" class="ext-input" debounce="500" @update="onInputChange" />
      </b-col>
      <b-col cols="auto" class="mr-4"></b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'
import Link from '@/icons/LinkIcon.vue'

@Component({
  components: {
    Link
  },
  emits: ['update']
})
export default class InputGroup extends Vue {
  @Prop()
  hint!: string

  onInputChange(val: string) {
    this.$emit('update', val)
  }
}
</script>

<style lang="sass" scoped>
.title
  font-size: 26px

.background
  align-content: flex-start
  background-color: var(--tertiary)
  height: 41px
  overflow: hidden

.item-text
  font-size: 18px
  color: var(--textSecondary)
  min-width: 0
  text-align: left

.ext-input
  font-size: 16px
  max-width: 100%
  margin-bottom: 15px !important
  color: var(--textPrimary)
  background-color: transparent
  border: 0
  border-bottom: 1px solid transparent
  border-radius: 0
  padding: 0
  &::placeholder
    color: var(--textSecondary)
</style>
