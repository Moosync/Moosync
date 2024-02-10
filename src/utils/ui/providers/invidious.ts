/*
 *  youtube.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider'

import { bus } from '@/mainWindow/main'
import { vxm } from '@/mainWindow/store'
import { InvidiousApiResources, ProviderScopes } from '@/utils/commonConstants'
import { EventBus } from '@/utils/preload/ipc/constants'

const KeytarService = 'MoosyncInvidiousToken'

export class InvidiousProvider extends GenericProvider {
  private _token: string | undefined
  private oAuthChannel: string | undefined

  public loggedIn = false

  public get key() {
    return 'youtube'
  }

  provides(): ProviderScopes[] {
    return [
      ProviderScopes.SEARCH,
      ProviderScopes.SEARCH_ARTIST,
      ProviderScopes.ARTIST_SONGS,
      ProviderScopes.PLAYLIST_FROM_URL,
      ProviderScopes.SONG_FROM_URL,
      ProviderScopes.PLAYLIST_SONGS,
      ProviderScopes.RECOMMENDATIONS,
    ]
  }

  public async updateConfig(): Promise<boolean> {
    const AUTH_BASE_URL = await window.PreferenceUtils.loadSelective('invidious_instance')
    this._token = (await this.fetchStoredToken()) ?? undefined

    this.authInitializedResolver()
    return !!AUTH_BASE_URL
  }

  private async fetchStoredToken() {
    return window.Store.getSecure(KeytarService)
  }

  private async populateRequest<
    T extends InvidiousResponses.InvidiousApiResources,
    K extends InvidiousResponses.SearchTypes,
  >(resource: T, search: InvidiousResponses.SearchObject<T, K>, invalidateCache = false) {
    await this.getLoggedIn()
    return window.SearchUtils.requestInvidious(resource, search, this._token, invalidateCache)
  }

  public async getLoggedIn() {
    await this.authInitialized
    this.loggedIn = !!this._token
    return !!this._token
  }

  public async login() {
    if (!(await this.getLoggedIn())) {
      if (!this.oAuthChannel) {
        this.oAuthChannel = await window.WindowUtils.registerOAuthCallback('invidiousCallback')
      }

      const AUTH_BASE_URL = await window.PreferenceUtils.loadSelective('invidious_instance')
      if (AUTH_BASE_URL) {
        const resp = await new Promise<boolean>((resolve) => {
          window.WindowUtils.listenOAuth(this.oAuthChannel as string, async (data) => {
            const url = new URL(data)
            const session = decodeURIComponent(url.searchParams.get('token') ?? '')
            if (session) {
              try {
                this._token = session
                if (this._token) {
                  window.Store.setSecure(KeytarService, this._token).then(() => {
                    resolve(true)
                  })
                  return
                }
              } catch (e) {
                console.error(e)
                resolve(false)
              }
            }
          })

          bus.emit(EventBus.SHOW_OAUTH_MODAL, {
            providerName: 'Invidious',
            url: `${AUTH_BASE_URL}/authorize_token?scopes=:*&callback_url=https://moosync.app/invidious&expire=360000`,
            providerColor: '#E62017',
            oauthPath: 'invidiousCallback',
          } as LoginModalOptions)

          window.WindowUtils.openExternal(
            `${AUTH_BASE_URL}/authorize_token?scopes=:*&callback_url=https://moosync.app/invidious`,
          )
        })

        bus.emit(EventBus.HIDE_OAUTH_MODAL)
        return resp
      }
      return false
    }
    return true
  }

  public async signOut() {
    await window.Store.removeSecure(KeytarService)
    this._token = undefined
    await this.getLoggedIn()
  }

  public async getUserDetails(): Promise<string | undefined> {
    if (this._token) return 'Anonymous'
  }

  private parsePlaylists(items: InvidiousResponses.UserPlaylists.PlaylistResponse[]): ExtendedPlaylist[] {
    const playlists: ExtendedPlaylist[] = []
    for (const p of items) {
      playlists.push({
        playlist_id: `youtube-playlist:${p.playlistId}`,
        playlist_name: p.title,
        playlist_song_count: p.videoCount,
        playlist_coverPath: p.videos[0]?.videoThumbnails[0]?.url ?? '',
        isLocal: false,
      })
    }
    return playlists
  }

  public async getUserPlaylists(invalidateCache = false) {
    const resp = await this.populateRequest(InvidiousApiResources.PLAYLISTS, { params: undefined }, invalidateCache)
    if (resp && !resp.error) {
      return this.parsePlaylists(resp ?? [])
    }
    return []
  }

  private parsePlaylistItems(items: InvidiousResponses.VideoDetails.VideoResponse[]): InvidiousSong[] {
    const songs: InvidiousSong[] = []
    for (const s of items) {
      const stream = s.formatStreams?.slice(-1).pop()
      songs.push({
        _id: `youtube:${s.videoId}`,
        title: s.title,
        duration: s.lengthSeconds,
        artists: [
          {
            artist_id: `youtube-author:${s.authorId}`,
            artist_name: s.author,
            artist_extra_info: {
              youtube: {
                channel_id: s.authorId,
              },
            },
          },
        ],
        date_added: Date.now(),
        song_coverPath_high: s.videoThumbnails?.find((val) => val.quality.includes('maxres'))?.url,
        song_coverPath_low: s.videoThumbnails?.find((val) => val.quality.includes('medium'))?.url,
        url: s.videoId,
        playbackUrl: s.videoId,
        invidiousPlaybackUrl: stream?.url ?? '',
        type: 'YOUTUBE',
      })
    }
    return songs
  }

  public matchPlaylist(url: string) {
    return vxm.providers._youtubeProvider.matchPlaylist(url)
  }

  public matchSongUrl(str: string): boolean {
    return vxm.providers._youtubeProvider.matchSongUrl(str)
  }

  private getPlaylistIDFromURL(url: string) {
    try {
      return new URL(url)?.searchParams?.get('list') ?? undefined
    } catch (e) {
      console.debug('Tried parsing', url, 'as invidious playlist but failed')
    }
  }

  public getVideoIdFromURL(url: string) {
    try {
      return new URL(url)?.searchParams?.get('v') ?? undefined
    } catch (e) {
      console.debug('Tried parsing', url, 'as invidious Video but failed')
    }
  }

  public async *getPlaylistContent(
    str: string,
    invalidateCache = false,
    nextPageToken?: number,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: number }> {
    const playlist_id = this.getPlaylistIDFromURL(str) ?? str

    const resp = await this.populateRequest(
      InvidiousApiResources.PLAYLIST_ITEMS,
      {
        params: {
          playlist_id,
          page: nextPageToken,
        },
      },
      invalidateCache,
    )

    yield { songs: this.parsePlaylistItems(resp?.videos ?? []), nextPageToken: (nextPageToken ?? 1) + 1 }
  }

  public async getPlaylistDetails(url: string, invalidateCache = false) {
    const playlist_id = this.getPlaylistIDFromURL(url)
    if (playlist_id) {
      const resp = await this.populateRequest(
        InvidiousApiResources.PLAYLIST_ITEMS,
        {
          params: {
            playlist_id,
          },
        },
        invalidateCache,
      )

      if (resp) {
        const playlists = this.parsePlaylists([resp])
        if (playlists.length > 0) {
          return playlists[0]
        }
      }
    }
  }

  private parseSong(item: InvidiousResponses.VideoDetails.VideoResponse): Song {
    return this.parsePlaylistItems([item])[0]
  }

  public async getSongDetails(url: string, invalidateCache = false): Promise<Song | undefined> {
    let videoID: string = url
    if (url.startsWith('http')) {
      const parsedUrl = new URL(url)
      videoID = parsedUrl.searchParams.get('v') ?? url
    }

    if (videoID) {
      const resp = await this.populateRequest(
        InvidiousApiResources.VIDEO_DETAILS,
        { params: { video_id: videoID } },
        invalidateCache,
      )

      if (resp) return this.parseSong(resp)
    }

    return
  }

  public async *getRecommendations(): AsyncGenerator<Song[]> {
    const resp = await this.populateRequest(InvidiousApiResources.TRENDING, { params: { type: 'music' } }, false)
    if (resp) yield this.parsePlaylistItems(resp)
  }

  public async *getArtistSongs(
    artist: Artists,
    nextPageToken?: string,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: string }> {
    let channelId = artist.artist_extra_info?.youtube?.channel_id
    if (!channelId && artist.artist_name) {
      const searchRes = await this.searchArtists(artist.artist_name)
      channelId = searchRes[0]?.artist_extra_info?.youtube?.channel_id
    }

    if (channelId) {
      const resp = await this.populateRequest(InvidiousApiResources.CHANNEL_VIDEOS, {
        params: {
          channel_id: channelId,
          continuation: nextPageToken,
        },
      })

      const songs = this.parsePlaylistItems(resp?.videos ?? [])
      yield { songs, nextPageToken: resp?.continuation }
    }
  }

  public async searchSongs(term: string): Promise<Song[]> {
    const resp = await this.populateRequest(InvidiousApiResources.SEARCH, {
      params: {
        q: term,
        type: 'video',
        sort_by: 'relevance',
      },
    })

    if (resp) return this.parsePlaylistItems(resp)
    return []
  }

  public async getArtistDetails(artist: Artists): Promise<Artists | undefined> {
    const channelId = artist.artist_extra_info?.youtube?.channel_id
    if (!channelId) {
      const artists = await this.searchArtists(artist.artist_name ?? '')
      return artists[0]
    }

    if (channelId) {
      const resp = await this.populateRequest(InvidiousApiResources.CHANNELS, {
        params: {
          channel_id: channelId,
        },
      })

      if (resp) {
        return this.parseArtists([resp])[0]
      }
    }
  }

  private parseArtists(artists: InvidiousResponses.ChannelDetails[]) {
    const artistList: Artists[] = []

    for (const a of artists) {
      artistList.push({
        artist_id: a.authorId,
        artist_name: a.author,
        artist_coverPath: `https:${a.authorThumbnails?.sort((a, b) => b.height - a.height)[0].url}`,
        artist_extra_info: {
          youtube: {
            channel_id: a.authorId,
          },
        },
      })
    }

    return artistList
  }

  public async searchArtists(term: string): Promise<Artists[]> {
    const resp = await this.populateRequest(InvidiousApiResources.SEARCH, {
      params: {
        q: term,
        type: 'channel',
        sort_by: 'relevance',
      },
    })

    return this.parseArtists(resp ?? [])
  }

  public async searchPlaylists(term: string): Promise<Playlist[]> {
    const resp = await this.populateRequest(InvidiousApiResources.SEARCH, {
      params: {
        q: term,
        type: 'playlist',
        sort_by: 'relevance',
      },
    })

    return this.parsePlaylists(resp ?? [])
  }

  public async searchAlbum(): Promise<Album[]> {
    return []
  }

  public async getSongById(id: string): Promise<Song | undefined> {
    if (this.matchEntityId(id)) {
      const sanitized = this.sanitizeId(id, 'SONG')
      const song = await this.getSongDetails(sanitized)
      return song
    }

    return
  }

  public async getRemoteURL(song: Song): Promise<string | undefined> {
    const BASE_URL = await window.PreferenceUtils.loadSelective('invidious_instance')

    if (!song.url?.startsWith('http')) {
      return `${BASE_URL}/watch?v=${song.url || song.playbackUrl}`
    }
    return song.url
  }

  public get Title(): string {
    return 'Invidious'
  }

  public get BgColor(): string {
    return '#00B6F0'
  }

  public get IconComponent(): string {
    return 'InvidiousIcon'
  }

  public async getPlaybackUrlAndDuration(
    song: Song,
  ): Promise<{ url: string | undefined; duration: number } | undefined> {
    return { url: song.url, duration: song.duration }
  }

  public matchEntityId(id: string): boolean {
    return id.startsWith('youtube:') || id.startsWith('youtube-playlist:') || id.startsWith('youtube-author:')
  }

  public sanitizeId(id: string, type: 'SONG' | 'PLAYLIST' | 'ALBUM' | 'ARTIST'): string {
    switch (type) {
      case 'SONG':
        return id.replace('youtube:', '')
      case 'PLAYLIST':
        return id.replace('youtube-playlist:', '')
      case 'ALBUM':
        return id
      case 'ARTIST':
        return id.replace('youtube-author:', '')
    }
  }
}
