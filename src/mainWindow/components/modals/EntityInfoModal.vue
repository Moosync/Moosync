<!-- 
  EntityInfoModal.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-modal class="song-url-modal" centered size="xl" :id="id" :ref="id" hide-footer hide-header>
    <div class="modal-content-container" v-if="entity">
      <b-container fluid class="p-0">
        <b-row no-gutters class="d-flex">
          <b-col cols="auto">
            <SongDefault v-if="forceEmptyImg || !imgSrc" class="song-url-cover" />
            <b-img
              v-else
              class="song-url-cover"
              :src="imgSrc"
              @error="handleImageError"
              referrerPolicy="no-referrer"
            ></b-img>
            <div @click="changeEntityCover" class="edit-button d-flex justify-content-center">
              <EditIcon class="align-self-center" />
            </div>
          </b-col>
          <b-col class="details" cols="8" xl="9">
            <b-row>
              <b-col>
                <b-input :title="title" class="title text-truncate input-style" :value="title" @input="changeTitle">
                </b-input>
              </b-col>
            </b-row>
            <b-row class="mt-1">
              <b-col class="field-col" cols="12" v-for="key in filterFields" :key="key">
                <b-row no-gutters class="d-flex">
                  <b-col cols="auto" class="field-title"> {{ getParsedFieldTitle(key) }}: </b-col>
                  <b-col class="ml-1 d-flex align-items-center text-truncate">
                    <component
                      :class="`text-truncate field-value w-100 input-style ${isEditable(key) ? 'editable' : ''}`"
                      :is="isEditable(key) ? 'b-input' : 'div'"
                      :value="getValue(key)"
                      placeholder="Enter value"
                      @input="changeEntityField(key, $event)"
                    >
                      <span class="text-truncate" v-if="!isEditable(key)">{{ getValue(key) }}</span>
                    </component>
                  </b-col>
                </b-row>
              </b-col>
            </b-row>
          </b-col>
        </b-row>
      </b-container>
      <div class="button-container">
        <b-button class="close-button ml-3" @click="close">Close</b-button>
        <b-button class="save-button ml-3" @click="save">Save</b-button>
      </div>
    </div>
  </b-modal>
</template>

<script lang="ts">
import { Component, Prop, Vue } from 'vue-facing-decorator'
import SongDefault from '@/icons/SongDefaultIcon.vue'
import EditIcon from '@/icons/EditIcon.vue'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import { dotIndex } from '@/utils/common'
import { convertProxy } from '../../../utils/ui/common'

@Component({
  components: {
    SongDefault,
    EditIcon
  }
})
export default class EntityInfoModal extends Vue {
  @Prop({ default: 'EntityInfoModal' })
  id!: string

  tmpEntity: Artists | Album | Playlist | null = null
  entity: Artists | Album | Playlist | null = null

  forceEmptyImg = false

  handleImageError() {
    this.forceEmptyImg = true
  }

  get imgSrc() {
    const src =
      (this.tmpEntity as Artists).artist_coverPath ??
      (this.tmpEntity as Album).album_coverPath_high ??
      (this.tmpEntity as Playlist).playlist_coverPath

    if (src && !src?.startsWith('http')) {
      return 'media://' + src
    }

    return src
  }

  getValue(key: string) {
    if (this.entity) {
      return dotIndex(this.entity as Record<string, unknown>, key)
    }
  }

  isEditable(field: string) {
    switch (field) {
      case 'album_artist':
      case 'artist_mbid':
      case 'playlist_desc':
      case 'artist_extra_info.youtube.channel_id':
      case 'artist_extra_info.spotify.artist_id':
        return true
    }

    return false
  }

  get title() {
    return (
      (this.tmpEntity as Artists).artist_name ??
      (this.tmpEntity as Album).album_name ??
      (this.tmpEntity as Playlist).playlist_name
    )
  }

  get filterFields() {
    const fields = []
    if (this.tmpEntity) {
      for (const key of Object.keys(this.tmpEntity)) {
        if (
          key === 'artist_extra_info' &&
          'artist_extra_info' in this.tmpEntity &&
          this.tmpEntity['artist_extra_info']
        ) {
          for (const [extraInfoKey, val] of Object.entries(this.tmpEntity['artist_extra_info'])) {
            if (extraInfoKey === 'youtube' || extraInfoKey === 'spotify') {
              fields.push(...Object.keys(val).map((val) => `${key}.${extraInfoKey}.${val}`))
            }
          }
          continue
        }

        switch (key as keyof (Album & Artists & Playlist)) {
          case 'album_name':
          case 'artist_name':
          case 'playlist_name':
          case 'album_song_count':
          case 'artist_song_count':
          case 'playlist_song_count':
          case 'album_coverPath_high':
          case 'album_coverPath_low':
          case 'artist_coverPath':
          case 'playlist_coverPath':
            break
          default:
            fields.push(key)
        }
      }
    }
    return fields
  }

