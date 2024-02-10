import { convertProxy } from '@/utils/ui/common'
import { VuexModule } from './module'
import Vuex from 'vuex'
import { createProxy } from 'vuex-class-component'
import { ProxyWatchers } from 'vuex-class-component/dist/interfaces'

const persist = [
  'player._volume',
  'player.volumeMap',
  'player.currentSong',
  'player.songQueue',
  'player.repeat',
  'player.state',
  'themes.colors',
  'themes.songSortBy',
  'themes.playlistSortBy',
  'themes.entitySortBy',
  'themes._sidebarOpen',
  'themes._jukeboxMode',
  'themes._lastSearchTab',
]

export function getProxy<T extends typeof VuexModule>(
  store: InstanceType<typeof Vuex.Store<{ state: unknown }>>,
  cls: T,
): ProxyWatchers & InstanceType<T> {
  const clsExtended = cls as typeof VuexModule & {
    prototype?: {
      __namespacedPath__?: string
    }
  }

  const namespace = clsExtended.prototype.__namespacedPath__
  const proxy = createProxy(store, cls)

  if (namespace) {
    const filteredPersist = persist.filter((val) => val.includes(namespace))
    for (const p of filteredPersist) {
      proxy.$watch(
        p.substring(p.indexOf('.') + 1),
        (val) => window.PreferenceUtils.saveSelective(`persisted.${p}`, convertProxy(val)),
        { deep: true, immediate: false },
      )
    }
  }

  return proxy
}
