/*
 *  CacheMixin.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component, Vue } from 'vue-facing-decorator'

@Component
export default class CacheMixin extends Vue {
  private localStorageInstance = window.localStorage

  public setItem(key: string, value: unknown, expiry?: number) {
    this.localStorageInstance.setItem(key, JSON.stringify({ value, expiry }))
  }

  public getItem<T>(key: string): T | undefined {
    const data = this.localStorageInstance.getItem(key)
    if (data) {
      const parsed = JSON.parse(data, (key, value) => {
        if (key === 'duration' && value === null) {
          return Infinity
        }
        return value
      }) as { value: never; expiry?: number }
      if (parsed.expiry && Date.now() > parsed.expiry) {
        return
      }

      return parsed.value
    }
  }
}
