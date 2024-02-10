<!-- 
  PathSetup.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="p-0">
    <b-row>
      <b-col class="title-text mb-4"> Let's start by setting the location to your Music Library </b-col>
    </b-row>
    <b-row no-gutters>
      <b-col>
        <DirectoryGroup
          :bottomButton="true"
          :enableCheckbox="false"
          :height="4"
          :defaultValue="[]"
          :isMainWindow="true"
          key="musicPaths"
        />
      </b-col>
    </b-row>
    <b-row no-gutters align-h="end" class="button-row">
      <b-col cols="auto" class="mr-3">
        <b-button @click="skip" class="close-button">Prev</b-button>
      </b-col>
      <b-col cols="auto">
        <b-button @click="next" class="create-button">Next</b-button>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component } from 'vue-facing-decorator'
import Logo from '@/icons/LogoIcon.vue'
import DirectoryGroup from '@/preferenceWindow/components/DirectoryGroup.vue'

@Component({
  components: {
    Logo,
    DirectoryGroup
  },
  emits: ['next', 'prev']
})
export default class PathSetup extends Vue {
  next() {
    this.$emit('next')
  }

  skip() {
    this.$emit('prev')
  }
}
</script>

<style lang="sass">
.modal-dialog
  transition: max-width 0.4s cubic-bezier(0.71, -0.03, 0.32, 1.01), max-height 0.4s cubic-bezier(0.71, -0.03, 0.32, 1.01), transform 0.3s ease-out !important
</style>

<style lang="sass" scoped>
.title-text
  margin-left: 15px
  font-size: 16px

.create-button, .close-button
  font-size: 16px
  font-weight: 400
  color: var(--textInverse)
  background-color: var(--accent)
  border-radius: 6px
  margin-bottom: 8px
  padding: 6px 20px 6px 20px
  margin-top: 15px
  border: 0

.close-button
  background-color: var(--textSecondary)
  color: var(--textPrimary)

.button-row
  margin-right: 15px
</style>
