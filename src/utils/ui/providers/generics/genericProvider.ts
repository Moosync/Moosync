/*
 *  genericProvider.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

/* eslint-disable @typescript-eslint/no-unused-vars */

import { ProviderScopes } from '@/utils/commonConstants'
import 'reflect-metadata'

// rome-ignore lint/suspicious/noExplicitAny: Idk what to use otherwise
function MethodMetadataDecorator(target: unknown, propertyKey: string, descriptor: TypedPropertyDescriptor<any>) {
  return descriptor
}
export abstract class GenericProvider {
  protected authInitialized: Promise<void>
  protected authInitializedResolver!: () => void

  public abstract readonly loggedIn: boolean

  constructor() {
    this.updateConfig()
    this.authInitialized = new Promise<void>((r) => {
      this.authInitializedResolver = r
    })
  }

  public get canLogin() {
    return true
  }

  public abstract getLoggedIn(): Promise<boolean>

  /**
   * Login auth handler for provider
   * @returns Promise returned after login event is completed
   */
  public abstract login(): Promise<boolean>

  /**
   * Sign out handler for provider
   * @returns Promise returned after sign out event is completed
   */
  public abstract signOut(): Promise<void>

  /**
   * Updates config before calling login
   * Method can be used to update config last moment before login
   */
  public abstract updateConfig(): Promise<boolean>

  /**
   * Gets user details from the provider
   * @returns username as string
   */
  public abstract getUserDetails(): Promise<string | undefined>

  abstract key: string

  /**
   * Get user playlists
   * @returns Array of playlist fetched from users profile
   */
  public async getUserPlaylists(invalidateCache?: boolean): Promise<Playlist[]> {
    return []
  }

  /**
   * Gets details of single playlist.
   *
   * @param id id of playlist
   * @returns Playlist if data is found otherwise undefined
   */
  public async getPlaylistDetails(url: string, invalidateCache?: boolean): Promise<Playlist | undefined> {
    return
  }

  /**
   * Gets songs present in playlist
   * @param id
   * @returns Generator of array {@link Song}
   */
  @MethodMetadataDecorator
  public async *getPlaylistContent(
    id: string,
    invalidateCache?: boolean,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    yield { songs: [] }
  }

  /**
   * Matches playlist link to verify if current provider is suitable for given link
   * @param str link to match
   * @returns true if playlist can be parsed by current provider
   */
  public matchPlaylist(str: string): boolean {
    return false
  }

  public matchSongUrl(str: string): boolean {
    return false
  }

  /**
   * Gets playback url and duration of song from provider. When song conversion to youtube is rate limited then url and duration fetching can be deferred
   * @param song whose url and duration is to be fetched
   * @returns playback url and duration
   */
  public async getPlaybackUrlAndDuration(
    song: Song,
    playerKey: string,
  ): Promise<{ url: string | undefined; duration?: number } | undefined> {
    return
  }

  /**
   * Gets details of a song from its url
   * @param url of song
   * @returns {@link Song} details
   */
  public async getSongDetails(url: string, invalidateCache?: boolean): Promise<Song | undefined> {
    return
  }

  /**
   * Gets recommendations
   * @returns recommendations
   */
  public async *getRecommendations(): AsyncGenerator<Song[]> {
    yield []
  }

  /**
   * Get songs by artist ID
   * @param artist_id ID of artists whose tracks are to be fetched
   */
  public async *getArtistSongs(
    artist: Artists,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    yield { songs: [] }
  }

  public async *getAlbumSongs(
    album: Album,
    nextPageToken?: unknown,
  ): AsyncGenerator<{ songs: Song[]; nextPageToken?: unknown }> {
    yield { songs: [] }
  }

  public async searchSongs(term: string): Promise<Song[]> {
    return []
  }

  public async getArtistDetails(artist: Artists, forceFetch?: boolean): Promise<Artists | undefined> {
    return
  }

  public async searchArtists(term: string): Promise<Artists[]> {
    return []
  }

  public async searchPlaylists(term: string): Promise<Playlist[]> {
    return []
  }

  public async searchAlbum(term: string): Promise<Album[]> {
    return []
  }

  public async scrobble(song: Song): Promise<void> {
    return
  }

  public async validatePlaybackURL(playbackUrl: string, player: string): Promise<boolean> {
    return true
  }

  public async getSongById(id: string): Promise<Song | undefined> {
    return undefined
  }

  public async getRemoteURL(song: Song): Promise<string | undefined> {
    return song.url?.startsWith('http') ? song.url : undefined
  }

  public abstract matchEntityId(id: string): boolean
  public abstract sanitizeId(id: string, type: 'SONG' | 'PLAYLIST' | 'ALBUM' | 'ARTIST'): string

  public abstract provides(): ProviderScopes[]
  public abstract get Title(): string
  public abstract get BgColor(): string
  public abstract get IconComponent(): string
}
