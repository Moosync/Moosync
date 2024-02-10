/*
 *  main.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import '@/preferenceWindow/plugins/recycleScroller'
import '@/preferenceWindow/plugins/vueBootstrap'
import '@/preferenceWindow/plugins/contextmenu'
import '@/sass/global.sass'
import 'animate.css'
import 'vue3-colorpicker/style.css'

import App from './Preferences.vue'
import ContextMenu from '@imengyu/vue3-context-menu'
import EventEmitter from 'events'
import Vue3ColorPicker from 'vue3-colorpicker'
import { createApp } from 'vue'
import { i18n } from '@/preferenceWindow/plugins/i18n'
import { registerLogger } from '@/utils/ui/common'
import { router } from '@/preferenceWindow/plugins/router'

export const bus = new EventEmitter()

const app = createApp(App)
app.use(i18n)
app.use(router)
app.use(ContextMenu)
app.use(Vue3ColorPicker)

app.directive('click-outside', {
  mounted(el, binding) {
    el.clickOutsideEvent = function (e: Event) {
      if (el !== e.target && !el.contains(e.target)) {
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

app.mount('#app')
