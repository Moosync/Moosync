/*
 *  FileMixin.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component, Vue } from 'vue-facing-decorator'

@Component
export default class FileMixin extends Vue {
  dragFile(event: DragEvent) {
    event.preventDefault()
    window.WindowUtils.dragFile((event.target as HTMLImageElement)?.src)
  }
}
