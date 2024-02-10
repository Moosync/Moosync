/*
 *  youtubeResponses.d.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

declare namespace LastFMResponses {
  // These are only the resources used
  enum ApiResources {
    GET_SESSION = 'auth.getSession',
    UPDATE_NOW_PLAYING = 'track.updateNowPlaying',
    SCROBBLE = 'track.scrobble',
    GET_USER_INFO = 'user.getInfo',
    GET_TRACK_INFO = 'track.getInfo',
  }

  interface SessionInfo {
    session?: {
      key: string
    }
  }

  interface UserInfo {
    user?: {
      name: string
    }
  }

  interface TrackInfo {
    track: {
      name: string
      artist: {
        name: string
        mbid: string
      }
      album?: {
        title: string
        artist: string
        image: {
          '#text': string
        }[]
      }
    }
  }

  interface ScrapeResponse {
    playlist: {
      name: string
      artists: {
        name: string
      }[]
      duration: number
      playlinks: {
        affiliate: 'youtube' | 'spotify'
        id: string
      }[]
    }[]
  }

  type ResponseType<T extends ApiResources> = T extends ApiResources.GET_SESSION
    ? SessionInfo
    : T extends ApiResources.UPDATE_NOW_PLAYING
    ? undefined
    : T extends ApiResources.SCROBBLE
    ? undefined
    : T extends ApiResources.GET_USER_INFO
    ? UserInfo
    : T extends ApiResources.GET_TRACK_INFO
    ? TrackInfo
    : undefined
}
