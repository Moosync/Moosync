<!-- 
  index.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="h-100 w-100 parent" @contextmenu="contextHandler">
    <b-container fluid class="album-container">
      <b-row no-gutters class="page-title">
        <b-col cols="auto">{{ $t('pages.playlists') }}</b-col>
        <b-col class="button-grow" @click="newPlaylist" cols="auto">
          <PlusIcon class="add-icon mb-2" />
        </b-col>
        <b-col class="align-self-center">
          <TabCarousel :items="tabCarouselItems" :alignProvidersToEnd="true" @onItemsChanged="onTabProvidersChanged"
            @onSortClicked="sortMenuHandler" @onSearchChange="onSearchChange" :isSortAsc="isSortAsc" />
        </b-col>
      </b-row>
      <b-row class="d-flex">
        <b-col col xl="2" md="3" v-for="playlist in filteredPlaylists" :key="playlist.playlist_id" class="card-col">
          <CardView :title="playlist.playlist_name" :imgSrc="playlist.playlist_coverPath" :id="playlist.playlist_id"
            :iconBgColor="getIconBgColor(playlist)" @click="gotoPlaylist(playlist)"
            @CardContextMenu="getPlaylistMenu($event, playlist)">
            <template #icon>
              <IconHandler class="h-100" :item="playlist" />
            </template>

            <template #defaultCover>
              <PlaylistDefault v-if="playlist.playlist_id !== FAVORITES_PLAYLIST_ID" />
              <FavPlaylistIcon v-else />
            </template>
          </CardView>
        </b-col>
      </b-row>
      <DeleteModal id="playlistDeleteModal" @confirm="deletePlaylist"
        :itemName="playlistInAction && playlistInAction.playlist_name" />
      <MultiButtonModal :show="showMultiButtonModal" :slots="2" @click-1="showNewPlaylistModal"
        @click-2="showPlaylistFromURLModal">
        <template #1>
          <CreatePlaylistIcon />
        </template>
        <template #1-title> {{ $t('contextMenu.playlist.new') }} </template>

        <template #2>
          <ImportPlaylistIcon />
        </template>
        <template #2-title> {{ $t('contextMenu.playlist.addFromURL') }} </template>
      </MultiButtonModal>
    </b-container>
  </div>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import CardView from '@/mainWindow/components/generic/CardView.vue'
import { mixins } from 'vue-facing-decorator'
import RouterPushes from '@/utils/ui/mixins/RouterPushes'
import ContextMenuMixin from '@/utils/ui/mixins/ContextMenuMixin'
import { vxm } from '@/mainWindow/store'
import PlaylistDefault from '@/icons/PlaylistDefaultIcon.vue'
import DeleteModal from '../../../commonComponents/ConfirmationModal.vue'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import PlusIcon from '@/icons/PlusIcon.vue'
import ProviderMixin from '@/utils/ui/mixins/ProviderMixin'
import { FAVORITES_PLAYLIST_ID } from '@/utils/commonConstants'
import FavPlaylistIcon from '@/icons/FavPlaylistIcon.vue'
import CreatePlaylistIcon from '@/icons/CreatePlaylistIcon.vue'
import ImportPlaylistIcon from '@/icons/ImportPlaylistIcon.vue'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import IconHandler from '../../components/generic/IconHandler.vue'
import TabCarousel from '../../components/generic/TabCarousel.vue'
import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider'
import MultiButtonModal from '@/commonComponents/MultiButtonModal.vue'
import { convertProxy } from '@/utils/ui/common'

@Component({
  components: {
    CardView,
    PlaylistDefault,
    DeleteModal,
    PlusIcon,
    FavPlaylistIcon,
    IconHandler,
    TabCarousel,
    CreatePlaylistIcon,
    ImportPlaylistIcon,
    MultiButtonModal
  }
})
export default class Playlists extends mixins(RouterPushes, ContextMenuMixin, ProviderMixin, ImgLoader) {
  @Prop({ default: () => () => undefined })
  private enableRefresh!: () => void

  showMultiButtonModal = false

  private searchText = ''
  onSearchChange(searchText: string) {
    this.searchText = searchText ?? ''
  }

  get allPlaylists(): ExtendedPlaylist[] {
    return [...this.localPlaylists, ...this.remotePlaylists]
  }

  get filteredPlaylists(): ExtendedPlaylist[] {
    const playlists: ExtendedPlaylist[] = []
    for (const p of Object.values(this.activeProviders).filter((val) => val.checked)) {
      if (p.key === 'local') {
        playlists.push(...this.localPlaylists)
      } else {
        playlists.push(
          ...this.allPlaylists.filter((val) => val.extension === p.key || val.playlist_id.startsWith(p.key))
        )
      }
    }
    return playlists.filter((val) => val.playlist_name.toLowerCase().includes(this.searchText))
  }

  private localPlaylists: ExtendedPlaylist[] = []
  private remotePlaylists: ExtendedPlaylist[] = []

  playlistInAction: Playlist | null = null

  private activeProviders: Record<string, { key: string; checked: boolean }> = {}

  onTabProvidersChanged(data: { key: string; checked: boolean }) {
    this.activeProviders[data.key] = data
  }

  FAVORITES_PLAYLIST_ID = FAVORITES_PLAYLIST_ID

  private get providers() {
    return this.getAllProviders()
  }

  private providersWithPlaylists: GenericProvider[] = []

