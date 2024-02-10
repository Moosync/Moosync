<!-- 
  SetupModal.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal
    class="setup-modal"
    no-close-on-backdrop
    centered
    :size="getWidth()"
    :id="id"
    :ref="id"
    hide-footer
    hide-header
  >
    <div class="modal-content-container">
      <transition name="fade" mode="out-in">
        <component
          key="comp"
          :is="
            state === SetupModalState.WELCOME
              ? 'Welcome'
              : state === SetupModalState.PATHS
              ? 'PathSetup'
              : 'AccountsSetup'
          "
          @next="nextState"
          @prev="prevState"
        />
        <!-- <Welcome key="welcome" v-if="state === SetupModalState.WELCOME" @next="nextState" @prev="close" />
        <PathSetup key="paths" v-if="state === SetupModalState.PATHS" @next="nextState" @prev="prevState" />
        <AccountsSetup key="accounts" v-if="state === SetupModalState.ACCOUNTS" @next="close" @prev="prevState" /> -->
      </transition>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import Welcome from './Welcome.vue'
import PathSetup from './PathSetup.vue'
import AccountsSetup from './AccountsSetup.vue'

enum SetupModalStates {
  WELCOME,
  PATHS,
  ACCOUNTS
}

@Component({
  components: {
    Welcome,
    PathSetup,
    AccountsSetup
  }
})
export default class SetupModal extends Vue {
  state: SetupModalStates = SetupModalStates.WELCOME
  SetupModalState = SetupModalStates

  @Prop({ default: 'SetupModal' })
  id!: string

  close() {
    this.$bvModal.hide(this.id)
  }

  nextState() {
    if (this.state < SetupModalStates.ACCOUNTS) this.state += 1
    else this.close()
  }

  prevState() {
    if (this.state > SetupModalStates.WELCOME) this.state -= 1
  }

  getWidth() {
    switch (this.state) {
      case SetupModalStates.WELCOME:
        return 'sm'
      case SetupModalStates.PATHS:
        return 'lg'
      case SetupModalStates.ACCOUNTS:
        return 'md'
    }
  }

  mounted() {
    bus.on(EventBus.SHOW_SETUP_MODAL, () => {
      this.$bvModal.show(this.id)
    })
  }
}
</script>

<style lang="sass" scoped>
.modal-content-container
  user-select: none
</style>
