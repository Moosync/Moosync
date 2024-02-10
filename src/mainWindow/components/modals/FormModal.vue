<!-- 
  FormModal.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal no-close-on-backdrop centered size="sm" :id="id" :ref="id" hide-footer hide-header>
    <b-container>
      <b-row no-gutters class="title">
        {{ title }}
      </b-row>
      <b-row no-gutters>
        <b-input v-model="value" type="number" class="input-field"></b-input>
      </b-row>
      <b-row no-gutters align-h="end" class="button-row">
        <b-col cols="auto" class="mr-3">
          <b-button @click="close" class="close-button">Cancel</b-button>
        </b-col>
        <b-col cols="auto">
          <b-button @click="setValue" class="create-button">Set</b-button>
        </b-col>
      </b-row>
    </b-container>
  </b-modal>
</template>

<script lang="ts">
import { EventBus } from '@/utils/preload/ipc/constants'
import { Component, Prop, Vue } from 'vue-facing-decorator'
import { bus } from '@/mainWindow/main'

@Component({})
export default class FormModal extends Vue {
  @Prop({ default: 'FormModal' })
  id!: string

  title = ''
  private callback?: (val: number) => void
  value = 0

  private showing = false

  mounted() {
    bus.on(EventBus.SHOW_FORM_MODAL, (title: string, callback: (val: number) => void) => {
      if (!this.showing) {
        this.title = title
        this.callback = callback
        this.$bvModal.show(this.id)
        this.showing = true
      }
    })
  }

  close() {
    this.$bvModal.hide(this.id)
    this.showing = false
  }

  setValue() {
    this.callback && this.callback(this.value)
    this.close()
  }
}
</script>

<style lang="sass" scoped>
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

.title
  font-size: 22px
  margin-bottom: 10px

.input-field
  font-size: 16px
  max-width: 100%
  margin-bottom: 15px !important
  color: var(--textPrimary)
  background-color: var(--tertiary)
  border: 0
  border-bottom: 1px solid transparent
  border-radius: 0
  padding: 5px
  border-bottom: 1px solid var(--accent)
  &:focus
    outline: none
    -webkit-box-shadow: none
</style>
