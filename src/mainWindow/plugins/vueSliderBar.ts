/*
 *  vueSliderBar.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import 'vue-slider-component/theme/default.css'

import Vue from 'vue'
import VueSlider from 'vue-slider-component'

VueSlider.compatConfig = {
  ...VueSlider.compatConfig,
  MODE: 3,
}

Vue.component('VueSlider', VueSlider)
