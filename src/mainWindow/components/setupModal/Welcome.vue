<!-- 
  Welcome.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container fluid class="p-0">
    <b-row>
      <b-col class="d-flex justify-content-center"> <Logo class="logo" /> </b-col>
    </b-row>
    <b-row>
      <b-col class="welcome-text"> Welcome to </b-col>
    </b-row>
    <b-row>
      <b-col class="moosync-text"> Moosync </b-col>
    </b-row>
    <b-row>
      <b-col class="d-flex justify-content-center">
        <div @click="beginSetup" class="start-button button-grow mt-4 d-flex justify-content-center align-items-center">
          Begin Setup
        </div>
      </b-col>
    </b-row>
    <b-row>
      <b-col class="d-flex justify-content-center">
        <div @click="skip" id="skip-setup" class="skip-button mt-2 d-flex justify-content-center align-items-center">
          Skip
        </div>
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Vue } from 'vue-facing-decorator'
import { Component } from 'vue-facing-decorator'
import Logo from '@/icons/LogoIcon.vue'

@Component({
  components: {
    Logo
  },
  emits: ['next', 'prev']
})
export default class Welcome extends Vue {
  beginSetup() {
    this.$emit('next')
  }

  skip() {
    this.$emit('prev')
  }
}
</script>

<style lang="sass" scoped>
.welcome-text
  font-size: 16px
  text-align: center

.moosync-text
  font-family: Poppins
  font-style: normal
  font-weight: 600
  font-size: 42px
  text-align: center
  margin-top: -8px

.logo
  width: 100px
  height: 100px
  margin-bottom: 16px

.start-button
  width: 135px
  height: 36px
  border: 1px solid var(--accent)
  border-radius: 6px
  font-size: 16px

.skip-button
  font-size: 16px
  color: var(--textSecondary)
  cursor: pointer
</style>
