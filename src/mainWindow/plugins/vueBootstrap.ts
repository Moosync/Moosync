/*
 *  vueBootstrap.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import {
  ButtonPlugin,
  DropdownPlugin,
  FormCheckboxPlugin,
  FormInputPlugin,
  FormTagsPlugin,
  FormTextareaPlugin,
  ImagePlugin,
  LayoutPlugin,
  LinkPlugin,
  ModalPlugin,
  PopoverPlugin,
  SidebarPlugin,
  SpinnerPlugin,
  TabsPlugin,
} from 'bootstrap-vue'
import Vue from 'vue'

Vue.use(ModalPlugin)
Vue.use(LayoutPlugin)
Vue.use(ButtonPlugin)
Vue.use(SidebarPlugin)
Vue.use(PopoverPlugin)
Vue.use(TabsPlugin)
Vue.use(ImagePlugin)
Vue.use(FormInputPlugin)
Vue.use(FormTextareaPlugin)
Vue.use(LinkPlugin)
Vue.use(SpinnerPlugin)
Vue.use(FormTagsPlugin)
Vue.use(FormCheckboxPlugin)
Vue.use(DropdownPlugin)
