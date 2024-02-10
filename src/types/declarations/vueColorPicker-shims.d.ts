/*
 *  vueColorPicker-shims.d.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

declare module '@caohenghu/vue-colorpicker' {}

type ColorPickerOutput = {
  colors: {
    hex: string
    rgb: {
      r: number
      g: number
      b: number
      a: number
    }
  }
}
