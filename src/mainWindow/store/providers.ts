/*
 *  providers.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { action, mutation } from 'vuex-class-component'

import { ExtensionProvider } from '@/utils/ui/providers/extensionWrapper'
import { InvidiousProvider } from '@/utils/ui/providers/invidious'
import { LastFMProvider } from '../../utils/ui/providers/lastfm'
import { PipedProvider } from '../../utils/ui/providers/piped'
import { SpotifyProvider } from '../../utils/ui/providers/spotify'
import { VuexModule } from './module'
import { YoutubeProvider } from '@/utils/ui/providers/youtube'

export enum YoutubeAlts {
  YOUTUBE = 0,
  INVIDIOUS = 1,
  PIPED = 2,
}

export class ProviderStore extends VuexModule.With({ namespaced: 'providers' }) {
  public spotifyProvider = new SpotifyProvider()
  public lastfmProvider = new LastFMProvider()
  public _youtubeProvider = new YoutubeProvider()
  public _invidiousProvider = new InvidiousProvider()
  public _pipedProvider = new PipedProvider()
  public _extensionProviders: ExtensionProvider[] = []

  private _youtubeAlt = YoutubeAlts.YOUTUBE

  get youtubeProvider() {
    switch (this._youtubeAlt) {
      case YoutubeAlts.YOUTUBE:
        return this._youtubeProvider
      case YoutubeAlts.INVIDIOUS:
        return this._invidiousProvider
      case YoutubeAlts.PIPED:
        return this._pipedProvider
    }
  }

  get youtubeAlt() {
    return this._youtubeAlt
  }

  set youtubeAlt(val: YoutubeAlts) {
    this._youtubeAlt = val
  }

  get extensionProviders() {
    return this._extensionProviders
  }

  @mutation
  addExtensionProvider(provider: ExtensionProvider) {
    const index = this._extensionProviders.findIndex((val) => val.key === provider.key)
    if (index === -1) {
      this._extensionProviders.push(provider)
    } else {
      this._extensionProviders.splice(index, 1, provider)
    }
  }

  @mutation
  clearExtensionProviders() {
    this._extensionProviders.splice(0, this._extensionProviders.length)
  }

  @action
  async updateExtensionProvider(packageName: string) {
    const scopes = (await window.ExtensionUtils.getExtensionProviderScopes(packageName))[packageName]
    if (scopes.length > 0) {
      this.addExtensionProvider(new ExtensionProvider(packageName, scopes))
    }
  }

  @action
  async fetchExtensionProviders() {
    const extensions = await window.ExtensionUtils.getAllExtensions()
    this.clearExtensionProviders()

    for (const e of extensions) {
      const scopes = (await window.ExtensionUtils.getExtensionProviderScopes(e.packageName))[e.packageName]
      if (scopes.length > 0) {
        this.addExtensionProvider(new ExtensionProvider(e.packageName, scopes))
      }
    }
  }
}
