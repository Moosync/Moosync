import { sleep } from '../../common'
import { FetchWrapper } from './generics/fetchWrapper'
import { GenericProvider } from './generics/genericProvider'
import { vxm } from '@/mainWindow/store'
import { ProviderScopes } from '@/utils/commonConstants'
import qs from 'qs'

const KeytarService = 'MoosyncPipedToken'

enum PipedResources {
  SEARCH = 'search',
  PLAYLIST_DETAILS = 'playlists/${playlist_id}',
  PLAYLIST_DETAILS_NEXT = 'nextpage/playlists/${playlist_id}',
  CHANNEL_DETAILS = 'channel/${channel_id}',
  CHANNEL_DETAILS_NEXT = 'nextpage/channel/${channel_id}',
  STREAM_DETAILS = 'streams/${video_id}',
  LOGIN = 'login',
  USER_PLAYLISTS = 'user/playlists',
}
export class PipedProvider extends GenericProvider {
  key = 'youtube'

  private api = new FetchWrapper()
  private _token: string | undefined

  public loggedIn = false

  public async getLoggedIn(): Promise<boolean> {
    this.loggedIn = !!this._token
    return !!this._token
  }

  private async fetchStoredToken() {
    return window.Store.getSecure(KeytarService)
  }

  public async login(): Promise<boolean> {
    const username = await window.PreferenceUtils.loadSelective('piped.username')
    const password = await window.Store.getSecure('piped.password')

    if (username && password) {
      const BASE_URL = await this.parseBaseURL()
      const resp: PipedResponses.TokenResponse = await (
        await this.api.request(PipedResources.LOGIN, {
          baseURL: BASE_URL,
          method: 'POST',
          body: JSON.stringify({
            username,
            password,
          }),
          invalidateCache: true,
        })
      ).json()

      this._token = resp.token
      await window.Store.setSecure(KeytarService, this._token)
      return true
    }

    return false
  }

  public async signOut(): Promise<void> {
    this._token = undefined
    await window.Store.removeSecure(KeytarService)
  }

  public async updateConfig(): Promise<boolean> {
    const username = await window.PreferenceUtils.loadSelective('piped.username')
    const password = await window.Store.getSecure('piped.password')

    this._token = (await this.fetchStoredToken()) ?? undefined

    this.authInitializedResolver()
    return !!(username && password)
  }

  public async getUserDetails(): Promise<string | undefined> {
    return this._token && (await window.PreferenceUtils.loadSelective('piped.username'))
  }

  private parseUserPlaylists(playlists: PipedResponses.UserPlaylistDetails.Root[]) {
    const ret: Playlist[] = []
    for (const p of playlists) {
      ret.push({
        playlist_id: `youtube-playlist:${p.id}`,
        playlist_name: p.name,
        playlist_desc: p.shortDescription,
        playlist_coverPath: this.highResImage(p.thumbnail),
        playlist_song_count: p.videos,
      })
    }

    return ret
  }

  public async getUserPlaylists(invalidateCache?: boolean | undefined): Promise<Playlist[]> {
    if (await this.getLoggedIn()) {
      const resp = await this.populateRequest(PipedResources.USER_PLAYLISTS, undefined, this._token, invalidateCache)
      if (resp) {
        return this.parseUserPlaylists(resp)
      }
    }

    return []
  }

