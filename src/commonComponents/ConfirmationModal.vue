<!-- 
  ConfirmationModal.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal body-class="delete-modal" centered size="lg" :id="id" :ref="id" hide-footer hide-header>
    <div class="modal-content-container">
      <b-container fluid class="p-0">
        <b-row no-gutters class="d-flex">
          <b-col>
            <h4>
              Are you sure you want to <span class="keyword">{{ keyword }}</span>
              <span class="item"> {{ itemName }}</span
              >?
            </h4>
            <h6 class="mt-3">Press Confirm if you are sure</h6>
          </b-col>
        </b-row>
        <b-row cols="auto" class="mt-3">
          <b-button class="cancel-button ml-auto" @click="onCancelPressed">Cancel</b-button>
          <b-button class="confirm-button ml-3" @click="onConfirmPressed">Confirm</b-button>
        </b-row>
      </b-container>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'

@Component({
  emits: ['cancel', 'confirm']
})
export default class ConfirmationModal extends Vue {
  @Prop({ default: 'DeleteDialog' })
  id!: string

  @Prop({ default: '' })
  itemName!: string

  @Prop({ default: '' })
  description!: string

  @Prop({ default: 'delete' })
  keyword!: string

  onCancelPressed() {
    this.$emit('cancel')
    this.$bvModal.hide(this.id)
  }

  onConfirmPressed() {
    this.$emit('confirm')
    this.$bvModal.hide(this.id)
  }
}
</script>

<style lang="sass" scoped>
.topbar-container
  background: var(--primary)
  height: 70px
  padding-left: calc(261px + 30px + 7.5px)

.confirm-button, .cancel-button
  font-size: 16px
  border-radius: 6px
  margin-bottom: 20px
  margin-top: 15px
  border: 0
  font-weight: 600
  box-shadow: 4px 3px 7px 0px rgb(0 0 0 / 25%)

.confirm-button
  background-color: #E62017
  color: #fff

.cancel-button
  background-color: var(--tertiary)
  color: var(--textPrimary)

.keyword
  font-weight: 700
  color: #E62017

.item
  font-weight: 700
</style>
