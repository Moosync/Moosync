<!-- 
  EditText.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container v-if="prefKey" fluid class="path-container w-100">
    <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" @tooltipClick="emitTooltipClick" />
    <b-row no-gutters class="background w-100 mt-2 d-flex">
      <b-row no-gutters class="m-1 item w-100">
        <b-col cols="auto" align-self="center" class="w-100 h-100">
          <iframe :title="title" ref="md-iframe" class="w-100 h-100 iframe"> </iframe>
        </b-col>
      </b-row>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, Prop, Watch } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import PreferenceHeader from './PreferenceHeader.vue'
import { ExtensionPreferenceMixin } from '../mixins/extensionPreferenceMixin'
import MarkdownIt from 'markdown-it'

@Component({
  components: {
    PreferenceHeader
  },
  emits: ['tooltipClick']
})
export default class TextField extends mixins(ExtensionPreferenceMixin) {
  @Prop()
  title!: string

  @Prop()
  tooltip!: string

  private render = ''

  emitTooltipClick() {
    this.$emit('tooltipClick')
  }

  private async generateCSSRules() {
    const colors = await window.ThemeUtils.getActiveTheme()
    const iframeStyle = document.createElement('style')
    const styleSheets = Array.from(document.styleSheets)

    let style = ''

    styleSheets.forEach((styleSheet) => {
      const cssRules = styleSheet.cssRules

      for (let i = 0; i < cssRules.length; i++) {
        const rule = cssRules.item(i)
        if (rule?.cssText.startsWith('@font-face')) {
          style += rule.cssText
        }
      }
    })
    style += `
    *::-webkit-scrollbar,
*::-webkit-scrollbar-thumb {
  width: 26px;
  border-radius: 13px;
  background-clip: padding-box;
  border: 10px solid transparent;
  min-height: 50px;
  }

*::-webkit-scrollbar-thumb {
  box-shadow: inset 0 0 0 10px;
  min-height: 40px;
  }

*::-webkit-scrollbar-track {
  background: transparent;
  }
  

  * {
    font-family: 'Nunito Sans';
    color: ${colors.theme.textPrimary};
  }
    `

    iframeStyle.appendChild(document.createTextNode(style))

    return iframeStyle.outerHTML
  }

  @Watch('render')
  private async onRenderChange(val: string) {
    const doc = (this.$refs['md-iframe'] as HTMLIFrameElement)?.contentDocument?.body
    if (doc) {
      doc.innerHTML = val
      if (doc.scrollTop >= doc.scrollHeight - (20 / 100) * doc.scrollHeight) {
        doc.scrollTo({
          top: doc.scrollHeight,
          behavior: 'smooth'
        })
      }
    }
  }

  async mounted() {
    const data = `<html lang="en"><head>${await this.generateCSSRules()}</head><body></body></html>`
    const doc = (this.$refs['md-iframe'] as HTMLIFrameElement)?.contentDocument
    doc?.open('text/html', 'replace')
    doc?.write(data)
    doc?.close()

    this.onRenderChange(this.render)
  }

  @Watch('value')
  private onValueChanged(value?: string) {
    const md = new MarkdownIt('default', {
      html: true
    })
    this.render = md.render(value ?? '')
  }
}
</script>

<style lang="sass" scoped>
.title
  font-size: 26px

.item
  height: calc(100% - 1rem)

.background
  align-content: flex-start
  background-color: var(--tertiary)
  height: 200px

.iframe
  border: none !important
</style>