  get tabCarouselItems(): TabCarouselItem[] {
    return [
      {
        title: this.$t('playlists.local'),
        key: 'local',
        defaultChecked: true
      },
      ...this.providersWithPlaylists
        .filter((val, index) => this.providersWithPlaylists.indexOf(val) === index)
        .map((val) => ({
          key: val.key,
          title: val.Title,
          defaultChecked: true
        }))
    ]
  }

  getIconBgColor(playlist: Playlist) {
    for (const p of this.getAllProviders()) {
      if (p.matchEntityId(playlist.playlist_id)) {
        return p.BgColor
      }
    }
  }

  private async getPlaylists(invalidateCache = false) {
    this.localPlaylists.splice(0, this.localPlaylists.length)
    this.remotePlaylists.splice(0, this.remotePlaylists.length)
    this.providersWithPlaylists.splice(0, this.providersWithPlaylists.length)

    const promises: Promise<unknown>[] = []

    await this.getLocalPlaylists()

    for (const p of this.providers) {
      promises.push(
        p
          .getUserPlaylists(invalidateCache)
          .then((val) => {
            if (val.length > 0) {
              this.pushPlaylistToList(val.map((val1) => ({ ...val1, isLocal: false })))
              this.providersWithPlaylists.push(p)
            }
          })
          .then(this.sort)
      )
    }

    await Promise.all(promises)
  }

  private async getLocalPlaylists() {
    const localPlaylists = await window.SearchUtils.searchEntityByOptions<Playlist>({
      playlist: true
    })

    for (const p of localPlaylists) {
      const extended: ExtendedPlaylist = {
        ...p,
        isLocal: true
      }
      if (this.allPlaylists.findIndex((val) => val.playlist_id === p.playlist_id) === -1) {
        if (p.extension && !p.icon) {
          p.icon = await window.ExtensionUtils.getExtensionIcon(p.extension)
        }
        // this.localPlaylists.push(p)

        let providerMatch = false
        for (const provider of this.providers) {
          if (provider.matchEntityId(p.playlist_id)) {
            providerMatch = true
            this.remotePlaylists.push(extended)
            this.providersWithPlaylists.push(provider)
          }
        }

        if (!providerMatch) {
          this.localPlaylists.push(extended)
        }
      }
    }
  }

  private async pushPlaylistToList(playlists: ExtendedPlaylist[]) {
    for (const p of playlists) {
      if (this.allPlaylists.findIndex((val) => val.playlist_id === p.playlist_id) === -1) {
        if (p.extension && !p.icon) {
          p.icon = await window.ExtensionUtils.getExtensionIcon(p.extension)
        }
        this.remotePlaylists.push(p)
      }
    }
  }

  private setSort(options: PlaylistSortOptions) {
    vxm.themes.playlistSortBy = options
  }

  get isSortAsc() {
    return vxm.themes.playlistSortBy?.asc ?? true
  }

  private sort() {
    this.allPlaylists.sort((a, b) => {
      switch (vxm.themes.playlistSortBy.type) {
        case 'name':
          return vxm.themes.playlistSortBy.asc
            ? a.playlist_name.localeCompare(b.playlist_name)
            : b.playlist_name.localeCompare(a.playlist_name)
        default:
        case 'provider':
          return vxm.themes.playlistSortBy.asc
            ? a.playlist_id.localeCompare(b.playlist_id)
            : b.playlist_id.localeCompare(a.playlist_id)
      }
    })
  }

  contextHandler(event: MouseEvent) {
    this.getContextMenu(event, {
      type: 'GENERAL_PLAYLIST',
      args: {
        sort: {
          callback: this.setSort,
          current: vxm.themes.playlistSortBy
        },
        refreshCallback: this.refresh
      }
    })
  }

  sortMenuHandler(event: MouseEvent) {
    this.getContextMenu(event, {
      type: 'PLAYLIST_SORT',
      args: {
        sortOptions: {
          callback: this.setSort,
          current: vxm.themes.playlistSortBy
        }
      }
    })
  }

  deletePlaylist() {
    if (this.playlistInAction) window.DBUtils.removePlaylist(convertProxy(this.playlistInAction))
    this.refresh()
  }

  newPlaylist() {
    this.showMultiButtonModal = !this.showMultiButtonModal
  }

  showNewPlaylistModal() {
    bus.emit(EventBus.SHOW_NEW_PLAYLIST_MODAL, [], () => this.refresh())
  }

  showPlaylistFromURLModal() {
    bus.emit(EventBus.SHOW_PLAYLIST_FROM_URL_MODAL, [], () => this.refresh())
  }

  mounted() {
    this.enableRefresh()
    this.getPlaylists()
    this.listenGlobalRefresh()

    vxm.themes.$watch('playlistSortBy', this.sort)
  }

  private refresh(invalidateCache = false) {
    this.getPlaylists(invalidateCache).then(() => (vxm.playlist.updated = true))
  }

  private listenGlobalRefresh() {
    bus.on(EventBus.REFRESH_PAGE, () => {
      this.refresh(true)
    })
  }

  getPlaylistMenu(event: MouseEvent, playlist: ExtendedPlaylist) {
    this.playlistInAction = playlist
    this.getContextMenu(event, {
      type: 'PLAYLIST',
      args: {
        playlist: playlist,
        isRemote: !playlist.isLocal,
        deleteCallback: () => this.$bvModal.show('playlistDeleteModal')
      }
    })
  }
}
</script>

<style lang="sass" scoped>
.title
  font-weight: bold
  font-size: 55px
  margin-left: 15px
  margin-bottom: 50px
  margin-top: 20px

.add-icon
  width: 20px
  height: 20px
  margin-left: 15px
</style>
