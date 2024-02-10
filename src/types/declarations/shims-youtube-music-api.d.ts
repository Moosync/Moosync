/* eslint-disable @typescript-eslint/no-explicit-any */
/*
 *  shims-youtube-music-api.d.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

declare module 'youtube-music-api' {
  interface Thumbnails {
    height: number
    width: number
    url: string
  }

  interface Queryable {
    browseId: string
    name: string
  }

  interface SearchResult {
    content: {
      type: string
      videoId: string
      playlistId: string
      name: string
      artist: Queryable
      album: Queryable
      duration: number
      thumbnails: Thumbnails[]
      params: string
    }[]
    continuation: {
      clickTrackingParams: string
      continuation: string
    }
  }
  export default class YoutubeMusicApi {
    // constructor()
    initalize(): Promise<{ locale: string; logged_in: string }>
    getSearchSuggestions(query: string): string[]
    search(query: string, categoryName: string, _pageLimit?: number): Promise<SearchResult>

    //TODO: Detailed definitions
    getAlbum(browseId: string): unknown
    getPlaylist(browseId: string, contentLimit?: number): unknown
    getArtist(browseId: string)
    getNext(videoId: string, playlistId: string, paramString: string)
  }
}
