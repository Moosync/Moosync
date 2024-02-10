import { vxm } from '@/mainWindow/store'
import { Component } from 'vue-facing-decorator'
import { convertProxy } from '../common'
import { GenericProvider } from '../providers/generics/genericProvider'
import ProviderMixin from './ProviderMixin'

@Component
export default class ProviderFetchMixin extends ProviderMixin {
  private loadingMap: Record<string, boolean> = {}
  public songList: Song[] = []

  private songMap: Record<string, Song> = {}
  generator:
    | ((
        provider: GenericProvider,
        nextPageToken: unknown,
      ) => AsyncGenerator<{
        songs: Song[]
        nextPageToken?: unknown
      }>)
    | undefined = undefined

  localSongFetch: ((sortBy: SongSortOptions[]) => Promise<Song[]>) | undefined

  optionalSongList: Record<string, string[]> = {}

  activeProviders: Record<string, boolean> = {
    local: true,
  }

  private nextPageToken: Record<string, unknown> = {}

  get filteredSongList() {
    return this.songList.filter((val) => {
      for (const [key, value] of Object.entries(this.activeProviders)) {
        if (this.optionalSongList[key]) {
          if (value) {
            if (this.optionalSongList[key].includes(val._id)) return true
          }
        }
      }
      return false
    })
  }

  get isLoading() {
    return Object.values(this.loadingMap).includes(true)
  }

  private async *fetchProviderSonglist(provider: GenericProvider) {
    this.loadingMap[provider.key] = true
    if (this.generator) {
      for await (const items of this.generator(provider, this.nextPageToken[provider.key])) {
        this.nextPageToken[provider.key] = items.nextPageToken
        yield items
      }
    }

    this.loadingMap[provider.key] = false
  }

  private isFetching = false

  async fetchSongList() {
    this.loadingMap.local = true
    ;((await this.localSongFetch?.(convertProxy(vxm.themes.songSortBy))) ?? []).forEach((val) => {
      this.songMap[val._id] = val
    })
    this.songList = Object.values(this.songMap)
    this.optionalSongList.local = this.songList.map((val) => val._id)
    this.loadingMap.local = false
  }

  public async fetchAll(afterFetch?: (songs: Song[]) => void, onFetchEnded?: (songCount: number) => void) {
    if (!this.isFetching) {
      this.isFetching = true

      let count = 0
      for (const key of Object.keys(this.nextPageToken)) {
        while (this.nextPageToken[key]) {
          for await (const songs of this.loadNextPageWrapper()) {
            afterFetch?.(songs.songs)
            count += songs.songs.length
          }
        }
      }

      this.songList = Object.values(this.songMap)
      onFetchEnded?.(count)
      this.isFetching = false
    }
  }

  async loadNextPage() {
    for await (const s of this.loadNextPageWrapper()) {
    }
    this.songList = Object.values(this.songMap)
  }

  async *loadNextPageWrapper() {
    for (const key of Object.keys(this.nextPageToken)) {
      if (this.nextPageToken[key]) {
        for (const [key, checked] of Object.entries(this.activeProviders)) {
          if (checked) {
            for await (const s of this.fetchRemoteProviderByKey(key)) {
              yield s
            }
          }
        }
      }
    }
  }

  private async *fetchRemoteProviderByKey(key: string) {
    const provider = this.getProviderByKey(key)
    if (provider) {
      for await (const items of this.fetchProviderSonglist(provider)) {
        for (const s of items.songs) {
          if (!this.songMap[s._id]) {
            this.songMap[s._id] = s
            if (!this.optionalSongList[provider.key]) {
              this.optionalSongList[provider.key] = []
            }
            this.optionalSongList[provider.key].push(s._id)
          }
        }

        yield items
      }
      return
    }
  }

  async onProviderChanged({ key, checked }: { key: string; checked: boolean }) {
    this.activeProviders[key] = checked
    if (checked) {
      console.debug('Fetching from provider', key, checked)
      for await (const s of this.fetchRemoteProviderByKey(key)) {
      }
      this.songList = Object.values(this.songMap)
    }
  }

  clearSongList() {
    this.songMap = {}
  }

  clearNextPageTokens() {
    this.nextPageToken = {}
  }

  hasNextPage() {
    return Object.keys(this.nextPageToken).length > 0
  }
}
