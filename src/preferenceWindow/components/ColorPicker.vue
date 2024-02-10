<!-- 
  ColorPicker.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <tr height="60">
    <td class="color-title pr-5" :title="title">
      {{ title }}
    </td>
    <td class="pr-4" ref="parent">
      <div class="color-box" :id="id" @click="toggleColorPicker" :style="{ background: color }"></div>
    </td>
    <td>
      <!-- <color-picker v-if="showColorPicker" :style="{ left: `${pickerPosition[0]}px`, top: `${pickerPosition[1]}px` }"
        class="color-picker" theme="dark" :color="color" :sucker-hide="false" :sucker-area="[]"
        @changeColor="changeColor" /> -->

      <!-- <color-picker /> -->
      <ColorPicker v-if="showColorPicker" v-click-outside="hideColorPicker" :color="defColor"
        :style="{ left: `${pickerPosition[0]}px`, top: `${pickerPosition[1]}px` }" @color-change="changeColor" />
    </td>
  </tr>
</template>

<script lang="ts">
import { Component, Prop, Watch } from 'vue-facing-decorator'
import { Vue } from 'vue-facing-decorator'
import { ColorPicker } from 'vue-accessible-color-picker'
import { v1 } from 'uuid';


@Component({
  components: {
    ColorPicker
  },
  emits: ['colorChange'],
  options: {
    compatConfig: {
      MODE: 1
    }
  }
})
export default class ColorPickerr extends Vue {
  @Prop({ default: 'Primary' })
  title!: string

  showColorPicker = false
  pickerPosition = [0, 0]

  @Prop({ default: '#ffffff' })
  defColor!: string

  color = ''

  id = v1()

  testVal = {}

  hideColorPicker(event: PointerEvent) {
    if ((event.target as HTMLDivElement)?.id !== this.id) {
      this.showColorPicker = false
    }
  }

  @Watch('defColor') onDefaultChange() {
    this.color = this.defColor
  }

  public toggleColorPicker(mouseEvent?: MouseEvent) {
    const parent = this.$refs['parent'] as HTMLDivElement
    this.pickerPosition = [parent.offsetLeft, parent.offsetTop + 40]
    if (mouseEvent) {
      this.pickerPosition[0] += mouseEvent.offsetX
      this.pickerPosition[1] += mouseEvent.offsetY
    } else {
      this.pickerPosition[0] += 30
      this.pickerPosition[1] += 15
    }

    this.showColorPicker = !this.showColorPicker
  }

  private parseChannel(val: number) {
    if (val)
      return (val * 255).toFixed(2)
    return 0
  }

  private RGBAToString(color: ColorPickerOutput['colors']['rgb']) {
    return `rgba(${this.parseChannel(color.r)}, ${this.parseChannel(color.g)}, ${this.parseChannel(color.b)}, ${this.parseChannel(color.a)})`
  }

  changeColor(color: ColorPickerOutput) {
    const parsed = this.RGBAToString(color.colors.rgb)

    if (parsed !== this.color) {
      this.color = parsed
      this.$emit('colorChange', this.color)
    }
  }

  created() {
    this.color = this.defColor
  }
}
</script>

<style lang="sass" scoped>
.color-box
  width: 90px
  height: 40px
  border-radius: 4px
  border: 0.61px solid var(--textPrimary)

.color-title
  text-align: left

.icon
  width: 20px
  height: 20px
  margin-top: -8px

.vacp-color-picker
  position: absolute
  z-index: 999
  width: 100%
</style>
