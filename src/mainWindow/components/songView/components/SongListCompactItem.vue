<template>
  <b-container fluid @dblclick="onRowDoubleClicked(item)" @click="onRowSelected(index)"
    @contextmenu="onRowContext($event, item)" class="wrapper w-100" :class="{ selectedItem: selected.includes(index) }">
    <b-row no-gutters align-content="center" class="w-100">
      <LowImageCol @imgClicked="onPlayNowClicked(item)" height="56px" width="56px" :src="getValidImageLow(item)"
        :showPlayHoverButton="showPlayHoverButton" />
      <b-col cols="5" class="ml-2" align-self="center">
        <b-row no-gutters align-v="center">
          <b-col cols="auto" class="d-flex">
            <div class="title text-truncate mr-2">
              {{ item.title }}
            </div>

            <IconHandler :item="item" />
          </b-col>
        </b-row>
        <b-row no-gutters class="flex-nowrap">
          <div v-for="(artist, index) in item.artists" :key="index" class="subtitle text-truncate"
            :class="index !== 0 ? 'ml-1' : ''" @click="onSubtitleClicked(artist)">
            <span> {{ artist.artist_name }}{{ index !== (item.artists?.length ?? 0) - 1 ? ',' : '' }}</span>
          </div>
        </b-row>
      </b-col>
      <b-col cols="auto" align-self="center" offset="1" class="ml-auto timestamp">
        {{ item._id === currentSong?._id && currentSong._id ? $t('now_playing') : formattedDuration(item.duration) }}
      </b-col>
      <b-col cols="auto" align-self="center" class="button-icon ml-5" v-if="showAddToQueueButton">
        <AddToQueue title="Add song to queue" @click="onRowDoubleClicked(item)" />
      </b-col>
      <b-col v-if="!isJukeboxModeActive && showEllipsis" cols="auto" align-self="center"
        class="ml-5 mr-3 py-2 ellipsis-icon" @click="onRowContext($event, item)">
        <Ellipsis />
      </b-col>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { convertDuration } from '@/utils/common'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import { mixins } from 'vue-facing-decorator'
import { Component, Prop } from 'vue-facing-decorator'
import LowImageCol from '@/mainWindow/components/generic/LowImageCol.vue'
import Ellipsis from '@/icons/EllipsisIcon.vue'

import AddToQueue from '@/icons/AddToQueueIcon.vue'
import PlainPlay from '@/icons/AddToLibraryIcon.vue'
import { vxm } from '@/mainWindow/store'
import JukeboxMixin from '@/utils/ui/mixins/JukeboxMixin'
import IconHandler from '@/mainWindow/components/generic/IconHandler.vue'

@Component({
  components: {
    LowImageCol,
    Ellipsis,
    IconHandler,
    PlainPlay,
    AddToQueue
  },
  emits: ['onRowDoubleClicked', 'onRowContext', 'onRowSelected', 'onPlayNowClicked', 'onArtistClicked']
})
export default class SongListCompactItem extends mixins(ImgLoader, JukeboxMixin) {
  formattedDuration = convertDuration

  iconType = ''
  iconURL = ''

  private async getIconType() {
    this.iconURL = ''

    if (this.item.icon) {
      this.iconURL = 'media://' + this.item.icon
      return 'URL'
    }

    if (this.item.providerExtension) {
      const icon = await window.ExtensionUtils.getExtensionIcon(this.item.providerExtension)
      if (icon) {
        this.iconURL = 'media://' + icon
        return 'URL'
      }
    }

    return this.item.type
  }

  @Prop({ default: () => [] })
  selected!: number[]

  @Prop({ default: () => null })
  item!: Song

  @Prop({ default: 0 })
  index!: number

  @Prop({ default: true })
  showPlayHoverButton!: boolean

  @Prop({ default: true })
  showEllipsis!: boolean

  @Prop({ default: true })
  showAddToQueueButton!: boolean

  get currentSong() {
    return vxm.player.currentSong
  }

  onRowDoubleClicked(item: Song) {
    this.$emit('onRowDoubleClicked', item)
  }

  onRowContext(event: MouseEvent, item: Song) {
    event.stopPropagation()
    this.$emit('onRowContext', event, item)
  }

  onRowSelected(index: number) {
    this.$emit('onRowSelected', index)
  }

  onPlayNowClicked(item: Song) {
    this.$emit('onPlayNowClicked', item)
  }

  async onSubtitleClicked(artist: Artists) {
    this.$emit('onArtistClicked', artist)
  }

  async created() {
    this.iconType = (await this.getIconType()) ?? ''
  }
}
</script>

<style lang="sass" scoped>
.wrapper
  background: var(--secondary)
  border-radius: 17px
  height: 80px
  border: 1px solid transparent
  &:hover
    border: 1px solid var(--divider)
  div
    user-select: none

.selectedItem
  background: var(--secondary) !important
  border: 1px solid var(--accent) !important

.title
  color: var(--textPrimary)
  font-weight: bold
  font-size: 16px

.subtitle
  color: var(--textPrimary)
  font-size: 14px
  text-decoration: none
  cursor: pointer
  &:hover
    text-decoration: underline

.timestamp
  font-size: 14px
  color: var(--textSecondary)
  @media (max-width: 1054px)
    padding-right: 30px

.button-icon
  @media (max-width: 1213px)
    display: none

.ellipsis-icon
  cursor: pointer
  @media (max-width: 1054px)
    display: none
</style>
