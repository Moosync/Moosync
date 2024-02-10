/*
 *  remoteSongMixin.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { vxm } from '@/mainWindow/store'
import { mergeDeep } from '@/utils/common'
import { Component, Vue } from 'vue-facing-decorator'
import { toast } from 'vue3-toastify'
import { convertProxy } from '../common'

@Component
export default class RemoteSong extends Vue {
  public async addSongsToLibrary(...songs: Song[]) {
    const storedSongs = await window.DBUtils.storeSongs(convertProxy(songs))
    this.fetchCoverDetails(...storedSongs)
    toast(`Added ${songs.length} songs to library`)
  }

  private async fetchCoverDetails(...songs: (Song | undefined)[]) {
    for (const s of songs) {
      if (s?.artists) {
        for (const a of s.artists) {
          if (!a.artist_coverPath) {
            const fetchedArtist = await this.fetchRemoteArtistDetails(a)
            await window.DBUtils.updateArtist({
              ...a,
              artist_coverPath: fetchedArtist?.artist_coverPath,
              artist_extra_info: mergeDeep(fetchedArtist?.artist_extra_info ?? {}, a.artist_extra_info),
            })
          }
        }
      }
    }
  }

  public async fetchRemoteArtistDetails(a: Artists) {
    let fetchedArtist: Artists | undefined

    if (a.artist_extra_info?.youtube?.channel_id) {
      fetchedArtist = await vxm.providers.youtubeProvider.getArtistDetails(a)
    } else if (a.artist_extra_info?.spotify?.artist_id) {
      fetchedArtist = await vxm.providers.spotifyProvider.getArtistDetails(a)
    } else {
      fetchedArtist = await vxm.providers.spotifyProvider.getArtistDetails(a, true)
    }

    return fetchedArtist
  }
}
