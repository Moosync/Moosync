/*
 *  AccountsMixin.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { vxm } from '@/mainWindow/store'
import { Component, Vue } from 'vue-facing-decorator'

@Component
export default class JukeboxMixin extends Vue {
  public get isJukeboxModeActive() {
    return vxm.themes.jukeboxMode
  }

  private isJukeboxFieldActive(fieldName: string) {
    return vxm.themes.jukeboxOptionalFields.find((val) => val.key === `jukebox_${fieldName}`)?.enabled ?? false
  }

  public get isSkipEnabled() {
    return !this.isJukeboxModeActive || this.isJukeboxFieldActive('skip')
  }

  public get isShuffleEnabled() {
    return !this.isJukeboxModeActive || this.isJukeboxFieldActive('shuffle')
  }

  public get isRepeatEnabled() {
    return !this.isJukeboxModeActive || this.isJukeboxFieldActive('repeat')
  }
}
