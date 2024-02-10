import localforage from 'localforage'

type CacheItem = {
  expiry: number
  value: unknown
}

export class FetchCacheHandler {
  private maxAge = 604800000

  private localForage = localforage.createInstance({
    driver: [localforage.INDEXEDDB],
    name: 'fetch-cache',
  })

  async setItem<T>(url: string, data: T) {
    return this.localForage.setItem<CacheItem>(url, {
      expiry: Date.now() + this.maxAge,
      value: data,
    })
  }

  async getItem<T>(url: string): Promise<T | undefined> {
    const cache = await this.localForage.getItem<CacheItem>(url)

    if (cache) {
      if (cache?.expiry > Date.now()) {
        return cache.value as T
      }
    }
  }

  async hasItem(url: string): Promise<boolean> {
    return (await this.localForage.keys()).includes(url)
  }
}
