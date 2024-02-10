/*
 *  errorHandler.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component, Vue } from 'vue-facing-decorator'

@Component
export default class ErrorHandler extends Vue {
  handlerImageError(err: ErrorEvent, callback?: (err: ErrorEvent) => void) {
    callback?.(err)
  }
}
