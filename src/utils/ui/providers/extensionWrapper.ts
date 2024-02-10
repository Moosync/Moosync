import { bus } from '@/mainWindow/main'
import { vxm } from '@/mainWindow/store'
/* eslint-disable @typescript-eslint/ban-types */
/* eslint-disable prefer-rest-params */
import { ProviderScopes } from '@/utils/commonConstants'
import { EventBus } from '@/utils/preload/ipc/constants'
import 'reflect-metadata'
import { GenericProvider } from './generics/genericProvider'

type KeysMatching<T, V> = { [K in keyof T]-?: T[K] extends V ? K : never }[keyof T]

type ExecutionStack = {
  isExecStack: boolean
  stack: string[]
}

// Decorator needed for reflect-metadata to emit function metadata
// eslint-disable-next-line @typescript-eslint/no-unused-vars
function dummyDecorator(_target: unknown, _member: string) {
  // do nothing
}

export class ExtensionProvider extends GenericProvider {
  public key: string

  loggedIn = false

  private _title = ''
  private _icon = ''
  private _bgColor = 'var(--secondary)'
  private _username?: string
  private _accountId = ''
  private providerScopes: ProviderScopes[] = []

  constructor(packageName: string, scopes: ProviderScopes[]) {
    super()
    this.key = packageName
    this.providerScopes = scopes

    window.ExtensionUtils.getExtensionDisplayName(this.key).then((name) => {
      this._title = name
    })

    window.ExtensionUtils.getRegisteredAccounts(this.key).then((details) => {
      if (details[this.key] && details[this.key][0]) this.setAccountDetails(details[this.key][0])
    })
  }

  public setAccountDetails(details: StrippedAccountDetails) {
    this._title = details.name
    this.loggedIn = details.loggedIn
    this._icon = details.icon
    this._bgColor = details.bgColor
    this._username = details.username
    this._accountId = details.id

    bus.emit(EventBus.REFRESH_ACCOUNTS, this.key)
  }

  public get canLogin() {
    return !!this._accountId
  }

  public async getLoggedIn(): Promise<boolean> {
    return this.loggedIn
  }

  public async login(): Promise<boolean> {
    await window.ExtensionUtils.performAccountLogin(this.key, this._accountId, true)
    return true
  }

  public async signOut(): Promise<void> {
    await window.ExtensionUtils.performAccountLogin(this.key, this._accountId, false)
  }

  public async updateConfig(): Promise<boolean> {
    return !!this._accountId
  }

  public async getUserDetails(): Promise<string | undefined> {
    return this._username
  }

  public matchEntityId(id: string): boolean {
    return id.startsWith(`${this.key}:`)
  }

  public matchPlaylist(): boolean {
    return true
  }

  public sanitizeId(id: string): string {
    return id.replace(`${this.key}:`, '')
  }

  private isForwardRequest<T extends ExtraExtensionEventTypes>(
    data: ExtraExtensionEventReturnType<T> | ForwardRequestReturnType<T>,
  ): data is ForwardRequestReturnType<T> {
    return !!(data as ForwardRequestReturnType<T>)?.forwardTo
  }

  // eslint-disable-next-line @typescript-eslint/ban-types
  private handleForwardRequest<T extends ExtraExtensionEventTypes, K extends KeysMatching<GenericProvider, Function>>(
    method: K,
    data: ForwardRequestReturnType<T>,
    originalData: ExtraExtensionEventData<T>,
    execStack: ExecutionStack,
  ): ReturnType<GenericProvider[K]> | undefined {
    const allProviders = [
      vxm.providers.youtubeProvider,
      vxm.providers.spotifyProvider,
      vxm.providers.lastfmProvider,
      ...vxm.providers.extensionProviders,
    ]

    if (!execStack.stack.includes(this.key)) {
      execStack.stack.push(this.key)

      const forwardToProvider = allProviders.find((val) => val.key === data.forwardTo)
      if (forwardToProvider) {
        const m = forwardToProvider[method]
        if (typeof m === 'function') {
          const metadataArgs: (() => unknown)[] = Reflect.getMetadata('design:paramtypes', this, m.name)
          const args: unknown[] = []

          const transformedArgs = data.transformedData ?? originalData

          if (metadataArgs) {
            for (let i = 0; i < metadataArgs.length; i++) {
              args.push(transformedArgs[i])
            }
          } else {
            // TODO: Should type check the args somehow
            console.warn('Could not check transformed args')
            args.push(...transformedArgs)
          }

          const ret = (m as (...args: unknown[]) => ReturnType<GenericProvider[K]>).call(
            forwardToProvider,
            ...args,
            execStack,
          )

          return ret
        }
      }
    } else {
      console.error('Recursion detected in forward request, aborting...')
    }
  }

