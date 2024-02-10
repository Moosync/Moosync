/*
 *  main.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { registerLogger } from '@/utils/ui/common'
import '@/mainWindow/plugins/contextMenu'
import { i18n } from '@/mainWindow/plugins/i18n'
import '@/mainWindow/plugins/inlineSVG'
import '@/mainWindow/plugins/recycleScroller'
import '@/mainWindow/plugins/tags-typeahead'
import '@/mainWindow/plugins/vueBootstrap'
import '@/mainWindow/plugins/vueSliderBar'
import '@/sass/global.sass'
import 'animate.css'
import Vue3Toastify, { type ToastContainerOptions } from 'vue3-toastify'
import 'vue3-toastify/dist/index.css'

import App from '@/mainWindow/App.vue'
import { router } from '@/mainWindow/plugins/router'
import { store } from '@/mainWindow/store'
import ContextMenu from '@imengyu/vue3-context-menu'
import EventEmitter from 'events'
import { configureCompat, createApp } from 'vue'



export const bus = new EventEmitter()

const app = createApp(App)
app.use(i18n)
app.use(router)
app.use(store)
app.use(ContextMenu)
app.use<ToastContainerOptions>(Vue3Toastify, {
  autoClose: 3000,
})

function isImage(e: HTMLElement) {
  const tagName = e.tagName.toLowerCase()
  const parentTagName = e.parentElement?.tagName.toLowerCase()
  return tagName === 'img' || tagName === 'svg' || parentTagName === 'svg'
}

app.directive('click-outside', {
  mounted(el, binding) {
    el.clickOutsideEvent = (e: Event) => {
      if (el !== e.target && !el.contains(e.target) && !isImage(e.target as HTMLElement)) {
        binding.value(e, el)
      }
    }
    document.body.addEventListener('click', el.clickOutsideEvent)
  },
  unmounted(el) {
    document.body.removeEventListener('click', el.clickOutsideEvent)
  },
})

registerLogger(app)

configureCompat({})

app.mount('#app')
