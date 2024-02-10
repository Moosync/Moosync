import { URLSearchParams } from 'url'
import { isEmpty } from '@/utils/common'
import { FetchCacheHandler } from './cacheHandler'

interface StoredResponse {
  ok: boolean
  redirected: boolean
  status: number
  statusText: string
  headers: Record<string, string>
  type: ResponseType
  url: string
  bodyUsed: boolean
  data: Blob
}

interface RequestOptions extends RequestInit {
  baseURL?: string
  search?: Record<string, string | (string | undefined)[] | number | boolean | undefined> | URLSearchParams
  serialize?: (params: unknown) => string
  invalidateCache?: boolean
}

export class FetchWrapper {
  private cacheHandler = new FetchCacheHandler()
  private allowedMethods = ['GET']

  async isCached(url: string) {
    return this.cacheHandler.hasItem(url.toString())
  }

  async request(input: string, init?: RequestOptions): Promise<Response> {
    const url = new URL(`${init?.baseURL ?? ''}${input}`)

    if (init) {
      if (init.search && !init.serialize) {
        const entries = typeof init.search.entries === 'function' ? init.search.entries() : Object.entries(init.search)
        for (const [key, value] of entries) {
          if (!isEmpty(value)) url.searchParams.set(key, value.toString())
        }
      }

      if (init.serialize) {
        url.search = init.serialize(init.search)
      }
    }

    if (this.allowedMethods.includes(init?.method ?? 'GET') && !init?.invalidateCache) {
      const cache = await this.cacheHandler.getItem<StoredResponse>(url.toString())

      if (cache) {
        console.debug('Cache-hit:', url.toString())

        const headers = new Headers()
        for (const [key, value] of Object.entries(cache.headers)) {
          headers.set(key, value)
        }

        const resp: Response = {
          ...cache,
          headers,
          clone: function () {
            return this
          },
          blob: async () => {
            return cache.data
          },
          json: async () => {
            return JSON.parse(await cache.data.text())
          },
          text: () => {
            return cache.data.text()
          },
          arrayBuffer: () => {
            return cache.data.arrayBuffer()
          },
          body: cache.data.stream(),
          formData: async () => {
            return new FormData()
          },
        }

        return resp
      }
    }

    console.debug('Cache-miss:', url.toString())

    const data = await window.fetch(url, init)
    const blob = await data.clone().blob()

    const headers: Record<string, string> = {}
    for (const [key, value] of data.headers) {
      headers[key] = value
    }

    if (data.status === 200) {
      await this.cacheHandler.setItem<StoredResponse>(url.toString(), {
        ok: data.ok,
        redirected: data.redirected,
        status: data.status,
        statusText: data.statusText,
        headers,
        type: data.type,
        url: data.url,
        bodyUsed: data.bodyUsed,
        data: blob,
      })
    }

    return data
  }
}
