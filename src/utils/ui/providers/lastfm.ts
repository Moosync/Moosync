/*
 *  lastfm.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { bus } from '@/mainWindow/main'
import { ProviderScopes } from '@/utils/commonConstants'
import { EventBus } from '@/utils/preload/ipc/constants'
import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider'
import { md5 } from 'hash-wasm'
import { FetchWrapper } from './generics/fetchWrapper'

const AUTH_BASE_URL = 'https://www.last.fm/api/'
const API_BASE_URL = 'https://ws.audioscrobbler.com/2.0'
const KeytarService = 'MoosyncLastFmToken'

enum ApiResources {
  GET_SESSION = 'auth.getSession',
  UPDATE_NOW_PLAYING = 'track.updateNowPlaying',
  SCROBBLE = 'track.scrobble',
  GET_USER_INFO = 'user.getInfo',
  GET_TRACK_INFO = 'track.getInfo',
}

type authenticatedBody = {
  api_key: string
  method: string
  token?: string
  sk?: string
}

type sessionKey = string

export class LastFMProvider extends GenericProvider {
  private _session: sessionKey | undefined
  private api = new FetchWrapper()
  private scrobbleTimeout: ReturnType<typeof setTimeout> | undefined
  private oAuthChannel: string | undefined

  private username = ''

  public loggedIn = false

  private _config: { key: string; secret: string } | undefined

  public get key() {
    return 'lastfm'
  }

  provides(): ProviderScopes[] {
    return [ProviderScopes.RECOMMENDATIONS, ProviderScopes.SCROBBLES]
  }

  private setLoggedInStatus() {
    this.loggedIn = !!this._session
  }

  public async getLoggedIn() {
    await this.authInitialized
    if (!this._session) {
      this._session = (await this.fetchStoredToken()) ?? undefined
    }
    this.setLoggedInStatus()
    return this.loggedIn
  }

  public async updateConfig(): Promise<boolean> {
    const conf = (await window.PreferenceUtils.loadSelective('lastfm')) as
      | { client_id: string; client_secret: string }
      | undefined
    const key = conf?.client_id ?? process.env.LastFmApiKey
    const secret = conf?.client_secret ?? process.env.LastFmSecret
    if (key && secret) {
      this._config = { key, secret }

      this._session = (await this.fetchStoredToken()) ?? undefined
      this.authInitializedResolver()
      return true
    }

    this.authInitializedResolver()
    return false
  }

  private async getMethodSignature(...params: object[]) {
    let str = ''
    let allParams: { [key: string]: object } = {}

    for (const data of params) {
      if (data) {
        allParams = {
          ...allParams,
          ...data,
        }
      }
    }

    const keys = Object.keys(allParams).sort()

    for (const key of keys) {
      str += key + allParams[key]
    }
    str += this._config?.secret
    return await md5(str)
  }

  private async populateRequest<T extends ApiResources>(
    fetchMethod: 'GET' | 'POST',
    lastFmMethod: T,
    data?: object,
    token?: string,
  ): Promise<LastFMResponses.ResponseType<T> | undefined> {
    if (this._config) {
      const defaultParams: authenticatedBody = {
        api_key: this._config.key,
        method: lastFmMethod,
      }

      if (token) {
        defaultParams.token = token
      } else {
        if (!this._session) {
          return
        } else {
          defaultParams.sk = this._session
        }
      }

      const signatureParams = data

      const parsedParams = new URLSearchParams({
        ...defaultParams,
        ...signatureParams,
        api_sig: await this.getMethodSignature(defaultParams, signatureParams ?? []),
        format: 'json',
      })

      const resp = await this.api.request('', {
        baseURL: API_BASE_URL,
        method: fetchMethod,
        search: fetchMethod === 'GET' ? parsedParams : {},
        body: fetchMethod === 'POST' ? parsedParams : undefined,
      })

      return resp.json()
    }
  }

  private async fetchStoredToken() {
    return window.Store.getSecure(KeytarService)
  }

  public async login() {
    if (!this._session) {
      if (!this.oAuthChannel) {
        this.oAuthChannel = await window.WindowUtils.registerOAuthCallback('lastfmcallback')
      }

      const resp = await new Promise<boolean>((resolve) => {
        window.WindowUtils.listenOAuth(this.oAuthChannel as string, async (data) => {
          const url = new URL(data)
          const token = url.searchParams.get('token') as string
          const resp = await this.populateRequest('GET', ApiResources.GET_SESSION, undefined, token)
          this._session = resp?.session?.key

          if (this._session) {
            window.Store.setSecure(KeytarService, this._session).then(() => {
              resolve(true)
            })
            return
          }
          resolve(false)
        })

        bus.emit(EventBus.SHOW_OAUTH_MODAL, {
          providerName: 'LastFM',
          url: `${AUTH_BASE_URL}auth/?api_key=${this._config?.key}&cb=https://moosync.app/lastfm`,
          providerColor: '#BA0000',
          oauthPath: 'lastfmcallback',
        } as LoginModalOptions)

        window.WindowUtils.openExternal(
          `${AUTH_BASE_URL}auth/?api_key=${this._config?.key}&cb=https://moosync.app/lastfm`,
        )
      })

      bus.emit(EventBus.HIDE_OAUTH_MODAL)
      return resp
    } else {
      return true
    }
  }

  public async signOut() {
    window.Store.removeSecure(KeytarService)
    this._session = undefined
    this.username = ''
  }

  private serializeBody(body: Record<string, unknown>) {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const newBody: Record<string, unknown> = {}
    for (const [key, value] of Object.entries(body)) {
      if (value) {
        if (Array.isArray(value)) {
          for (const index in value) {
            newBody[`${key}[${index}]`] = value[index]
          }
        } else {
          newBody[key] = value
        }
      }
    }
    return newBody
  }

  public async scrobble(song: Song | null | undefined) {
    if ((await this.getLoggedIn()) && song) {
      const parsedSong = this.serializeBody({
        track: song.title,
        album: song.album?.album_name,
        album_artist: song.album?.album_artist?.[0],
        duration: song.duration,
      })

      this.populateRequest('POST', ApiResources.UPDATE_NOW_PLAYING, {
        ...parsedSong,
        artist: song.artists && song.artists.length > 0 && song.artists[0].artist_name,
      })

      if (this.scrobbleTimeout) {
        clearTimeout(this.scrobbleTimeout)
      }

      this.scrobbleTimeout = setTimeout(async () => {
        await this.populateRequest('POST', ApiResources.SCROBBLE, {
          ...parsedSong,
          artist: song.artists?.[0].artist_name,
          timestamp: (Date.now() / 1000).toFixed(0),
        })
      }, 20 * 1e3)
    }
  }

  public async getUserDetails() {
    try {
      const resp = await this.populateRequest('GET', ApiResources.GET_USER_INFO)
      this.username = resp?.user?.name ?? ''
      return this.username
    } catch (e) {
      console.error(e)
      return 'Failed to get username'
    }
  }

  private async parseTrack(track: string, artist: string) {
    const resp = await this.populateRequest('GET', ApiResources.GET_TRACK_INFO, {
      track,
      artist,
      autocorrect: 1,
    })
    return resp
  }

  private getCoverImage(parsed: LastFMResponses.TrackInfo['track'], high: boolean): string {
    if (parsed.album) {
      if (parsed.album?.image?.length >= 4)
        return high ? parsed.album.image[3]['#text'] : parsed.album.image[0]['#text']
    }
    return ''
  }

  public async *getRecommendations(): AsyncGenerator<Song[]> {
    if ((await this.getLoggedIn()) && (await this.getUserDetails()) && this.username) {
      const resp = await window.SearchUtils.scrapeLastFM(
        `https://www.last.fm/player/station/user/${this.username}/recommended`,
      )

      const parsedResponse: LastFMResponses.ScrapeResponse = JSON.parse(resp as string)
      for (const song of parsedResponse.playlist) {
        if (song.playlinks.length > 0) {
          let typeOfLink: 'YOUTUBE' | 'SPOTIFY' | undefined
          let url
          for (const link of song.playlinks) {
            if (link.affiliate === 'youtube') {
              typeOfLink = 'YOUTUBE'
              url = link.id
              break
            } else if (link.affiliate === 'spotify') {
              typeOfLink = 'SPOTIFY'
              url = link.id
              break
            }
          }

          if (typeOfLink && url) {
            const parsed = (await this.parseTrack(song.name, song.artists[0].name))?.track
            if (parsed) {
              const final: Song = {
                _id: `lastfm:${song.playlinks[0].id}`,
                title: parsed.name,
                artists: [
                  {
                    artist_id: `lastfm:author-${parsed.artist?.mbid}`,
                    artist_name: parsed.artist?.name,
                    artist_mbid: parsed.artist?.mbid,
                  },
                ],
                duration: song.duration,
                date_added: Date.now(),
                song_coverPath_high: this.getCoverImage(parsed, true),
                song_coverPath_low: this.getCoverImage(parsed, false),
                url,
                type: typeOfLink,
              }
              if (parsed.album) {
                final.album = {
                  album_name: parsed.album.title,
                  album_artist: parsed.album.artist,
                  album_coverPath_high: this.getCoverImage(parsed, true),
                  album_coverPath_low: this.getCoverImage(parsed, false),
                }
              }
              yield [final]
            }
          }
        }
      }
    }
  }

  public get Title(): string {
    return 'LastFM'
  }

  public get BgColor(): string {
    return '#BA0000'
  }

  public get IconComponent(): string {
    return 'LastFMIcon'
  }

  public matchEntityId(id: string): boolean {
    return id.startsWith('lastfm:')
  }

  public sanitizeId(id: string): string {
    return id.replace('lastfm:', '')
  }
}
