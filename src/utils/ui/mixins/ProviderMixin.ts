/*
 *  AccountsMixin.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component, Vue } from 'vue-facing-decorator'

import { vxm } from '@/mainWindow/store'
import { isEmpty } from '@/utils/common'
import { ProviderScopes } from '@/utils/commonConstants'
import { GenericProvider } from '../providers/generics/genericProvider'

@Component
export default class ProviderMixin extends Vue {
  getProvidersByScope(action?: ProviderScopes) {
    const allProviders = [
      vxm.providers.youtubeProvider,
      vxm.providers.spotifyProvider,
      vxm.providers.lastfmProvider,
      ...vxm.providers.extensionProviders,
    ]
    const ret: GenericProvider[] = []

    for (const provider of allProviders) {
      const provides = provider.provides()
      if (!isEmpty(action)) {
        if (provides.includes(action)) {
          ret.push(provider)
        }
      } else {
        ret.push(provider)
      }
    }

    return ret
  }

  getProviderByKey(key: string): GenericProvider | undefined {
    const allProviders: GenericProvider[] = [
      vxm.providers.youtubeProvider,
      vxm.providers.spotifyProvider,
      vxm.providers.lastfmProvider,
      ...vxm.providers.extensionProviders,
    ]

    return allProviders.find((val) => val.key.toLowerCase() === key.toLowerCase())
  }

  onProvidersChanged(callback: () => void) {
    vxm.providers.$watch('_extensionProviders', callback)
  }

  getAllProviders() {
    return [
      vxm.providers.youtubeProvider,
      vxm.providers.spotifyProvider,
      vxm.providers.lastfmProvider,
      ...vxm.providers.extensionProviders,
    ]
  }

  getProviderBySong(song: Song) {
    if (song.providerExtension) {
      return this.getProviderByKey(song.providerExtension)
    } else {
      return this.getProviderByKey(song.type)
    }
  }
}
