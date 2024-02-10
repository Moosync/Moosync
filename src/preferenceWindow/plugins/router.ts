/*
 *  router.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import Keybinds from '../components/pages/Keybinds.vue'
import Logs from '../components/pages/Logs.vue'
import NewTheme from '../components/pages/NewTheme.vue'
import System from '../components/pages/System.vue'
import Themes from '../components/pages/Themes.vue'
import Extensions from '@/preferenceWindow/components/pages/Extensions.vue'
import Paths from '@/preferenceWindow/components/pages/Paths.vue'
import { RouteRecordRaw, createRouter, createWebHashHistory, createWebHistory } from 'vue-router'

const routes: RouteRecordRaw[] = [
  {
    path: '/',
    redirect: { name: 'paths' },
  },
  {
    name: 'paths',
    path: '/paths',
    component: Paths,
  },
  {
    name: 'extensions',
    path: '/extensions',
    component: Extensions,
  },
  {
    name: 'system',
    path: '/system',
    component: System,
  },
  {
    name: 'keybinds',
    path: '/keybinds',
    component: Keybinds,
  },
  {
    name: 'themes',
    path: '/themes',
    component: Themes,
  },
  {
    name: 'new_theme',
    path: '/themes/new',
    component: NewTheme,
  },
  {
    name: 'logs',
    path: '/logs',
    component: Logs,
  },
]

export const router = createRouter({
  history: process.env.IS_ELECTRON
    ? createWebHashHistory(process.env.BASE_URL)
    : createWebHistory(process.env.BASE_URL),
  routes,
})
