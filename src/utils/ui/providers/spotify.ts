/*
 *  spotify.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { AuthFlow, AuthStateEmitter } from '@/utils/ui/oauth/flow'
import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider'

import { once } from 'events'
import { bus } from '@/mainWindow/main'
import { vxm } from '@/mainWindow/store'
import { ProviderScopes } from '@/utils/commonConstants'
import { EventBus } from '@/utils/preload/ipc/constants'
import { AuthorizationServiceConfiguration } from '@openid/appauth'
import qs from 'qs'
import { FetchWrapper } from './generics/fetchWrapper'

/**
 * Spotify API base URL
 */
const BASE_URL = 'https://api.spotify.com/v1/'

enum ApiResources {
  USER_DETAILS = 'me',
  LIKED_SONGS = 'me/tracks',
  PLAYLISTS = 'me/playlists',
  PLAYLIST = 'playlists/{playlist_id}',
  PLAYLIST_ITEMS = 'playlists/{playlist_id}/tracks',
  SONG_DETAILS = 'tracks/{song_id}',
  TOP = 'me/top/{type}',
  RECOMMENDATIONS = 'recommendations',
  SEARCH = 'search',
  ARTIST_TOP = 'artists/{artist_id}/top-tracks',
  ARTIST_ALBUMS = 'artists/{artist_id}/albums',
  ARTIST = 'artists/{artist_id}',
  ALBUM = 'albums/{album_id}',
  ALBUM_SONGS = 'albums/{album_id}/tracks',
}

/**
 * API Handler for Spotify.
 */
export class SpotifyProvider extends GenericProvider {
  private auth?: AuthFlow
  private _config!: ReturnType<SpotifyProvider['getConfig']>

  public loggedIn = false

  public canPlayPremium = false
  public async shouldPlayPremium() {
    return (
      (
        await window.PreferenceUtils.loadSelectiveArrayItem<Checkbox>(
          'spotify.librespot.options.use_librespot_playback',
        )
      )?.enabled ?? true
    )
  }

  public get key() {
    return 'spotify'
  }

  provides(): ProviderScopes[] {
    return [
      ProviderScopes.SEARCH,
      ProviderScopes.PLAYLISTS,
      ProviderScopes.PLAYLIST_SONGS,
      ProviderScopes.ARTIST_SONGS,
      ProviderScopes.ALBUM_SONGS,
      ProviderScopes.RECOMMENDATIONS,
      ProviderScopes.PLAYLIST_FROM_URL,
      ProviderScopes.SONG_FROM_URL,
      ProviderScopes.SEARCH_ALBUM,
      ProviderScopes.SEARCH_ARTIST,
    ]
  }

  private api = new FetchWrapper()

  private getConfig(oauthChannel: string, id: string, secret: string) {
    return {
      openIdConnectUrl: 'https://accounts.spotify.com/authorize',
      clientId: id,
      clientSecret: secret,
      redirectUri: 'https://moosync.app/spotify',
      scope: 'playlist-read-private user-top-read user-library-read user-read-private',
      keytarService: 'MoosyncSpotifyRefreshToken',
      oAuthChannel: oauthChannel,
    }
  }

