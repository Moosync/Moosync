<!-- 
  IconHandler.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="d-flex provider-icon">
    <YoutubeIcon v-if="iconType === 'YOUTUBE'" :color="getIconColor(iconType)" :filled="true" :dropShadow="true" />
    <SpotifyIcon v-else-if="iconType === 'SPOTIFY'" :color="getIconColor(iconType)" :filled="true" :dropShadow="true" />

    <inline-svg v-else-if="iconURL && iconType === 'URL' && iconURL.endsWith('svg')" :src="iconURL" class="icon-svg" />
    <img
      referrerPolicy="no-referrer"
      v-else-if="iconURL && iconType === 'URL' && !iconURL.endsWith('svg')"
      :src="iconURL"
      alt="provider icon"
    />
  </div>
</template>

<script lang="ts">
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import { mixins, Component } from 'vue-facing-decorator'
import YoutubeIcon from '@/icons/YoutubeIcon.vue'
import SpotifyIcon from '@/icons/SpotifyIcon.vue'
import { Prop } from 'vue-facing-decorator'
import ProviderMixin from '@/utils/ui/mixins/ProviderMixin'
import { vxm } from '@/mainWindow/store'

@Component({
  components: {
    YoutubeIcon,
    SpotifyIcon
  }
})
export default class IconHandler extends mixins(ImgLoader, ProviderMixin) {
  @Prop({ default: null })
  public item!: Song | Playlist

  public iconURL = ''

  public getIconColor(providerKey: string) {
    let provider = this.getProviderByKey(providerKey.toLowerCase())
    if (!provider && providerKey === 'YOUTUBE') {
      provider = vxm.providers.youtubeProvider
    }
    return provider?.BgColor
  }

  private isSong(item: unknown): item is Song {
    return !!(item as Song)._id
  }

  private isPlaylist(item: unknown): item is Playlist {
    return !!(item as Playlist).playlist_id
  }

  private getTypeFromPlaylist(item: Playlist) {
    if (item.playlist_id && item.playlist_id.startsWith('spotify')) return 'SPOTIFY'

    if (item.playlist_id && item.playlist_id.startsWith('youtube')) return 'YOUTUBE'

    return 'URL'
  }

  public get iconType() {
    this.iconURL = ''

    if (this.item.icon) {
      this.iconURL = this.item.icon.startsWith('media://') ? this.item.icon : 'media://' + this.item.icon
      return 'URL'
    }

    if (this.isSong(this.item)) {
      if (this.item.providerExtension) {
        window.ExtensionUtils.getExtensionIcon(this.item.providerExtension).then((val) => {
          if (val) {
            this.iconURL = 'media://' + val
          }
        })
        return 'URL'
      }

      return this.item.type
    }

    if (this.isPlaylist(this.item)) {
      if (this.item.extension) {
        window.ExtensionUtils.getExtensionIcon(this.item.extension).then((val) => {
          if (val) {
            this.iconURL = 'media://' + val
          }
        })
        return 'URL'
      }

      return this.getTypeFromPlaylist(this.item)
    }

    return ''
  }
}
</script>

<style lang="sass" scoped>
.icon-svg
  height: 100% !important
  width: 100% !important
</style>
