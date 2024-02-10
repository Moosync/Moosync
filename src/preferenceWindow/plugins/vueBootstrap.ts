/*
 *  vueBootstrap.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import {
  ButtonPlugin,
  DropdownPlugin,
  FormCheckboxPlugin,
  FormInputPlugin,
  InputGroupPlugin,
  LayoutPlugin,
  ModalPlugin,
  PaginationPlugin,
  PopoverPlugin,
  ProgressPlugin,
  SidebarPlugin,
  TablePlugin,
  TooltipPlugin,
} from 'bootstrap-vue'

import Vue from 'vue'

Vue.use(LayoutPlugin)
Vue.use(SidebarPlugin)
Vue.use(TooltipPlugin)
Vue.use(FormCheckboxPlugin)
Vue.use(ModalPlugin)
Vue.use(FormInputPlugin)
Vue.use(ButtonPlugin)
Vue.use(ProgressPlugin)
Vue.use(TablePlugin)
Vue.use(DropdownPlugin)
Vue.use(PaginationPlugin)
Vue.use(InputGroupPlugin)
Vue.use(PopoverPlugin)