  close() {
    this.tmpEntity = null
    this.entity = null
    this.$bvModal.hide(this.id)
  }

  async save() {
    if (this.tmpEntity) {
      if ((this.tmpEntity as Artists).artist_id) {
        window.DBUtils.updateArtist(convertProxy(this.tmpEntity as Artists))
      }

      if ((this.tmpEntity as Album).album_id) {
        window.DBUtils.updateAlbum(convertProxy(this.tmpEntity as Album))
      }

      if ((this.tmpEntity as Playlist).playlist_id) {
        window.DBUtils.updatePlaylist(convertProxy(this.tmpEntity as Playlist))
      }

      this.mergeIntoOriginal()
      this.close()
    }
  }

  private mergeIntoOriginal() {
    if (this.tmpEntity && this.entity) {
      for (const key of Object.keys(this.entity)) {
        this.entity[key as keyof (Album | Artists | Playlist)] =
          this.tmpEntity[key as keyof (Album | Artists | Playlist)]
      }
    }
  }

  async changeEntityCover() {
    if (this.tmpEntity) {
      const file = await window.WindowUtils.openFileBrowser(true, true, [
        {
          name: 'Image',
          extensions: ['png', 'jpg', 'jpeg', 'gif', 'svg']
        }
      ])

      if (!file.canceled && file.filePaths.length > 0) {
        if ((this.tmpEntity as Artists)['artist_id']) {
          ;(this.tmpEntity as Artists).artist_coverPath = file.filePaths[0]
        }

        if ((this.tmpEntity as Album)['album_id']) {
          ;(this.tmpEntity as Album).album_coverPath_high = file.filePaths[0]
          ;(this.tmpEntity as Album).album_coverPath_low = file.filePaths[0]
        }

        if ((this.tmpEntity as Playlist)['playlist_id']) {
          ;(this.tmpEntity as Playlist).playlist_coverPath = file.filePaths[0]
        }
      }
    }
  }

  changeEntityField(field: string, value: Event) {
    if (this.tmpEntity) {
      dotIndex(this.tmpEntity as Record<string, unknown>, field, value)
    }
  }

  changeTitle(value: never) {
    if (this.tmpEntity) {
      if ((this.tmpEntity as Artists).artist_id) {
        ;(this.tmpEntity as Artists).artist_name = value
      }

      if ((this.tmpEntity as Album).album_id) {
        ;(this.tmpEntity as Album).album_name = value
      }

      if ((this.tmpEntity as Playlist).playlist_id) {
        ;(this.tmpEntity as Playlist).playlist_name = value
      }
    }
  }

  getParsedFieldTitle(field: string) {
    switch (field.toLowerCase()) {
      case 'artist_extra_info.spotify.artist_id':
        return 'spotify artist id'
      case 'artist_extra_info.youtube.channel_id':
        return 'youtube channel id'
      default:
        return field.replaceAll('_', ' ')
    }
  }

  mounted() {
    bus.on(EventBus.SHOW_ENTITY_INFO_MODAL, (entity: Artists | Album | Playlist) => {
      this.forceEmptyImg = false
      this.entity = entity
      this.tmpEntity = JSON.parse(JSON.stringify(entity))
      if (this.entity) {
        this.$bvModal.show(this.id)
      }
    })
  }
}
</script>

<style lang="sass" scoped>
.field-title
  text-transform: capitalize
  font-weight: 700

.field-col
  margin-bottom: 13px

.field-value
  font-size: 14px
  font-weight: 400
  width: auto
  margin-left: 8px

.modal-content-container
  max-height: 600px
  height: 300px
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

.input-style
  background-color: transparent !important
  background: transparent !important
  border: none !important
  border-radius: 0 !important
  color: var(--textPrimary) !important
  height: inherit
  padding: 0 !important

.editable
  border-bottom: transparent 1px solid !important
  &:focus
    border-bottom: var(--accent) 1px solid !important

.details
  margin-left: 30px

.edit-button
  position: absolute
  width: 157px
  height: 157px
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