  public async updateConfig() {
    console.log('prefs', window.PreferenceUtils)
    const conf = (await window.PreferenceUtils.loadSelective('spotify')) as { client_id: string; client_secret: string }
    const channel = await window.WindowUtils.registerOAuthCallback('spotifyoauthcallback')

    const id = conf?.client_id ?? process.env.SpotifyClientID
    const secret = conf?.client_secret ?? process.env.SpotifyClientSecret

    this._config = this.getConfig(channel, id, secret)

    const serviceConfig = new AuthorizationServiceConfiguration({
      authorization_endpoint: this._config.openIdConnectUrl,
      token_endpoint: 'https://accounts.spotify.com/api/token',
      revocation_endpoint: this._config.openIdConnectUrl,
    })

    const useUserPass =
      (await window.PreferenceUtils.loadSelectiveArrayItem<Checkbox>('spotify.options.use_librespot'))?.enabled ?? false

    if (useUserPass) {
      console.debug('Trying to login using librespot')

      const username = await window.PreferenceUtils.loadSelective<string>('spotify.username')
      const password = await window.Store.getSecure('spotify.password')

      if (username && password) {
        try {
          await window.SpotifyPlayer.connect({
            auth: {
              username,
              password,
            },
            connectConfig: {
              name: 'Moosync',
              deviceType: 'computer',
              initialVolume: vxm.player.volume,
              hasVolumeControl: true,
            },
            volumeCtrl: 'linear',
          })

          const token = await window.SpotifyPlayer.getToken(this._config.scope.split(' ') as unknown[])
          if (token) {
            this.auth = new AuthFlow(this._config, serviceConfig, false)
            this.auth.setToken({
              ...token,
              scope: token.scopes.join(' '),
              token_type: 'bearer',
              expires_in: token.expires_in.toString(),
              issued_at: token.expiry_from_epoch - token.expires_in,
            })

            bus.emit(EventBus.REFRESH_ACCOUNTS, this.key)
            this.canPlayPremium = true

            console.debug('Can use librespot')

            this.authInitializedResolver()
            return true
          }
        } catch (e) {
          console.error('Error while fetching token from librespot', e)
        }
      }
    }

    if (id && secret) {
      this.auth = new AuthFlow(this._config, serviceConfig)
      this.authInitializedResolver()
      return true
    }

    this.authInitializedResolver()
    return false
  }

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
          providerName: 'Spotify',
          url,
          providerColor: '#1ED760',
          oauthPath: 'spotifyoauthcallback',
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
    this.canPlayPremium = false
    await this.getLoggedIn()
  }

  private async refreshToken() {
    if (await this.auth?.hasValidRefreshToken()) {
      await this.auth?.performWithFreshTokens()
    } else {
      const token = await window.SpotifyPlayer.getToken(this._config.scope.split(' ') as unknown[])
      if (token) {
        this.auth?.setToken({
          ...token,
          scope: token.scopes.join(' '),
          token_type: 'bearer',
          expires_in: token.expires_in.toString(),
          issued_at: token.expiry_from_epoch - token.expires_in,
        })

        bus.emit(EventBus.REFRESH_ACCOUNTS, this.key)
      }
    }
  }

  private async populateRequest<K extends ApiResources>(
    resource: K,
    search: SpotifyResponses.SearchObject<K>,
    invalidateCache = false,
  ): Promise<SpotifyResponses.ResponseType<K>> {
    const accessToken = await this.auth?.performWithFreshTokens()

    let url: string = resource

    if (resource === ApiResources.PLAYLIST_ITEMS || resource === ApiResources.PLAYLIST) {
      url = resource.replace('{playlist_id}', (search as SpotifyResponses.PlaylistItemsRequest).params.playlist_id)
    }

    if (resource === ApiResources.SONG_DETAILS) {
      url = resource.replace('{song_id}', (search as SpotifyResponses.TrackItemRequest).params.song_id)
    }

    if (resource === ApiResources.TOP) {
      url = resource.replace('{type}', (search as SpotifyResponses.TopRequest).params.type)
      search.params = undefined
    }

    if (
      resource === ApiResources.ARTIST_TOP ||
      resource === ApiResources.ARTIST_ALBUMS ||
      resource === ApiResources.ARTIST
    ) {
      url = resource.replace('{artist_id}', (search as SpotifyResponses.ArtistsTopTracks).params.id)
    }

    if (resource === ApiResources.ALBUM_SONGS || resource === ApiResources.ALBUM) {
      url = resource.replace('{album_id}', (search as SpotifyResponses.AlbumTracksRequest).params.id)
    }

    const resp = await this.api.request(url, {
      baseURL: BASE_URL,
      serialize: (params) => qs.stringify(params, { arrayFormat: 'comma', encode: false }),
      search: search.params,
      headers: { Authorization: `Bearer ${accessToken}` },
      invalidateCache,
    })

    if (resp.status === 401) {
      await this.refreshToken()
      return this.populateRequest(resource, search, invalidateCache)
    }

    return resp.json()
  }

  private async getUser(invalidateCache = true): Promise<SpotifyResponses.UserDetails.UserDetails | undefined> {
    const validRefreshToken = await this.auth?.hasValidRefreshToken()
    if ((await this.getLoggedIn()) || validRefreshToken) {
      const resp = await this.populateRequest(ApiResources.USER_DETAILS, { params: undefined }, invalidateCache)
      console.debug('got user', resp)
      return resp
    }
  }

  public async getUserDetails(): Promise<string | undefined> {
    try {
      return (await this.getUser())?.display_name
    } catch (e) {
      console.error(e)
      return 'Failed to get username'
    }
  }

  private parsePlaylists(items: SpotifyResponses.UserPlaylists.Item[]) {
    const parsed: ExtendedPlaylist[] = []
    for (const i of items) {
      parsed.push({
        playlist_id: `spotify-playlist:${i.id}`,
        playlist_name: i.name,
        playlist_coverPath: i.images?.[0] ? i.images?.[0].url : '',
        playlist_song_count: i.tracks.total,
        isLocal: false,
      })
    }
    return parsed
  }

  public async getUserPlaylists(invalidateCache = false): Promise<ExtendedPlaylist[]> {
    const limit = 20
    let offset = 0
    let hasNext = true

    const validRefreshToken = await this.auth?.hasValidRefreshToken()
    const playlists: ExtendedPlaylist[] = []

    if ((await this.getLoggedIn()) || validRefreshToken) {
      playlists.push({
        playlist_id: 'spotify-playlist:saved-tracks',
        playlist_name: 'Liked Songs',
        playlist_coverPath: 'https://t.scdn.co/images/3099b3803ad9496896c43f22fe9be8c4.png',
        isLocal: false,
      })

      while (hasNext) {
        const resp = await this.populateRequest(
          ApiResources.PLAYLISTS,
          {
            params: { limit, offset },
          },
          invalidateCache,
        )

        if (resp.next) {
          hasNext = true
          offset += limit
        } else {
          hasNext = false
        }

        playlists.push(...this.parsePlaylists(resp.items))
      }
    }
    return playlists
  }

  public async spotifyToYoutube(item: Song) {
    const res = await vxm.providers.youtubeProvider.searchSongs(
      `${item.artists?.map((val) => val.artist_name ?? '').join(', ') ?? ''} ${item.title}`,
      1,
      false,
    )

    console.debug(
      'Found',
      res?.[0]?.title,
      '-',
      res?.[0]?.playbackUrl,
      'for spotify song',
      item.artists?.map((val) => val.artist_name).join(', '),
      item.title,
    )
    if (res.length > 0) return res?.[0]
  }

  private parseSong(track: SpotifyResponses.PlaylistItems.Track): Song {
    const song: Song = {
      _id: `spotify:${track.id}`,
      title: track.name,
      album: {
        album_name: track.album.name,
        album_coverPath_high: track.album.images?.[0] ? track.album.images?.[0].url : '',
      },
      url: track.id,
      song_coverPath_high: track.album.images?.[0] ? track.album.images?.[0].url : '',
      artists: track.artists
        .filter((x, i) => i === track.artists.findIndex((v) => v.id === x.id))
        .map((artist) => ({
          artist_name: artist.name,
          artist_id: `spotify-author:${artist.id}`,
          artist_extra_info: {
            spotify: {
              artist_id: artist.id,
            },
          },
        })),
      duration: track.duration_ms / 1000,
      date_added: Date.now(),
      type: 'SPOTIFY',
    }

    if (track.album.images?.length > 0) {
      const high = track.album.images?.[0].url
      let low = high
      if (track.album.images[1]) low = track.album.images?.[1].url

      song.album = {
        ...song.album,
        album_coverPath_high: high,
        album_coverPath_low: low,
      }

      song.song_coverPath_high = high
      song.song_coverPath_low = low
    }

    return song
  }

  private async parsePlaylistItems(items: SpotifyResponses.PlaylistItems.Item[]) {
    const parsed: Song[] = []
    for (const i of items) {
      if (!i.is_local && parsed.findIndex((val) => val._id === i.track.id) === -1) parsed.push(this.parseSong(i.track))
    }
    return parsed
  }

  public matchPlaylist(str: string) {
    return !!str.match(/^(https:\/\/open.spotify.com\/playlist\/|spotify:playlist:)([a-zA-Z0-9]+)(.*)$/)
  }

  private getIDFromURL(url: string) {
    try {
      const split = new URL(url).pathname.split('/')
      return split[split.length - 1]
    } catch (_) {
      return url
    }
  }

  public async *getPlaylistContent(
    str: string,
    invalidateCache = false,
    nextPageToken?: number,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: number }> {
    const id: string | undefined = this.getIDFromURL(str)

    let nextOffset = nextPageToken ?? 0

    if (id) {
      const validRefreshToken = await this.auth?.hasValidRefreshToken()
      if ((await this.getLoggedIn()) || validRefreshToken) {
        const limit = id === 'saved-tracks' ? 50 : 100
        const parsed: Song[] = []

        let resp: SpotifyResponses.PlaylistItems.PlaylistItems

        if (id === 'saved-tracks') {
          resp = await this.populateRequest(
            ApiResources.LIKED_SONGS,
            {
              params: {
                limit,
                offset: nextOffset,
              },
            },
            invalidateCache,
          )
        } else {
          resp = await this.populateRequest(
            ApiResources.PLAYLIST_ITEMS,
            {
              params: {
                playlist_id: id,
                limit,
                offset: nextOffset,
              },
            },
            invalidateCache,
          )
        }
        const items = await this.parsePlaylistItems(resp.items)
        parsed.push(...items)
        if (resp.next) {
          nextOffset += limit
        } else {
          nextOffset = -1
        }

        if (nextOffset === -1) {
          yield { songs: items }
        } else {
          yield { songs: items, nextPageToken: nextOffset }
        }
      }
    }
    return
  }

  public async validatePlaybackURL(playbackUrl: string, player: string): Promise<boolean> {
    if (player === 'SPOTIFY') {
      await this.getLoggedIn()

      if (playbackUrl.startsWith('spotify:track:')) {
        if (playbackUrl === 'spotify:track:undefined') return false
        return true
      }
      return false
    }
    if (!playbackUrl.startsWith('spotify:track:')) return true
    return false
  }

  public async getPlaybackUrlAndDuration(song: Song, player: string) {
    if (player === 'SPOTIFY') {
      if (this.canPlayPremium && (await this.shouldPlayPremium())) {
        return { url: `spotify:track:${song.url}`, duration: song.duration }
      }
    } else {
      console.debug(`Searching for ${song.title} on youtube`)

      const ytItem = await this.spotifyToYoutube(song)
      if (ytItem) {
        return { url: ytItem.playbackUrl ?? ytItem.url, duration: ytItem.duration ?? 0 }
      }
    }
  }

  public async getPlaylistDetails(url: string, invalidateCache = false) {
    const id = this.getIDFromURL(url)

    if (id) {
      const validRefreshToken = await this.auth?.hasValidRefreshToken()
      if ((await this.getLoggedIn()) || validRefreshToken) {
        const resp = await this.populateRequest(
          ApiResources.PLAYLIST,
          {
            params: {
              playlist_id: id,
            },
          },
          invalidateCache,
        )

        return this.parsePlaylists([resp])?.[0]
      }
    }
  }

  public matchSongUrl(url: string) {
    return !!url.match(/^(https:\/\/open.spotify.com\/(track|embed)\/|spotify:track:)([a-zA-Z0-9]+)(.*)$/)
  }

  public async getSongDetails(url: string, _ = false): Promise<Song | undefined> {
    if (this.matchSongUrl(url)) {
      const parsedURL = new URL(url)
      const split = parsedURL.pathname.split('/')
      const songID = split[split.length - 1]

      const validRefreshToken = await this.auth?.hasValidRefreshToken()

      if ((await this.getLoggedIn()) || validRefreshToken) {
        const resp = await this.populateRequest(ApiResources.SONG_DETAILS, {
          params: {
            song_id: songID,
          },
        })
        if (resp) {
          const song = this.parseSong(resp)
          return song
          // const yt = await this.spotifyToYoutube(song)
          // if (yt) {
          //   song.playbackUrl = yt.playbackUrl
          //   return song
          // } else {
          //   console.error("Couldn't find song on youtube")
          // }
        }
        return
      }
    }
  }

  private parseRecommendations(recommendations: SpotifyResponses.RecommendationDetails.Recommendations) {
    const songList: Song[] = []
    for (const track of recommendations.tracks) {
      const song: Song = this.parseSong(track)
      songList.push(song)
    }
    return songList
  }

  public async *getRecommendations(): AsyncGenerator<Song[]> {
    if (await this.getLoggedIn()) {
      const seedTracks: string[] = []
      const seedArtists: string[] = []

      const userArtists = await this.populateRequest(ApiResources.TOP, {
        params: {
          type: 'artists',
          time_range: 'long_term',
        },
      })

      let libraryTracks = (
        await window.SearchUtils.searchSongsByOptions({
          song: {
            type: 'SPOTIFY',
          },
        })
      ).filter((val) => val._id.startsWith('spotify:'))

      if (libraryTracks.length > 5) {
        libraryTracks = libraryTracks.sort(() => 0.5 - Math.random()).slice(0, 5)
      }

      seedTracks.push(...libraryTracks.map((val) => val._id.replace('spotify:', '')))

      if (seedTracks.length < 5) {
        const userTracks = await this.populateRequest(ApiResources.TOP, {
          params: {
            type: 'tracks',
            time_range: 'long_term',
          },
        })

        for (let i = 0; i < 5 - seedTracks.length; i++) {
          if (userTracks.items.length > i) {
            seedTracks.push(userTracks.items[i].id)
          }
        }
      }

      for (const item of userArtists.items.slice(0, 5)) {
        seedArtists.push(item.id)
      }

      const recommendationsArtistsResp = await this.populateRequest(ApiResources.RECOMMENDATIONS, {
        params: {
          seed_artists: seedArtists,
        },
      })

      yield this.parseRecommendations(recommendationsArtistsResp)

      const recommendationsTracksResp = await this.populateRequest(ApiResources.RECOMMENDATIONS, {
        params: {
          seed_tracks: seedTracks,
        },
      })

      yield this.parseRecommendations(recommendationsTracksResp)
    }
  }

  public async searchSongs(term: string): Promise<Song[]> {
    const songList: Song[] = []
    if (await this.getLoggedIn()) {
      const parsedFromURL = await this.getSongDetails(term)
      if (parsedFromURL) {
        songList.push(parsedFromURL)
      } else {
        const resp = await this.populateRequest(ApiResources.SEARCH, {
          params: {
            query: term,
            type: 'track',
            limit: 20,
          },
        })

        if (resp.tracks) {
          for (const s of resp.tracks.items) {
            songList.push(this.parseSong(s))
          }
        }
      }
    }
    return songList
  }

  private parseArtist(artist: SpotifyResponses.RecommendationDetails.SpotifyArtist): Artists {
    return {
      artist_id: `spotify-author:${artist.id}`,
      artist_name: artist.name,
      artist_coverPath: artist.images?.at(0)?.url,
      artist_extra_info: {
        spotify: {
          artist_id: artist.id,
        },
      },
    }
  }

  private matchArtist(url: string) {
    return !!url.match(/^(https:\/\/open.spotify.com\/artist\/|spotify:artist:)([a-zA-Z0-9]+)(.*)$/)
  }

  public async searchArtists(term: string): Promise<Artists[]> {
    const artists: Artists[] = []
    if (await this.getLoggedIn()) {
      if (this.matchArtist(term)) {
        const id = this.getIDFromURL(term)
        const parsedFromURL = await this.getArtistDetails({
          artist_id: id,
          artist_extra_info: {
            spotify: {
              artist_id: id,
            },
          },
        })

        if (parsedFromURL) {
          artists.push(parsedFromURL)
        }
      } else {
        const resp = await this.populateRequest(ApiResources.SEARCH, {
          params: {
            query: term,
            type: 'artist',
            limit: 20,
          },
        })

        if (resp.artists) {
          for (const a of resp.artists.items) {
            artists.push(this.parseArtist(a))
          }
        }
      }
    }

    return artists
  }

  private async getArtistAlbums(artist_id: string) {
    if (await this.getLoggedIn()) {
      const resp = await this.populateRequest(ApiResources.ARTIST_ALBUMS, {
        params: {
          id: artist_id,
          market: 'ES',
          limit: 20,
          offset: 0,
        },
      })

      return resp.items
    }
    return []
  }

  private async *_getAlbumSongs(album: SpotifyResponses.RecommendationDetails.Album) {
    if (await this.getLoggedIn()) {
      let nextOffset = 0
      let isNext = false
      const limit = 50

      do {
        const resp = await this.populateRequest(ApiResources.ALBUM_SONGS, {
          params: {
            id: album.id,
            market: 'ES',
            limit: limit,
            offset: nextOffset,
          },
        })

        isNext = !!resp.next
        if (isNext) {
          nextOffset += limit
        }

        for (const s of resp.items) {
          yield this.parseSong({
            ...s,
            album,
          })
        }
      } while (isNext)
    }
  }

  private async correctArtist(artist: Artists) {
    if (artist.artist_name) {
      const sanitizedName = artist.artist_name
        .toLowerCase()
        .replaceAll(/vevo|official|videos|topic|music|youtube|-|,|/g, '')

      const resp = await this.searchArtists(sanitizedName)
      if (resp.length > 0) {
        window.DBUtils.updateArtist({
          artist_id: artist.artist_id,
          artist_coverPath: artist.artist_coverPath ?? resp?.[0].artist_coverPath,
          artist_extra_info: {
            spotify: resp?.[0].artist_extra_info?.spotify,
          },
        })
        return resp?.[0]
      }
    }
  }

  public async *getArtistSongs(artist: Artists): AsyncGenerator<{ songs: Song[]; nextPageToken?: string }> {
    if (await this.getLoggedIn()) {
      let artist_id = artist.artist_extra_info?.spotify?.artist_id

      if (!artist_id && artist.artist_name) {
        const resp = await this.correctArtist(artist)
        if (resp) {
          artist_id = resp.artist_extra_info?.spotify?.artist_id
        }
      }

      if (artist_id) {
        const resp = await this.populateRequest(ApiResources.ARTIST_TOP, {
          params: {
            id: artist_id,
            market: 'ES',
          },
        })

        for (const s of resp.tracks) {
          yield { songs: [this.parseSong(s)] }
        }

        const albums = await this.getArtistAlbums(artist_id)
        for (const a of albums) {
          for await (const s of this._getAlbumSongs(a)) yield { songs: [s] }
        }
      }
    }
  }

  public async getArtistDetails(artist: Artists, forceFetch = false) {
    if (await this.getLoggedIn()) {
      if (artist.artist_extra_info?.spotify?.artist_id) {
        const artistDetails = await this.populateRequest(ApiResources.ARTIST, {
          params: {
            id: artist.artist_extra_info?.spotify?.artist_id,
            market: 'ES',
          },
        })

        return this.parseArtist(artistDetails)
      }

      if (forceFetch && artist.artist_name) {
        const artistDetails = await this.searchArtists(artist.artist_name)
        if (artistDetails.length > 0) {
          return artistDetails[0]
        }
      }
    }
  }

  public async searchPlaylists(term: string): Promise<Playlist[]> {
    const playlists: ExtendedPlaylist[] = []

    if (await this.getLoggedIn()) {
      if (this.matchPlaylist(term)) {
        const parsedFromURL = await this.getPlaylistDetails(term)
        if (parsedFromURL) {
          playlists.push(parsedFromURL)
        }
      } else {
        const resp = await this.populateRequest(ApiResources.SEARCH, {
          params: {
            query: term,
            type: 'playlist',
            limit: 20,
          },
        })

        if (resp.playlists) {
          playlists.push(...this.parsePlaylists(resp.playlists.items))
        }
      }
    }
    return playlists
  }

  private parseAlbum(...items: SpotifyResponses.RecommendationDetails.Album[]) {
    const albums: Album[] = []

    for (const a of items) {
      albums.push({
        album_id: `spotify-album:${a.id}`,
        album_name: a.name,
        album_song_count: a.total_tracks,
        album_coverPath_high: a.images?.[0] ? a.images?.[0].url : '',
        album_coverPath_low: a.images?.[2] ? a.images?.[2].url : '',
        album_extra_info: {
          spotify: {
            album_id: a.id,
          },
        },
      })
    }

    return albums
  }

  private matchAlbum(url: string) {
    return !!url.match(/^(https:\/\/open.spotify.com\/album\/|spotify:album:)([a-zA-Z0-9]+)(.*)$/)
  }

  private async getAlbumDetails(album_id: string) {
    if (await this.getLoggedIn()) {
      const albumDetails = await this.populateRequest(ApiResources.ALBUM, {
        params: {
          id: album_id,
          market: 'ES',
        },
      })

      return this.parseAlbum(albumDetails)?.[0]
    }
  }

  public async searchAlbum(term: string): Promise<Album[]> {
    const albums: Album[] = []
    if (await this.getLoggedIn()) {
      if (this.matchAlbum(term)) {
        const id = this.getIDFromURL(term)
        const parsedFromURL = await this.getAlbumDetails(id)
        if (parsedFromURL) {
          albums.push(parsedFromURL)
        }
      } else {
        const resp = await this.populateRequest(ApiResources.SEARCH, {
          params: {
            query: term,
            type: 'album',
            limit: 20,
          },
        })

        if (resp.albums) {
          albums.push(...this.parseAlbum(...resp.albums.items))
        }
      }
    }

    return albums
  }

  public async *getAlbumSongs(album: Album): AsyncGenerator<{ songs: Song[]; nextPageToken?: string }> {
    if (await this.getLoggedIn()) {
      if (album.album_name) {
        let albumId = album.album_extra_info?.spotify?.album_id

        if (!albumId) {
          albumId = (await this.searchAlbum(album.album_name))?.[0]?.album_extra_info?.spotify?.album_id
        }

        if (albumId) {
          const albumDets = await this.populateRequest(ApiResources.ALBUM, {
            params: {
              id: albumId,
              market: 'ES',
            },
          })

          for await (const s of this._getAlbumSongs(albumDets)) {
            yield { songs: [s] }
          }
        }
      }
    }
  }

  public async getSongById(id: string): Promise<Song | undefined> {
    if (this.matchEntityId(id)) {
      const sanitized = this.sanitizeId(id, 'SONG')
      const song = this.getSongDetails(`https://open.spotify.com/track/${sanitized}`)
      return song
    }
    return
  }

  public async getRemoteURL(song: Song): Promise<string | undefined> {
    if (!song.url?.startsWith('http')) {
      return `https://open.spotify.com/track/${song.url}`
    }
    return song.url
  }

  public get Title(): string {
    return 'Spotify'
  }

  public get BgColor(): string {
    return '#07C330'
  }

  public get IconComponent(): string {
    return 'SpotifyIcon'
  }

  public matchEntityId(id: string): boolean {
    return (
      id.startsWith('spotify:') ||
      id.startsWith('spotify-playlist:') ||
      id.startsWith('spotify-author:') ||
      id.startsWith('spotify-album')
    )
  }

  public sanitizeId(id: string, type: 'SONG' | 'PLAYLIST' | 'ALBUM' | 'ARTIST'): string {
    switch (type) {
      case 'SONG':
        return id.replace('spotify:', '')
      case 'PLAYLIST':
        return id.replace('spotify-playlist:', '')
      case 'ALBUM':
        return id.replace('spotify-album:', '')
      case 'ARTIST':
        return id.replace('spotify-author:', '')
    }
  }
}
