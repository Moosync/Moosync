/*
 *  recycleScroller.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import 'vue-virtual-scroller/dist/vue-virtual-scroller.css'

import { RecycleScroller } from 'vue-virtual-scroller'
import Vue from 'vue'

// rome-ignore lint/suspicious/noExplicitAny: <explanation>
Vue.component('RecycleScroller', RecycleScroller as any)
