<!-- 
  PinEntryModal.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal centered size="md" :id="id" :ref="id" hide-footer hide-header>
    <b-container fluid>
      <b-row no-gutters align-h="center" align-v="center" class="pin-input-container">
        <b-col :cols="12 / pinLength" v-for="i of pinLength" :key="i">
          <input
            class="pin-input"
            :ref="`pin-input-${i}`"
            @input="validateInputAndPush(i, $event as InputEvent)"
            @click="onInputClick(i, $event as PointerEvent)"
          />
        </b-col>
      </b-row>

      <b-row no-gutters class="mt-3 error-status" v-if="showError">
        <b-col>Invalid pin</b-col>
      </b-row>

      <b-row no-gutters class="num-row mt-3">
        <b-col class="num-col" cols="4" v-for="num in 9" :key="num">
          <div class="inner-div w-100 h-100 d-flex align-items-center" @click="numClick(num)">
            <div class="num ml-auto mr-auto">{{ num }}</div>
          </div>
        </b-col>

        <b-col class="num-col" cols="4">
          <div class="inner-div w-100 h-100 d-flex align-items-center" @click="cancel">
            <div class="num ml-auto mr-auto"><CrossIcon :color="'#F02121'" class="icons" /></div>
          </div>
        </b-col>

        <b-col class="num-col" cols="4">
          <div class="inner-div w-100 h-100 d-flex align-items-center" @click="numClick(0)">
            <div class="num ml-auto mr-auto">{{ 0 }}</div>
          </div>
        </b-col>

        <b-col class="num-col" cols="4">
          <div class="inner-div w-100 h-100 d-flex align-items-center" @click="confirm">
            <div class="num ml-auto mr-auto"><TickIcon class="icons" :color="'#01D716'" /></div>
          </div>
        </b-col>
      </b-row>
    </b-container>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Watch } from 'vue-facing-decorator'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import CrossIcon from '@/icons/CrossIcon.vue'
import { Vue } from 'vue-facing-decorator'
import TickIcon from '@/icons/TickIcon.vue'

@Component({
  components: {
    CrossIcon,
    TickIcon
  }
})
export default class PinEntryModal extends Vue {
  @Prop({ default: 'PinEntry' })
  id!: string

  private pin: (number | null)[] = []
  private currentIndex = 1

  pinLength = 4

  showError = false

  private confirmCallback?: (pin: string) => boolean

  @Watch('currentIndex')
  private onCurrentIndexChange(newVal: number) {
    const elem = (this.$refs[`pin-input-${newVal}`] as HTMLInputElement[])?.at(0)
    elem?.focus()
  }

  onInputClick(index: number, ev: PointerEvent) {
    ev.preventDefault()
    const elem = ev.target as HTMLInputElement
    elem.setSelectionRange(1, 1)
    this.currentIndex = index
  }

  mounted() {
    bus.on(EventBus.SHOW_PIN_ENTRY_MODAL, async (pinLength: number, confirmCallback: (pin: string) => boolean) => {
      this.confirmCallback = confirmCallback
      this.pinLength = pinLength ?? 0
      this.showError = false
      this.$bvModal.show(this.id)
      this.currentIndex = 1

      await this.$nextTick()
      const elem = this.getElement(this.currentIndex)
      await this.$nextTick()
      elem?.focus()
    })
  }

  private moveToNextIndex(index: number = this.currentIndex) {
    const nextIndex = index + 1
    if (this.currentIndex === this.pinLength) {
      this.confirm()
    }

    if (nextIndex <= this.pinLength) {
      this.currentIndex = nextIndex
    } else {
      this.currentIndex = 1
    }
  }

  validateInputAndPush(index: number, event: InputEvent) {
    const input = (event.target as HTMLInputElement).value
    if (input) {
      const num = parseInt(input[input.length - 1])
      if (!isNaN(num)) {
        this.pin[index - 1] = num
        this.moveToNextIndex(index)
      }
    } else {
      this.pin[index - 1] = null
    }

    ;(event.target as HTMLInputElement).value = this.pin[index - 1]?.toString() ?? ''
  }

  numClick(num: number) {
    this.pin.splice(this.currentIndex - 1, 1, num)
    const elem = this.getElement(this.currentIndex)
    if (elem) {
      elem.value = num.toString()
      this.moveToNextIndex()
    }
  }

  cancel() {
    this.confirmCallback = undefined
    this.pin = []

    this.$bvModal.hide(this.id)
  }

  private clearInput() {
    for (const [key, elem] of Object.entries(this.$refs as Record<string, HTMLInputElement[]>)) {
      if (key.startsWith('pin-input')) {
        elem[0].value = ''
      }
    }

    this.pin.splice(0, this.pin.length)
  }

  private getElement(index: number) {
    return (this.$refs[`pin-input-${index}`] as HTMLInputElement[])?.at(0)
  }

  confirm() {
    if (this.confirmCallback) {
      if (this.confirmCallback(this.pin.join(''))) {
        this.cancel()
      } else {
        this.showError = true
        this.clearInput()
      }

      this.getElement(1)?.focus()
    }
  }
}
</script>

<style lang="sass" scoped>
.pin-input-container
  width: calc( 100% - 10px )
  height: 50px
  margin-left: auto
  margin-right: auto
  background: var(--tertiary)
  padding: 0px 15px 0 15px
  border-radius: 10px

.pin-input
  font-size: 26px
  max-width: 100%
  color: var(--textPrimary)
  background-color: transparent
  border: 0
  border-bottom: 1px solid transparent
  border-radius: 0
  padding: 0
  text-align: center
  transition: all 0.3s linear
  &:focus
    border-bottom: 1px solid var(--accent)
    outline: none
    -webkit-box-shadow: none
  &::selection
    background: transparent
    color: transparent

.num-col
  height: 80px
  padding: 8px

.inner-div
  background: var(--tertiary)
  border-radius: 16px
  cursor: pointer

.num
  font-size: 26px
  font-weight: 700
  height: fit-content
  width: fit-content
  user-select: none

.icons
  width: 26px

.error-status
  color: #F02121
  margin-left: auto
  margin-right: auto
  width: fit-content
</style>
