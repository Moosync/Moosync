/*
 *  youtube.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { AuthFlow, AuthStateEmitter } from '@/utils/ui/oauth/flow'

import { once } from 'events'
import { bus } from '@/mainWindow/main'
import { parseISO8601Duration } from '@/utils/common'
import { ProviderScopes } from '@/utils/commonConstants'
import { EventBus } from '@/utils/preload/ipc/constants'
import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider'
import { AuthorizationServiceConfiguration } from '@openid/appauth'
import qs from 'qs'
import { toRaw } from 'vue'
import { convertProxy } from '../common'
import { FetchWrapper } from './generics/fetchWrapper'

const BASE_URL = 'https://youtube.googleapis.com/youtube/v3/'

enum ApiResources {
  CHANNELS = 'channels',
  PLAYLISTS = 'playlists',
  PLAYLIST_ITEMS = 'playlistItems',
  VIDEO_DETAILS = 'videos',
  SEARCH = 'search',
}

export class YoutubeProvider extends GenericProvider {
  private auth!: AuthFlow
  private _config!: ReturnType<YoutubeProvider['getConfig']>

  loggedIn = false

  public get key() {
    return 'youtube'
  }

  provides(): ProviderScopes[] {
    return [
      ProviderScopes.SEARCH,
      ProviderScopes.PLAYLISTS,
      ProviderScopes.PLAYLIST_SONGS,
      ProviderScopes.ARTIST_SONGS,
      ProviderScopes.RECOMMENDATIONS,
      ProviderScopes.PLAYLIST_FROM_URL,
      ProviderScopes.SONG_FROM_URL,
    ]
  }

  private getConfig(oauthChannel: string, id: string, secret: string) {
    return {
      openIdConnectUrl: 'https://accounts.google.com',
      clientId: id,
      clientSecret: secret,
      redirectUri: 'https://moosync.app/youtube',
      scope: 'https://www.googleapis.com/auth/youtube.readonly',
      keytarService: 'MoosyncYoutubeRefreshToken',
      oAuthChannel: oauthChannel,
    }
  }

  private isEnvExists() {
    return !!(process.env.YoutubeClientID && process.env.YoutubeClientSecret)
  }

  public async updateConfig(): Promise<boolean> {
    const conf = (await window.PreferenceUtils.loadSelective('youtube')) as { client_id: string; client_secret: string }

    if ((conf?.client_id && conf.client_secret) || this.isEnvExists()) {
      const channel = await window.WindowUtils.registerOAuthCallback('ytoauth2callback')

      const secret = conf?.client_secret ?? process.env.YoutubeClientSecret
      const id = conf?.client_id ?? process.env.YoutubeClientID
      this._config = this.getConfig(channel, id, secret)

      const serviceConfig = new AuthorizationServiceConfiguration({
        authorization_endpoint: 'https://accounts.google.com/o/oauth2/v2/auth',
        revocation_endpoint: 'https://oauth2.googleapis.com/revoke',
        token_endpoint: 'https://oauth2.googleapis.com/token',
        userinfo_endpoint: 'https://openidconnect.googleapis.com/v1/userinfo',
      })

      this.auth = new AuthFlow(this._config, serviceConfig)
      this.authInitializedResolver()
      return true
    }

    this.authInitializedResolver()
    return false
  }

  private api = new FetchWrapper()

  public async getLoggedIn() {
    await this.authInitialized
    if (this.auth) {
      const validRefreshToken = await this.auth.hasValidRefreshToken()
      if ((await this.auth.loggedIn()) || validRefreshToken) {
        this.loggedIn = true
      } else {
        this.loggedIn = false
      }

      return this.loggedIn
    }
    return false
  }

  public async login() {
    if (!(await this.getLoggedIn())) {
      if (this.auth?.config) {
        const validRefreshToken = await this.auth.hasValidRefreshToken()
        if (validRefreshToken) {
          await this.auth.performWithFreshTokens()
          return true
        }

        const url = await this.auth.makeAuthorizationRequest()
        bus.emit(EventBus.SHOW_OAUTH_MODAL, {
          providerName: 'Youtube',
          url,
          providerColor: '#E62017',
          oauthPath: 'ytoauth2callback',
        } as LoginModalOptions)
        window.WindowUtils.openExternal(url)

        await once(this.auth.authStateEmitter, AuthStateEmitter.ON_TOKEN_RESPONSE)

        bus.emit(EventBus.HIDE_OAUTH_MODAL)
        return true
      }
      return false
    }
    return true
  }

  public async signOut() {
    this.auth?.signOut()
    await this.getLoggedIn()
  }

  private async populateRequest<K extends ApiResources>(
    resource: K,
    search: YoutubeResponses.SearchObject<K>,
    invalidateCache = false,
  ): Promise<YoutubeResponses.ResponseType<K>> {
    const accessToken = await this.auth?.performWithFreshTokens()
    const resp = await this.api.request(resource, {
      baseURL: BASE_URL,
      serialize: (params) => qs.stringify(params, { arrayFormat: 'repeat', encode: false }),
      search: search.params,
      method: 'GET',
      headers: { Authorization: `Bearer ${accessToken}` },
      invalidateCache,
    })

    return resp.json()
  }

  public async getUserDetails(invalidateCache = false, retries = 0): Promise<string | undefined> {
    const validRefreshToken = await this.auth?.hasValidRefreshToken()
    if ((await this.getLoggedIn()) || validRefreshToken) {
      const resp = await this.populateRequest(
        ApiResources.CHANNELS,
        {
          params: {
            part: ['id', 'snippet'],
            mine: true,
          },
        },
        invalidateCache,
      )

      const username = resp?.items?.at(0)?.snippet?.title
      if (username) {
        return username
      }

      if (retries > 0) {
        return 'Failed to get username'
      }

      return this.getUserDetails(true, retries + 1)
    }
  }

  private async parsePlaylists(items: YoutubeResponses.UserPlaylists.Item[]): Promise<ExtendedPlaylist[]> {
    const playlists: ExtendedPlaylist[] = []
    if (items.length > 0) {
      for (const p of items) {
        if (p.snippet)
          playlists.push({
            playlist_id: `youtube-playlist:${p.id}`,
            playlist_name: p.snippet.title,
            playlist_coverPath: (
              p.snippet.thumbnails.maxres ??
              p.snippet.thumbnails.high ??
              p.snippet.thumbnails.default
            ).url,
            playlist_song_count: p.contentDetails.itemCount,
            isLocal: false,
          })
      }
    }
    return playlists
  }

  public async getUserPlaylists(invalidateCache = false) {
    const validRefreshToken = await this.auth?.hasValidRefreshToken()
    if ((await this.getLoggedIn()) || validRefreshToken) {
      let nextPageToken: string | undefined
      const parsed: YoutubeResponses.UserPlaylists.Item[] = []
      do {
        const resp = await this.populateRequest(
          ApiResources.PLAYLISTS,
          {
            params: {
              part: ['id', 'contentDetails', 'snippet'],
              mine: true,
              maxResults: 50,
              pageToken: nextPageToken,
            },
          },
          invalidateCache,
        )
        parsed.push(...resp.items)
      } while (nextPageToken)
      return this.parsePlaylists(parsed)
    }
    return []
  }

  private async parsePlaylistItems(
    items: YoutubeResponses.PlaylistItems.Items[],
    invalidateCache = false,
  ): Promise<Song[]> {
    const songs: Song[] = []
    if (items.length > 0) {
      const ids = items.map((s) => ({ id: s.snippet?.resourceId.videoId, date: s.snippet?.publishedAt }))
      const details = await this.getSongDetailsFromID(invalidateCache, ...ids)
      songs.push(...details)
    }
    return songs
  }

  public matchPlaylist(str: string) {
    return !!str.match(
      /^((?:https?:)?\/\/)?((?:www|m|music)\.)?((?:youtube\.com|youtu.be))(\/(?:[\w-]+\?v=|embed\/|v\/)?)([\w-]+)(\S+)?$/,
    )
  }

  private getIDFromURL(url: string) {
    try {
      return new URL(url)?.searchParams?.get('list') ?? url
    } catch (_) {
      return url
    }
  }

  public async *getPlaylistContent(
    url: string,
    invalidateCache = false,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    const id: string | undefined = this.getIDFromURL(url)

    if (id) {
      if (await this.getLoggedIn()) {
        const resp = await this.populateRequest(
          ApiResources.PLAYLIST_ITEMS,
          {
            params: {
              part: ['id', 'snippet'],
              playlistId: id,
              maxResults: 50,
              pageToken: nextPageToken as string,
            },
          },
          invalidateCache,
        )
        const parsed = await this.parsePlaylistItems(resp.items, invalidateCache)
        yield { songs: parsed, nextPageToken: resp.nextPageToken }
      } else {
        yield window.SearchUtils.getYTPlaylistContent(id, convertProxy(nextPageToken) as never)
      }
    }
    return
  }

  private async parseVideo(items: { item: YoutubeResponses.VideoDetails.Item; date?: string }[]) {
    const songs: Song[] = []
    for (const v of items) {
      if (v.item.id && songs.findIndex((value) => value._id === v.item.id) === -1)
        songs.push({
          _id: `youtube:${v.item.id}`,
          title: v.item.snippet.title,
          artists: [
            {
              artist_id: `youtube-author:${v.item.snippet.channelId}`,
              artist_name: v.item.snippet.channelTitle.replace('-', '').replace('Topic', '').trim(),
              artist_extra_info: {
                youtube: {
                  channel_id: v.item.snippet.channelId,
                },
              },
            },
          ],
          song_coverPath_high: (
            v.item.snippet.thumbnails.maxres ??
            v.item.snippet.thumbnails.high ??
            v.item.snippet.thumbnails.default
          ).url,
          song_coverPath_low: (
            v.item.snippet.thumbnails.standard ??
            v.item.snippet.thumbnails.standard ??
            v.item.snippet.thumbnails.default
          ).url,
          album: {
            album_name: 'Misc',
          },
          date: new Date(v.item.snippet.publishedAt).toISOString().slice(0, 10),
          date_added: Date.parse(v.date ?? ''),
          duration: parseISO8601Duration(v.item.contentDetails.duration) || -1, // -1 indicates live music
          url: v.item.id,
          playbackUrl: v.item.id,
          type: 'YOUTUBE',
        })
    }
    return songs
  }

  private async getSongDetailsFromID(invalidateCache: boolean, ...songs: { id?: string; date?: string }[]) {
    const validRefreshToken = await this.auth?.hasValidRefreshToken()
    const filtered = songs.filter((val) => !!val)
    if (filtered.length > 0) {
      if ((await this.getLoggedIn()) || validRefreshToken) {
        const resp = await this.populateRequest(
          ApiResources.VIDEO_DETAILS,
          {
            params: {
              part: ['contentDetails', 'snippet'],
              id: filtered.map((val) => val.id) as string[],
              maxResults: 50,
            },
          },
          invalidateCache,
        )

        if (filtered.length !== resp.items.length) {
          console.warn('Something went wrong while parsing song details. Length mismatch')
        }

        const items: Parameters<typeof this.parseVideo>[0] = []

        for (let i = 0; i < resp.items.length; i++) {
          items.push({ item: resp.items[i], date: filtered[i].date ?? resp.items[i].snippet.publishedAt })
        }
        return this.parseVideo(items)
      }
    }
    return []
  }

  public async getPlaylistDetails(url: string, invalidateCache = false) {
    const id = this.getIDFromURL(url)

    if (id) {
      if (await this.getLoggedIn()) {
        const resp = await this.populateRequest(
          ApiResources.PLAYLISTS,
          {
            params: {
              id,
              part: ['id', 'contentDetails', 'snippet'],
              maxResults: 1,
            },
          },
          invalidateCache,
        )
        return (await this.parsePlaylists(resp.items))[0]
      } else {
        return window.SearchUtils.getYTPlaylist(id)
      }
    }
  }

  public async searchSongs(term: string, maxResults = 30, matchTitle?: boolean): Promise<Song[]> {
    const songList: Song[] = []

    if (await this.getLoggedIn()) {
      const parsedFromURL = await this.getSongDetails(term)
      if (parsedFromURL) {
        parsedFromURL && songList.push(parsedFromURL)
      } else {
        try {
          const resp = await this.populateRequest(ApiResources.SEARCH, {
            params: {
              part: ['id', 'snippet'],
              q: term,
              type: 'video',
              maxResults: maxResults,
              order: 'relevance',
              safeSearch: 'moderate',
              videoEmbeddable: true,
            },
          })

          const finalIDs: {
            id?: string | undefined
            date?: string | undefined
          }[] = []

          if (resp.items) {
            resp.items.forEach((val) => finalIDs.push({ id: val.id.videoId, date: val.snippet.publishedAt }))
            songList.push(...(await this.getSongDetailsFromID(false, ...finalIDs)))
          }
        } catch (e) {
          console.error('Youtube search api failed. Falling back to unofficial search', e)
          const resp = await this.unofficialSearch(term, matchTitle)
          if (resp && resp.songs.length > 0) {
            songList.push(...resp.songs)
          }
        }
      }
    } else {
      const resp = await this.unofficialSearch(term)
      if (resp && resp.songs.length > 0) {
        songList.push(...resp.songs)
      }
    }

    return songList
  }

  private async unofficialSearch(term: string, matchTitle = false) {
    return window.SearchUtils.searchYT(term, undefined, matchTitle, true, true)
  }

  public matchSongUrl(url: string): boolean {
    return !!url.match(
      /^((?:https?:)?\/\/)?((?:www|m|music)\.)?((?:youtube\.com|youtu.be))(\/(?:[\w-]+\?v=|embed\/|v\/)?)([\w-]+)(\S+)?$/,
    )
  }

  public async getSongDetails(url: string, invalidateCache = false): Promise<Song | undefined> {
    if (this.matchSongUrl(url)) {
      const parsedUrl = new URL(url)
      const videoID = parsedUrl.searchParams.get('v')

      if (videoID) {
        const details = await this.getSongDetailsFromID(invalidateCache, { id: videoID })
        if (details && details.length > 0) {
          return details[0]
        }

        // Apparently searching Video ID in youtube returns the proper video as first result
        const scraped = await window.SearchUtils.searchYT(videoID, undefined, false, true, true)
        if (scraped && scraped.songs.length > 0) {
          return scraped.songs.find((val) => val._id === videoID)
        }
      }
      return
    }
  }

  private async *searchAndGetRecommendations() {
    const songs = await window.SearchUtils.searchSongsByOptions({})
    const shuffled = songs.sort(() => 0.5 - Math.random())
    const selected = shuffled.slice(0, 5)

    for (const s of selected) {
      const song = (
        await this.searchSongs(
          `${s.artists ? `${s.artists?.map((val) => val.artist_name).join(', ')} - ` : ''}${s.title}`,
        )
      )[0]

      const recommendations = await window.SearchUtils.getYTSuggestions(this.sanitizeId(song._id, 'SONG'))
      yield recommendations
    }
  }

  public async *getRecommendations(): AsyncGenerator<Song[]> {
    const youtubeSongs = await window.SearchUtils.searchSongsByOptions({
      song: {
        type: 'YOUTUBE',
      },
    })

    if (youtubeSongs.length === 0) {
      yield* this.searchAndGetRecommendations()
    }

    const resp: Parameters<typeof this.getSongDetailsFromID>[1][] = []

    let count = 0
    for (const song of youtubeSongs.slice(0, 10)) {
      const songs = await window.SearchUtils.getYTSuggestions(this.sanitizeId(song._id, 'SONG'))
      count += songs.length
      yield songs
    }

    if (await this.getLoggedIn()) {
      if (count < 10) {
        // rome-ignore lint/complexity/noExtraSemicolon: False-positive
        ; (
          await this.populateRequest(ApiResources.SEARCH, {
            params: {
              part: ['id', 'snippet'],
              type: 'video',
              videoCategoryId: 10,
              videoDuration: 'short',
              videoEmbeddable: true,
              order: 'date',
              maxResults: 10 - resp.length,
            },
          })
        ).items.forEach((val) => resp.push({ id: val.id.videoId, date: val.snippet.publishedAt }))
      }

      yield await this.getSongDetailsFromID(false, ...resp)
    }
  }

  public async *getArtistSongs(
    artist: Artists,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    const channelId = artist.artist_extra_info?.youtube?.channel_id

    if (await this.getLoggedIn()) {
      const finalIDs: Parameters<typeof this.getSongDetailsFromID>[1][] = []

      if (channelId) {
        const resp = await this.populateRequest(ApiResources.SEARCH, {
          params: {
            part: ['id', 'snippet'],
            type: 'video',
            maxResults: 50,
            order: 'relevance',
            videoEmbeddable: true,
            pageToken: nextPageToken as string,
            channelId,
          },
        })

        if (resp.items) {
          resp.items.forEach((val) => finalIDs.push({ id: val.id.videoId, date: val.snippet?.publishedAt }))
        }

        while (finalIDs.length > 0) {
          yield {
            songs: await this.getSongDetailsFromID(false, ...finalIDs.splice(0, 50)),
            nextPageToken: resp.nextPageToken,
          }
        }
      } else {
        const resp = await this.populateRequest(ApiResources.SEARCH, {
          params: {
            part: ['id', 'snippet'],
            type: 'video',
            maxResults: 50,
            order: 'relevance',
            videoEmbeddable: true,
            q: `${artist.artist_name} music`,
          },
        })

        if (resp.items) {
          resp.items.forEach((val) => finalIDs.push({ id: val.id.videoId, date: val.snippet?.publishedAt }))
          yield { songs: await this.getSongDetailsFromID(false, ...finalIDs) }
        }
      }
    } else {
      if (channelId) {
        yield await window.SearchUtils.getYTPlaylistContent(channelId, convertProxy(nextPageToken as never, true))
      } else {
        const resp = await window.SearchUtils.searchYT(`${artist.artist_name}`)
        if (resp.artists?.length > 0 && resp.artists[0].artist_extra_info?.youtube?.channel_id) {
          yield await window.SearchUtils.getYTPlaylistContent(resp.artists[0].artist_extra_info.youtube.channel_id)
        }
      }
    }
  }

  private parseArtist(artist: YoutubeResponses.ChannelInfo.ChannelInfo): Artists | undefined {
    if (artist.items.length > 0) {
      return {
        artist_id: `youtube-author:${artist.items[0].id}`,
        artist_coverPath:
          artist.items[0].snippet?.thumbnails?.maxres?.url ??
          artist.items[0].snippet?.thumbnails?.high?.url ??
          artist.items[0].snippet?.thumbnails?.default?.url,
        artist_extra_info: {
          youtube: {
            channel_id: artist.items[0].id,
          },
        },
        artist_name: artist.items[0].snippet?.title,
      }
    }
  }

  public async getArtistDetails(artist: Artists) {
    if (await this.getLoggedIn()) {
      if (artist.artist_extra_info?.youtube?.channel_id) {
        const artistDetails = await this.populateRequest(ApiResources.CHANNELS, {
          params: {
            id: artist.artist_extra_info?.youtube?.channel_id,
            part: ['id', 'snippet'],
          },
        })

        return this.parseArtist(artistDetails)
      }
    }
  }

  private parseSearchArtist(...items: YoutubeResponses.SearchDetails.Item[]) {
    const artists: Artists[] = []

    for (const i of items) {
      artists.push({
        artist_id: `youtube-author:${i.snippet.channelId}`,
        artist_name: i.snippet.channelTitle,
        artist_coverPath: (i.snippet.thumbnails.maxres ?? i.snippet.thumbnails.high ?? i.snippet.thumbnails.default)
          .url,
        artist_extra_info: {
          youtube: {
            channel_id: i.snippet.channelId,
          },
        },
      })
    }

    return artists
  }

  public async searchArtists(term: string): Promise<Artists[]> {
    const artists: Artists[] = []

    if (await this.getLoggedIn()) {
      const resp = await this.populateRequest(ApiResources.SEARCH, {
        params: {
          part: ['id', 'snippet'],
          type: 'channel',
          maxResults: 50,
          order: 'relevance',
          q: term,
        },
      })

      artists.push(...this.parseSearchArtist(...resp.items))
    } else {
      const resp = await window.SearchUtils.searchYT(term, undefined, false, false, true)
      if (resp && resp.artists.length > 0) {
        artists.push(...resp.artists)
      }
    }

    return artists
  }

  private parseSearchPlaylists(...items: YoutubeResponses.SearchDetails.Item[]) {
    const playlists: Playlist[] = []

    for (const i of items) {
      playlists.push({
        playlist_id: `youtube-playlist:${i.id.playlistId}`,
        playlist_name: i.snippet.title,
        playlist_coverPath: (i.snippet.thumbnails.maxres ?? i.snippet.thumbnails.high ?? i.snippet.thumbnails.default)
          .url,
        playlist_desc: i.snippet.description,
      })
    }

    return playlists
  }

  public async searchPlaylists(term: string): Promise<Playlist[]> {
    const playlists: Playlist[] = []

    if (await this.getLoggedIn()) {
      if (this.matchPlaylist(term)) {
        const parsedFromURL = await this.getPlaylistDetails(term)
        if (parsedFromURL) {
          playlists.push(parsedFromURL)
        }
      }

      const resp = await this.populateRequest(ApiResources.SEARCH, {
        params: {
          part: ['id', 'snippet'],
          type: 'playlist',
          maxResults: 50,
          order: 'relevance',
          q: term,
        },
      })

      playlists.push(...this.parseSearchPlaylists(...resp.items))
    } else {
      const resp = await window.SearchUtils.searchYT(term, undefined, false, false, true)
      if (resp && resp.playlists.length > 0) {
        playlists.push(...resp.playlists)
      }
    }

    return playlists
  }

  public async searchAlbum(): Promise<Album[]> {
    return []
  }

  public async getSongById(id: string): Promise<Song | undefined> {
    if (this.matchEntityId(id)) {
      const sanitized = this.sanitizeId(id, 'SONG')
      const song = await this.getSongDetailsFromID(false, { id: sanitized })
      return song[0]
    }

    return
  }

  public get Title(): string {
    return 'Youtube'
  }

  public get BgColor(): string {
    return '#E62017'
  }

  public get IconComponent(): string {
    return 'YoutubeIcon'
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

  public async getRemoteURL(song: Song): Promise<string | undefined> {
    if (!song.url?.startsWith('http')) {
      return `https://www.youtube.com/watch?v=${song.url || song.playbackUrl}`
    }
    return song.url
  }

  public async getPlaybackUrlAndDuration(
    song: Song,
  ): Promise<{ url: string | undefined; duration?: number } | undefined> {
    if (!song.duration && song.url) {
      const fetchedSong = await this.getSongDetails(`https://youtube.com/watch?v=${song.url}`)
      return { url: fetchedSong?.url, duration: fetchedSong?.duration }
    }
    return { url: song.url, duration: song.duration }
  }
}
