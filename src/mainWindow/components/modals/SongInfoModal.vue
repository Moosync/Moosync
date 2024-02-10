<!-- 
  SongInfoModal.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal class="song-url-modal" centered size="xl" :id="id" :ref="id" hide-footer hide-header>
    <div class="modal-content-container" v-if="song">
      <b-container fluid class="p-0">
        <b-row no-gutters class="d-flex">
          <b-col cols="auto">
            <SongDefault v-if="forceEmptyImg || !tmpSong?.song_coverPath_high" class="song-url-cover" />
            <b-img v-else class="song-url-cover" :src="getImgSrc(getValidImageHigh(tmpSong))" @error="handleImageError"
              referrerPolicy="no-referrer"></b-img>
            <div @click="changeSongCover" class="edit-button d-flex justify-content-center">
              <EditIcon class="align-self-center" />
            </div>
          </b-col>
          <b-col class="details" cols="8" xl="9">
            <b-row>
              <b-col>
                <b-input :id="getKey('title')" :title="getValue('title' as any)" @input="(...args: unknown[]) => onInputChange('title', args as never)"
                  class="title text-truncate editable" :value="song.title">
                </b-input>
              </b-col>
            </b-row>
            <b-row class="mt-1">
              <b-col>
                <b-tabs nav-class="custom-nav" active-nav-item-class="active-nav-item" no-nav-style
                  content-class="mt-3 tab-inner-container" justified class="h-100">
                  <div v-for="i in tabs" :key="i.tab">
                    <b-tab :title="i.tab" :id="i.tab">
                      <div class="tab-content">
                        <b-container fluid class="tab-content-container">
                          <b-row no-gutters>
                            <b-col class="field-col" :cols="showDatalist(field[0]) ? 12 : 6" v-for="field in i.items"
                              :key="getKey(field)">
                              <b-row no-gutters class="d-flex flex-nowrap">
                                <b-col cols="auto" @click="copyText(field)" class="field-title">
                                  {{ getKey(field) }}:
                                </b-col>
                                <b-col class="ml-2 d-flex">
                                  <component :is="getComponent(field)" :id="getKey(field)" :ref="getKey(field)"
                                    :title="getValue(field)" :placeholder="getPlaceholder(field)" hide-input-on-limit
                                    add-tags-on-comma :limit="getLimit(field[0])" :class="`field-value w-100 ${getComponent(field) !== 'tags-input' && 'd-flex align-items-center text-truncate'
                                      } editable`" :value="getValue(field)" :existingTags="(datalist as any)[field[0]]"
                                    @input="(...args: unknown[]) => onInputChange(field[0], args[0] as never)" @tags-updated="onTagsUpdated(field[0])"
                                    :typeahead="true" :typeahead-hide-discard="true" typeahead-style="dropdown"
                                    :typeahead-always-show="false">
                                    <span class="w-100 text-truncate" v-if="!field[1]">{{ getValue(field) }}</span>
                                  </component>
                                </b-col>
                              </b-row>
                            </b-col>
                          </b-row>
                        </b-container>
                      </div>
                    </b-tab>
                  </div>
                </b-tabs>
              </b-col>
              <b-popover id="clipboard-popover" :show.sync="showPopover" :target="popoverTarget"
                :key="`${popoverTarget}-popover`" triggers="click blur" placement="top">
                Copied!
              </b-popover>
            </b-row>
          </b-col>
        </b-row>
      </b-container>
      <div class="button-container">
        <b-button class="close-button ml-3" @click="close">{{ $t('buttons.close') }}</b-button>
        <b-button class="save-button ml-3" @click="save">{{ $t('buttons.save') }}</b-button>
      </div>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import EditIcon from '@/icons/EditIcon.vue'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import { mixins } from 'vue-facing-decorator'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import { humanByteSize } from '@/utils/common'

type DatalistArray = { key: string; value: string }[]

@Component({
  components: {
    SongDefault,
    EditIcon
  }
})
export default class SongInfoModal extends mixins(ImgLoader) {
  @Prop({ default: 'SongInfo' })
  id!: string

  song: Song | null = null
  tmpSong: Song | null = null

  forceEmptyImg = false

  datalist: { artists: DatalistArray; genre: DatalistArray; album: DatalistArray } = {
    artists: [],
    genre: [],
    album: []
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  tabs: { tab: string; items: [keyof Song, boolean, ((value: any) => string | DatalistArray)?][] }[] = [
    {
      tab: 'Song Info',
      items: [
        ['date_added', false, (d: string) => new Date(parseInt(d)).toDateString()],
        ['year', true, (y: string) => parseInt(y).toFixed(0)],
        ['playbackUrl', true],
        ['type', false],
        ['size', false, (s: number) => humanByteSize(s)],
        ['genre', true, (g: string[]) => g.map((val) => ({ key: val, value: val }))],
        ['album', true, (a: Album) => a.album_id ? [{ key: a.album_id ?? '', value: a.album_name ?? '' }] : []],
        ['artists', true, (a: Artists[]) => a.map((val) => ({ key: val.artist_id, value: val.artist_name ?? '' }))]
      ]
    },
    {
      tab: 'File Info',
      items: [
        ['bitrate', false, (s: number) => humanByteSize(s, true)],
        ['codec', false],
        ['container', false],
        ['sampleRate', false, (s: string) => `${s} Hz`],
        ['hash', false],
        ['path', false]
      ]
    }
  ]

  popoverTarget = this.getKey('title')
  showPopover = false
  popoverTimeout: ReturnType<typeof setTimeout> | undefined

  getLimit(field: keyof Song) {
    if (field === 'album') return 1
    return 6
  }

  getComponent(t: (typeof this.tabs)[0]['items'][0]) {
    if (t[0] === 'artists' || t[0] === 'genre' || t[0] === 'album') {
      return 'tags-input'
    }

    if (t[1]) {
      return 'b-input'
    }

    return 'div'
  }

  getKey(t: (typeof this.tabs)[0]['items'][0] | string) {
    let ret: string
    if (typeof t === 'string') return (ret = t)
    else ret = t[0]

    return ret.replaceAll('_', ' ')
  }

  getValue(t: (typeof this.tabs)[0]['items'][0]): string | DatalistArray {
    if (this.song !== null) {
      if (!t[2]) return this.song[t[0] as keyof Song] as string
      else {
        if (t[2] && this.song[t[0]]) {
          return t[2](this.song[t[0]])
        }
      }
    }
    return ''
  }

  getPlaceholder(t: (typeof this.tabs)[0]['items'][0]): string {
    return `Add ${t[0].charAt(0).toUpperCase() + t[0].slice(1)}`
  }

  handleImageError() {
    this.forceEmptyImg = true
  }

  close() {
    this.song = null
    this.$bvModal.hide(this.id)
  }

  async save() {
    if (this.tmpSong) {
      await window.DBUtils.updateSongs([this.tmpSong])
      this.mergeIntoOriginal()
      this.close()
    }
  }

  private mergeIntoOriginal() {
    if (this.song && this.tmpSong) {
      for (const key of Object.keys(this.song)) {
        this.song[key as keyof Song] = this.tmpSong[key as keyof Song] as never
      }
    }
  }

  async copyText(field: (typeof this.tabs)[0]['items'][0]) {
    if (this.popoverTimeout) {
      clearTimeout(this.popoverTimeout)
      this.popoverTimeout = undefined
    }

    this.popoverTarget = this.getKey(field)
    navigator.clipboard.writeText(this.getValue(field).toString())
    this.showPopover = true
    this.popoverTimeout = setTimeout(() => {
      this.showPopover = false
      this.popoverTarget = ''
    }, 1000)
  }

  onInputChange(field: keyof Song, value: never) {
    if (this.tmpSong) {
      if (field === 'date_added') {
        this.tmpSong.date_added = new Date(value as string).getTime()
        return
      }

      this.tmpSong[field] = value
    }
  }

  onTagsUpdated(field: keyof Song) {
    if (this.tmpSong) {
      const el = this.$refs[this.getKey(field)]
      if (el) {
        const value: DatalistArray = (el as never)[0]['tags']
        if (value) {
          if (field === 'artists') {
            this.tmpSong.artists = value.map((val) => ({
              artist_id: val.key,
              artist_name: val.value
            }))
          }

          if (field === 'genre') {
            this.tmpSong.genre = value.map((val) => val.value)
          }

          if (field === 'album') {
            this.tmpSong.album = {
              album_id: value[0]?.key,
              album_name: value[0]?.value
            }
          }
        }
      }
    }
  }

  showDatalist(field: keyof Song): boolean {
    return !!(field === 'artists' || field === 'genre' || field === 'album')
  }

  private async fetchDatalist() {
    this.datalist['artists'] = (await window.SearchUtils.searchEntityByOptions<Artists>({ artist: true })).map(
      (val) => ({ key: val.artist_id, value: val.artist_name })
    ) as DatalistArray

    this.datalist['album'] = (await window.SearchUtils.searchEntityByOptions<Album>({ album: true })).map((val) => ({
      key: val.album_id,
      value: val.album_name
    })) as DatalistArray

    this.datalist['genre'] = (await window.SearchUtils.searchEntityByOptions<Genre>({ genre: true })).map((val) => ({
      key: val.genre_id,
      value: val.genre_name
    }))
  }

  async changeSongCover() {
    if (this.tmpSong) {
      const file = await window.WindowUtils.openFileBrowser(true, true, [
        {
          name: 'Image',
          extensions: ['png', 'jpg', 'jpeg', 'gif', 'svg']
        }
      ])

      if (!file.canceled && file.filePaths.length > 0) {
        this.tmpSong.song_coverPath_high = file.filePaths[0]
      }
    }
  }

  private async fetchSongDetails(id: string) {
    return (
      await window.SearchUtils.searchSongsByOptions(
        {
          song: {
            _id: id
          }
        },
        true
      )
    )[0]
  }

  private validateSong(song: Song) {
    if (!song.artists) song.artists = []
    if (!song.genre) song.genre = []
    if (!song.album) song.album = {}

    return song
  }

  mounted() {
    bus.on(EventBus.SHOW_SONG_INFO_MODAL, async (song: Song) => {
      song = (await this.fetchSongDetails(song._id)) ?? song
      this.fetchDatalist()
      this.forceEmptyImg = false
      this.song = this.validateSong(song)

      this.tmpSong = JSON.parse(JSON.stringify(song))

      if (this.song) {
        this.$bvModal.show(this.id)
      }
    })
  }
}
</script>

<style lang="sass">
.custom-nav
  border-bottom: none
  margin-bottom: 25px
  :not(:first-child)
    margin-left: 30px
  li
    text-align: left
    flex: 0 0 auto !important
    *
      padding: 0
      color: var(--textPrimary)

.editable
  color: var(--textPrimary)
  border-bottom: transparent 1px solid !important
  &:focus
    border-bottom: var(--accent) 1px solid !important

.input-tag
  font-size: 14px

.active-nav-item
  color: var(--accent) !important
  border-bottom: var(--accent) 1px solid

.typeahead-dropdown
  max-height: 150px
  overflow-y: auto
  &::-webkit-scrollbar-track
    background: var(--tertiary)

.tags-input-typeahead-item-default
  background-color: var(--tertiary) !important

.tags-input-typeahead-item-highlighted-default
  background-color: var(--accent) !important
  color: var(--textInverse)

.tags-input-wrapper-default
  padding-top: 1px !important
  background: transparent !important
  border: none
  border-radius: 0
  color: var(--textPrimary)
  height: inherit
  padding: 0
  &.active
    box-shadow: none !important
    border: none !important
  input
    border-bottom: transparent 1px solid
    color: var(--textPrimary) !important
    &:focus
      border-bottom: var(--accent) 1px solid

.tags-input-badge
  background-color: var(--secondary)
  color: var(--textPrimary)
  padding-top: 8px
  padding-bottom: 8px
  padding-left: 10px
  padding-right: 15px
  margin-bottom: 5px
  a
    margin-top: 6px
    margin-right: 3px
  span
    font-size: 14px

.tags-input-remove
  &::before
    background: var(--accent)
  &::after
    background: var(--accent)
</style>

<style lang="sass" scoped>
.tab-content
  position: absolute
  width: 100%

.tab-content-container
  padding-left: 0

.field-title
  text-transform: capitalize
  font-weight: 700

.field-col
  margin-bottom: 13px

.field-value
  font-size: 14px
  font-weight: 400
  width: auto

.modal-content-container
  max-height: 600px
  height: 400px
  overflow-y: visible

.title
  user-select: none
  font-size: 26px
  margin-bottom: 10px
  width: 100%
  max-width: 100%

.song-url-cover
  width: 157px
  height: 157px
  object-fit: cover
  border-radius: 16px

.edit-icon
  width: 15px
  height: 15px
  min-width: 15px
  min-height: 15px
  margin-left: 15px
  margin-top: 5px

.button-container
  position: absolute
  right: 0
  bottom: 0
  margin-bottom: 50px
  margin-right: 80px

.close-button
  border-radius: 6px
  background-color: var(--textSecondary)

.save-button
  border-radius: 6px
  border: 0
  color: var(--textInverse)
  background-color: var(--accent)

.details
  margin-left: 30px

.editable
  background-color: transparent !important
  background: transparent !important
  border: none !important
  border-radius: 0 !important
  color: var(--textPrimary) !important
  height: inherit
  padding: 0 !important
  border-bottom: transparent 1px solid !important
  &:focus
    border-bottom: var(--accent) 1px solid !important

.edit-button
  position: absolute
  width: 100%
  height: 100%
  background: rgba(0, 0, 0, 0.6)
  top: 0
  left: 0
  opacity: 0
  border-radius: 16px
  transition: opacity 0.2s ease
  cursor: pointer
  &:hover
    opacity: 1
  svg
    width: 70%
</style>