  public matchEntityId(id: string): boolean {
    return (
      id.startsWith('youtube:') ||
      id.startsWith('youtube-playlist:') ||
      id.startsWith('youtube-author:') ||
      id.startsWith('youtube-album:')
    )
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

  public provides(): ProviderScopes[] {
    return [
      ProviderScopes.SEARCH,
      ProviderScopes.PLAYLIST_FROM_URL,
      ProviderScopes.SONG_FROM_URL,
      ProviderScopes.PLAYLIST_SONGS,
      ProviderScopes.ARTIST_SONGS,
      ProviderScopes.ALBUM_SONGS,
      ProviderScopes.SEARCH_ALBUM,
      ProviderScopes.PLAYLISTS,
      ProviderScopes.RECOMMENDATIONS,
    ]
  }

  private async parseBaseURL() {
    let BASE_URL = await window.PreferenceUtils.loadSelective<string>('piped_instance')
    if (!BASE_URL?.endsWith('/')) {
      BASE_URL += '/'
    }

    return BASE_URL
  }

  private async populateRequest<T extends PipedResources, K extends PipedResponses.SearchFilters>(
    resource: T,
    search: PipedResponses.SearchObject<T, K>,
    authorization?: string,
    invalidateCache = false,
    tries = 0,
  ): Promise<PipedResponses.ResponseType<T, K> | undefined> {
    const BASE_URL = await this.parseBaseURL()

    let parsedResource: string = resource

    if (resource.includes('${playlist_id}')) {
      parsedResource = resource.replace('${playlist_id}', (search as PipedResponses.PlaylistDetailsRequest).playlistId)
    }

    if (resource.includes('${channel_id}')) {
      parsedResource = resource.replace('${channel_id}', (search as PipedResponses.ChannelDetailsRequest).channelId)
    }

    if (resource.includes('${video_id}')) {
      parsedResource = resource.replace('${video_id}', (search as PipedResponses.StreamRequest).videoId)
    }

    const headers: Record<string, string> = {}
    if (authorization) {
      headers['Authorization'] = authorization
    }

    try {
      const resp = await this.api.request(parsedResource, {
        baseURL: BASE_URL,
        serialize: (params) => qs.stringify(params, { arrayFormat: 'repeat', encode: false }),
        search,
        method: 'GET',
        headers,
        invalidateCache,
      })

      return resp.json()
    } catch (e) {
      console.error('Error while fetching', `${BASE_URL}${parsedResource}`, e)

      if (tries < 3) {
        await sleep(1000)
        return this.populateRequest(resource, search, authorization, invalidateCache, tries + 1)
      }
    }
  }

  private completeUrl(halfUrl: string) {
    let url = halfUrl
    if (!halfUrl.startsWith('http')) {
      url = `https://example.com/${halfUrl}`
    }

    try {
      return new URL(url)
    } catch (e) {
      console.error('Failed to parse URL', url, e)
    }
  }

  private getParam(url: URL | undefined, param: string) {
    return url?.searchParams.get(param) ?? undefined
  }

  private getIdFromURL(url: string) {
    return this.getParam(this.completeUrl(url), 'v')
  }

  private getChannelIdFromUploaderUrl(url: string) {
    return url.replace('/channel/', '')
  }

  private getPlaylistIdFromUrl(url: string) {
    return this.getParam(this.completeUrl(url), 'list')
  }

  public async *getRecommendations(): AsyncGenerator<Song[]> {
    const resp = await window.SearchUtils.searchSongsByOptions({
      song: {
        type: 'YOUTUBE',
      },
    })

    for (const s of resp) {
      const videoResp = await this.populateRequest(PipedResources.STREAM_DETAILS, {
        videoId: this.sanitizeId(s._id, 'SONG'),
      })

      if (videoResp?.relatedStreams) {
        yield this.parseSongs(...videoResp.relatedStreams)
      }
    }
  }

  private parseChannelDetails(artist: PipedResponses.ChannelDetailsExtended.Root): Artists {
    return {
      artist_id: `youtube-author:${artist.id}`,
      artist_name: artist.name,
      artist_coverPath: this.highResImage(artist.avatarUrl),
      artist_extra_info: {
        youtube: {
          channel_id: artist.id,
        },
      },
    }
  }

  public async getArtistDetails(artist: Artists): Promise<Artists | undefined> {
    if (artist.artist_extra_info?.youtube?.channel_id) {
      const resp = await this.populateRequest(PipedResources.CHANNEL_DETAILS, {
        channelId: artist.artist_extra_info?.youtube?.channel_id,
      })

      if (resp) {
        return this.parseChannelDetails(resp)
      }
    }
    return
  }

  private highResImage(url?: string) {
    return url?.replaceAll(/w\d{3}/g, 'w800')?.replaceAll(/h\d{3}/g, 'h800')?.replaceAll('hqdefault', 'maxresdefault')
  }

  private parseSongs(...videos: PipedResponses.VideoDetails[]): Song[] {
    const songList: Song[] = []

    for (const v of videos) {
      if (v.url) {
        songList.push({
          _id: `youtube:${this.getIdFromURL(v.url)}`,
          title: v.title,
          artists: v.uploaderUrl
            ? [
                {
                  artist_id: `youtube-author:${this.getChannelIdFromUploaderUrl(v.uploaderUrl)}`,
                  artist_name: v.uploaderName,
                  artist_coverPath: this.highResImage(v.uploaderAvatar),
                  artist_extra_info: {
                    youtube: {
                      channel_id: this.getChannelIdFromUploaderUrl(v.uploaderUrl),
                    },
                  },
                },
              ]
            : [],
          song_coverPath_low: v.thumbnail,
          song_coverPath_high: this.highResImage(v.thumbnail),
          duration: v.duration,
          url: this.getIdFromURL(v.url),
          playbackUrl: this.getIdFromURL(v.url),
          date_added: Date.now(),
          type: 'YOUTUBE',
        })
      }
    }

    return songList
  }

  public async searchSongs(term: string): Promise<Song[]> {
    const resp = await this.populateRequest(PipedResources.SEARCH, {
      q: term,
      filter: 'music_songs',
    })

    if (resp?.items) {
      return this.parseSongs(...resp.items)
    }
    return []
  }

  private parseArtist(...artists: PipedResponses.ChannelDetails[]): Artists[] {
    const artistList: Artists[] = []

    for (const a of artists) {
      artistList.push({
        artist_id: `youtube-author:${this.getChannelIdFromUploaderUrl(a.url)}`,
        artist_name: a.name,
        artist_coverPath: this.highResImage(a.thumbnail),
        artist_extra_info: {
          youtube: {
            channel_id: this.getChannelIdFromUploaderUrl(a.url),
          },
        },
      })
    }

    return artistList
  }

  public async searchArtists(term: string): Promise<Artists[]> {
    const resp = await this.populateRequest(PipedResources.SEARCH, {
      q: term,
      filter: 'channels',
    })

    if (resp?.items) {
      return this.parseArtist(...resp.items)
    }
    return []
  }

  private parsePlaylists(...playlists: PipedResponses.PlaylistDetails[]): Playlist[] {
    const playlistList: Playlist[] = []

    for (const p of playlists) {
      playlistList.push({
        playlist_id: `youtube-playlist:${this.getPlaylistIdFromUrl(p.url)}`,
        playlist_name: p.name,
        playlist_coverPath: this.highResImage(p.thumbnail),
      })
    }

    return playlistList
  }

  public async searchPlaylists(term: string): Promise<Playlist[]> {
    const resp = await this.populateRequest(PipedResources.SEARCH, {
      q: term,
      filter: 'playlists',
    })

    if (resp?.items) {
      return this.parsePlaylists(...resp.items)
    }
    return []
  }

  private parseAlbums(...albums: PipedResponses.AlbumDetails[]): Album[] {
    const albumList: Album[] = []

    for (const a of albums) {
      albumList.push({
        album_id: `youtube-album:${this.getPlaylistIdFromUrl(a.url)}`,
        album_name: a.name,
        album_coverPath_low: a.thumbnail,
        album_coverPath_high: this.highResImage(a.thumbnail),
        album_artist: a.uploaderName,
        album_extra_info: {
          youtube: {
            album_id: this.getPlaylistIdFromUrl(a.url),
          },
        },
      })
    }

    return albumList
  }

  public async searchAlbum(term: string): Promise<Album[]> {
    const resp = await this.populateRequest(PipedResources.SEARCH, {
      q: term,
      filter: 'music_albums',
    })

    if (resp?.items) {
      return this.parseAlbums(...resp.items)
    }
    return []
  }

  public matchPlaylist(str: string): boolean {
    return vxm.providers._youtubeProvider.matchPlaylist(str)
  }

  public matchSongUrl(str: string): boolean {
    return vxm.providers._youtubeProvider.matchSongUrl(str)
  }

  private parseExtendedPlaylist(playlistId: string, playlist: PipedResponses.PlaylistDetailsExtended.Root): Playlist {
    return {
      playlist_id: `youtube-playlist:${playlistId}`,
      playlist_name: playlist.name,
      playlist_coverPath: this.highResImage(playlist.thumbnailUrl),
    }
  }

  public async getPlaylistDetails(url: string, invalidateCache?: boolean | undefined): Promise<Playlist | undefined> {
    const id = this.getPlaylistIdFromUrl(url)

    if (id) {
      const resp = await this.populateRequest(
        PipedResources.PLAYLIST_DETAILS,
        {
          playlistId: id,
        },
        this._token,
        invalidateCache,
      )

      if (resp) {
        return this.parseExtendedPlaylist(id, resp)
      }
    }
    return
  }

  public async *getPlaylistContent(
    id: string,
    invalidateCache?: boolean | undefined,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    const resp = await this.populateRequest(
      nextPageToken ? PipedResources.PLAYLIST_DETAILS_NEXT : PipedResources.PLAYLIST_DETAILS,
      {
        playlistId: id,
        nextpage: encodeURIComponent(nextPageToken as string),
      },
      this._token,
      invalidateCache,
    )

    if (resp) {
      const songs = this.parseSongs(...(resp.relatedStreams ?? []))
      yield { songs: songs, nextPageToken: resp.nextpage }
    }
  }

  public async *getArtistSongs(
    artist: Artists,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    let channelId = artist.artist_extra_info?.youtube?.channel_id

    if (!channelId && artist.artist_name) {
      const artists = await this.searchArtists(artist.artist_name)
      channelId = artists[0]?.artist_extra_info?.youtube?.channel_id
    }

    if (channelId) {
      const resp = await this.populateRequest(
        nextPageToken ? PipedResources.CHANNEL_DETAILS_NEXT : PipedResources.CHANNEL_DETAILS,
        {
          channelId,
          nextpage: encodeURIComponent(nextPageToken as string),
        },
      )

      if (resp) {
        const songs = this.parseSongs(...(resp.relatedStreams ?? []))
        yield { songs: songs, nextPageToken: resp.nextpage }
      }
    }
  }

  public async *getAlbumSongs(
    album: Album,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    let albumId = album.album_extra_info?.youtube?.album_id

    if (!albumId && album.album_name) {
      const albums = await this.searchAlbum(album.album_name)
      albumId = albums[0].album_extra_info?.youtube?.album_id
    }

    if (albumId) {
      const resp = await this.populateRequest(
        nextPageToken ? PipedResources.PLAYLIST_DETAILS_NEXT : PipedResources.PLAYLIST_DETAILS,
        {
          playlistId: albumId,
          nextpage: encodeURIComponent(nextPageToken as string),
        },
      )

      if (resp) {
        const songs = this.parseSongs(...(resp.relatedStreams ?? []))
        yield { songs: songs, nextPageToken: resp.nextpage }
      }
    }
  }

  private getHighestBitrateAudioStream(streams: PipedResponses.VideoStreamDetails.AudioStream[]) {
    return streams.sort((a, b) => b.bitrate - a.bitrate)[0]
  }

  public async getPlaybackUrlAndDuration(
    song: Song,
  ): Promise<{ url: string | undefined; duration: number } | undefined> {
    return { url: song.url, duration: song.duration }
  }

  public async getStreamUrl(src: string) {
    let videoId = src
    if (src.startsWith('http')) {
      videoId = this.getIdFromURL(src) ?? ''
    }

    if (videoId) {
      const resp = await this.populateRequest(
        PipedResources.STREAM_DETAILS,
        {
          videoId,
        },
        this._token,
        true,
      )

      if (resp) {
        const stream = this.getHighestBitrateAudioStream(resp.audioStreams)
        return stream.url
      }
    }
  }

  public async getSongById(id: string): Promise<Song | undefined> {
    if (this.matchEntityId(id)) {
      const sanitized = this.sanitizeId(id, 'SONG')
      const song = await this.searchSongs(sanitized)
      return song[0]
    }

    return
  }

  public async getRemoteURL(song: Song): Promise<string | undefined> {
    if (!song.url?.startsWith('http')) {
      return `${this.parseBaseURL()}watch?v=${song.url || song.playbackUrl}`
    }
    return song.url
  }

  public get Title(): string {
    return 'Piped'
  }

  public get BgColor(): string {
    return '#d93427'
  }

  public get IconComponent(): string {
    return 'PipedIcon'
  }
}
