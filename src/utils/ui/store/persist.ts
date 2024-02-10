/*
 *  persist.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import merge from 'deepmerge'
import { Store } from 'vuex'
import { convertProxy } from '../common'

export function createPersist() {
  return (store: Store<{ state: unknown }>) => {
    setInitialState(store)
  }
}

async function setInitialState(store: Store<{ state: unknown }>) {
  const savedState = await window.PreferenceUtils.loadSelective<boolean>('persisted', false)
  if (savedState) {
    const merged = merge(convertProxy(store.state), savedState, {
      arrayMerge: (_, saved) => saved,
      clone: false,
    })

    store.replaceState(merged)
  }
}
