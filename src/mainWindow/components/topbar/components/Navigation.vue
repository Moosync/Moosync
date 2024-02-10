<!-- 
  Navigation.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-row class="justify-content-between">
    <b-col cols="6" @click="back">
      <NavBack />
    </b-col>
    <b-col cols="6" @click="forward">
      <NavForward />
    </b-col>
  </b-row>
</template>

<script lang="ts">
import { Component, Vue } from 'vue-facing-decorator'
import NavBack from '@/icons/NavBackIcon.vue'
import NavForward from '@/icons/NavForwardIcon.vue'

@Component({
  components: {
    NavBack,
    NavForward
  }
})
export default class Notifications extends Vue {
  forward() {
    this.$router.forward()
  }

  back() {
    this.$router.back()
  }
}
</script>