  private async sendExtensionEventRequest<T extends ExtraExtensionEventTypes>(
    type: T,
    data: ExtraExtensionEventData<T>,
  ) {
    const resp = await window.ExtensionUtils.sendEvent({
      type,
      data,
      packageName: this.key,
    })

    if (resp?.[this.key]) {
      const fetchedData = resp[this.key]
      return fetchedData
    }
  }

  private getExecStack(...args: unknown[]): ExecutionStack {
    const lastElem = args[args.length - 1]
    if ((lastElem as ExecutionStack)?.isExecStack) {
      return lastElem as ExecutionStack
    }

    return {
      isExecStack: true,
      stack: [],
    }
  }

  @dummyDecorator
  public async getUserPlaylists(invalidateCache?: boolean | undefined): Promise<ExtendedPlaylist[]> {
    const playlists: ExtendedPlaylist[] = []
    const resp = await this.sendExtensionEventRequest('requestedPlaylists', [invalidateCache ?? false])

    if (resp) {
      if (this.isForwardRequest(resp)) {
        return (
          await (this.handleForwardRequest(
            'getUserPlaylists',
            resp,
            [invalidateCache ?? false],
            this.getExecStack(...arguments),
          ) ?? [])
        ).map((val) => ({ ...val, isLocal: false }))
      }
      const icon = await window.ExtensionUtils.getExtensionIcon(this.key)
      for (const p of resp.playlists) {
        playlists.push({
          ...p,
          icon: (p.icon && `media://${p.icon}`) ?? (icon && `media://${icon}`),
          extension: this.key,
          isLocal: false,
        })
      }
    }

    return playlists
  }

  @dummyDecorator
  public async *getPlaylistContent(
    id: string,
    invalidateCache?: boolean | undefined,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    const resp = await this.sendExtensionEventRequest('requestedPlaylistSongs', [
      id,
      invalidateCache ?? false,
      nextPageToken,
    ])

    if (resp) {
      if (this.isForwardRequest(resp)) {
        const generator = this.handleForwardRequest(
          'getPlaylistContent',
          resp,
          [id, invalidateCache ?? false, nextPageToken],
          this.getExecStack(...arguments),
        )
        if (generator) {
          yield* generator
        }
        return
      }
      yield resp
    }
  }

  @dummyDecorator
  public async *getArtistSongs(
    artist: Artists,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    const resp = await this.sendExtensionEventRequest('requestedArtistSongs', [artist, nextPageToken])
    if (resp) {
      if (this.isForwardRequest(resp)) {
        return this.handleForwardRequest(
          'getArtistSongs',
          resp,
          [artist, nextPageToken],
          this.getExecStack(...arguments),
        )
      }
      yield resp
    }
  }

  @dummyDecorator
  public async *getAlbumSongs(
    album: Album,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    const resp = await this.sendExtensionEventRequest('requestedAlbumSongs', [album, nextPageToken])
    if (resp) {
      if (this.isForwardRequest(resp)) {
        return this.handleForwardRequest('getAlbumSongs', resp, [album, nextPageToken], this.getExecStack(...arguments))
      }
      yield resp
    }
  }

  @dummyDecorator
  public async getPlaylistDetails(url: string, invalidateCache?: boolean | undefined): Promise<Playlist | undefined> {
    const resp = await this.sendExtensionEventRequest('requestedPlaylistFromURL', [url, invalidateCache ?? false])
    if (resp) {
      if (this.isForwardRequest(resp)) {
        return this.handleForwardRequest(
          'getPlaylistDetails',
          resp,
          [url, invalidateCache ?? false],
          this.getExecStack(...arguments),
        )
      }
      return resp?.playlist
    }
  }

  // TODO: Match playlist url to extension
  public matchSongUrl(): boolean {
    return true
  }

  public async getSongDetails(url: string, invalidateCache?: boolean | undefined): Promise<Song | undefined> {
    const resp = await this.sendExtensionEventRequest('requestedSongFromURL', [url, invalidateCache ?? false])
    return (resp as SongReturnType)?.song
  }

  private _lastSearchResult: Record<string, SearchReturnType> = {}

  private setLastSearchResult(term: string, data: SearchReturnType | undefined | void) {
    let parsedData = data
    if (!parsedData) {
      parsedData = {
        songs: [],
        albums: [],
        artists: [],
        playlists: [],
      }
    }

    this._lastSearchResult = {
      [term]: parsedData,
    }
  }

