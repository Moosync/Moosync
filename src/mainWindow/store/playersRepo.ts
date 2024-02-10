/*
 *  players.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { VuexModule } from './module'
import { Player } from '@/utils/ui/players/player'
import { mutation } from 'vuex-class-component'

export class PlayerRepositoryStore extends VuexModule.With({ namespaced: 'playerRepository' }) {
  private players: Player[] = []

  get allPlayers() {
    return this.players
  }

  @mutation
  public clear() {
    this.players.splice(0, this.players.length)
  }

  @mutation
  public push(player: Player[]) {
    for (const p of player) {
      if (!this.players.some((val) => val.key === p.key)) {
        this.players.push(p)
      }
    }
  }
}
