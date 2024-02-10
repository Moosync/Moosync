<!-- 
  System.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="w-100 h-100">
    <HotkeyGroup
      class="mt-5 mb-3"
      key="hotkeys"
      :isExtension="false"
      title="Hotkeys"
      tooltip="Hotkeys to easily perform actions using button presses"
      :defaultValue="defaultKeybinds"
    />
  </div>
</template>

<script lang="ts">
import { Component } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'
import HotkeyGroup from '../HotkeyGroup.vue'
import { defaultKeybinds } from '@/utils/commonConstants'

@Component({
  components: {
    HotkeyGroup
  }
})
export default class Keybinds extends Vue {
  defaultKeybinds = defaultKeybinds
}
</script>

<style lang="sass" scoped></style>