  private getLastSearchResult(term: string) {
    return this._lastSearchResult[term]
  }

  private async splitSearch(term: string) {
    const cache = this.getLastSearchResult(term)
    if (cache) {
      return cache
    }

    const resp = await this.sendExtensionEventRequest('requestedSearchResult', [term])

    if (resp) {
      this.setLastSearchResult(term, resp as SearchReturnType)
      return resp
    }
  }

  private getSearchProperty(
    method: 'searchSongs' | 'searchArtists' | 'searchAlbum' | 'searchPlaylists',
  ): keyof SearchReturnType {
    switch (method) {
      case 'searchSongs':
        return 'songs'
      case 'searchAlbum':
        return 'albums'
      case 'searchArtists':
        return 'artists'
      case 'searchPlaylists':
        return 'playlists'
    }
  }

  @dummyDecorator
  private async handleSearchResultForwardRequest<
    T extends 'searchSongs' | 'searchArtists' | 'searchAlbum' | 'searchPlaylists',
  >(
    resp: ExtraExtensionEventReturnType<'requestedSearchResult'>,
    method: T,
    term: string,
  ): Promise<Awaited<ReturnType<GenericProvider[T]>>> {
    if (resp) {
      const property = this.getSearchProperty(method)

      if (this.isForwardRequest(resp)) {
        return ((await this.handleForwardRequest(method, resp, [term], this.getExecStack(...arguments))) ??
          []) as Awaited<ReturnType<GenericProvider[T]>>
      }
      return resp[property] as Awaited<ReturnType<GenericProvider[T]>>
    }
    return [] as Awaited<ReturnType<GenericProvider[T]>>
  }

  @dummyDecorator
  public async searchSongs(term: string): Promise<Song[]> {
    return this.handleSearchResultForwardRequest(await this.splitSearch(term), 'searchSongs', term)
  }

  @dummyDecorator
  public async searchArtists(term: string): Promise<Artists[]> {
    return this.handleSearchResultForwardRequest(await this.splitSearch(term), 'searchArtists', term)
  }

  @dummyDecorator
  public async searchAlbum(term: string): Promise<Album[]> {
    return this.handleSearchResultForwardRequest(await this.splitSearch(term), 'searchAlbum', term)
  }

  @dummyDecorator
  public async searchPlaylists(term: string): Promise<Playlist[]> {
    return this.handleSearchResultForwardRequest(await this.splitSearch(term), 'searchPlaylists', term)
  }

  @dummyDecorator
  public async *getRecommendations(): AsyncGenerator<Song[]> {
    const resp = await this.sendExtensionEventRequest('requestedRecommendations', [])

    if (resp) {
      if (this.isForwardRequest(resp)) {
        return this.handleForwardRequest('getRecommendations', resp, [], this.getExecStack(...arguments))
      }
      yield resp?.songs
    }
  }

  @dummyDecorator
  public async getPlaybackUrlAndDuration(
    song: Song,
  ): Promise<{ url: string | undefined; duration?: number } | undefined> {
    const resp = await this.sendExtensionEventRequest('playbackDetailsRequested', [
      { ...song, _id: this.sanitizeId(song._id) },
    ])
    if (resp) {
      if (this.isForwardRequest(resp)) {
        return this.handleForwardRequest('getPlaybackUrlAndDuration', resp, [song], this.getExecStack(...arguments))
      }
      return resp
    }
  }

  @dummyDecorator
  public async getSongById(id: string): Promise<Song | undefined> {
    if (this.matchEntityId(id)) {
      const sanitized = this.sanitizeId(id)
      const resp = await this.sendExtensionEventRequest('requestedSongFromId', [sanitized])
      if (resp) {
        if (this.isForwardRequest(resp)) {
          return this.handleForwardRequest('getSongById', resp, [id], this.getExecStack(...arguments))
        }
        return resp.song
      }
    }

    return
  }

  @dummyDecorator
  public async getRemoteURL(song: Song): Promise<string | undefined> {
    const resp = await this.sendExtensionEventRequest('getRemoteURL', [song])
    if (resp) {
      if (this.isForwardRequest(resp)) {
        return this.handleForwardRequest('getRemoteURL', resp, [song], this.getExecStack(...arguments))
      }
      return resp
    }
  }

  public provides(): ProviderScopes[] {
    return this.providerScopes
  }

  public get Title(): string {
    return this._title
  }

  public get BgColor(): string {
    return this._bgColor
  }

  public get IconComponent(): string {
    return this._icon
  }
}
