/*
 *  AccountsMixin.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component } from 'vue-facing-decorator'
import { EventBus } from '@/utils/preload/ipc/constants'
import ProviderMixin from './ProviderMixin'
import { ProviderScopes } from '@/utils/commonConstants'
import { bus } from '@/mainWindow/main'
import { vxm } from '@/mainWindow/store'

@Component
export default class AccountsMixin extends ProviderMixin {
  private _signoutProvider?: (provider: Provider) => void

  set signoutMethod(signout: ((provider: Provider) => void) | undefined) {
    this._signoutProvider = signout
  }

  get signoutMethod() {
    return this._signoutProvider
  }

  fetchProviders(): Provider[] {
    const p = this.getProvidersByScope()
    return p.map((val) => ({
      username: undefined,
      provider: val,
      key: val.key,
    }))
  }

  providers: Provider[] = []

  async getUserDetails(provider: Provider) {
    const username = (await provider?.provider.getUserDetails()) ?? ''
    provider['username'] = username
  }

  async handleClick(provider: Provider) {
    if (!(await provider?.provider.getLoggedIn())) {
      const success = await provider?.provider.updateConfig()
      if (!success) {
        window.WindowUtils.openWindow(false, { page: 'system' })
        return
      }
      return this.login(provider)
    }
    this._signoutProvider?.(provider)
  }

  async login(provider: Provider) {
    if (await provider.provider.login()) {
      try {
        await this.getUserDetails(provider)

        if (vxm.player.currentSong) {
          const providerScopes = provider.provider.provides()
          if (providerScopes.includes(ProviderScopes.SCROBBLES)) {
            provider.provider.scrobble(vxm.player.currentSong)
          }
        }
      } catch (e) {
        console.error(e)
        await provider.provider.signOut()
      }

      // Side-effect to set logged-in variable
      await provider.provider.getLoggedIn()
    }
  }

  created() {
    this.providers = this.fetchProviders()
  }

  async mounted() {
    vxm.providers.$watch(
      'youtubeAlt',
      () => {
        this.providers.forEach((val) => {
          this.getUserDetails(val)
        })
      },
      { immediate: true, deep: false },
    )

    bus.on(EventBus.REFRESH_ACCOUNTS, (providerKey?: string) => {
      if (providerKey) {
        const provider = this.providers.find((val) => val?.provider.key === providerKey)
        if (provider) {
          this.getUserDetails(provider)
        }
      }
    })
  }
}
