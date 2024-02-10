/*
 *  ModelHelper.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component, Vue } from 'vue-facing-decorator'

@Component
export default class ModelHelper extends Vue {
  public isAlbumExists(song: Song | null | undefined) {
    return song?.album
  }

  public isArtistExists(song: Song | null | undefined) {
    return song?.artists
  }

  public getAlbumName(song: Song | null | undefined): string {
    return this.isAlbumExists(song)?.album_name ?? ''
  }

  public getArtists(song: Song | null | undefined) {
    return this.isArtistExists(song)?.join(', ') ?? '-'
  }
}
