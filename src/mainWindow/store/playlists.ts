/*
 *  playlists.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { VuexModule } from './module'

export class PlaylistStore extends VuexModule.With({ namespaced: 'playlist' }) {
  public playlists: playlistInfo = {}
  public updated = false
}
