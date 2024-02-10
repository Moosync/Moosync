<!-- 
  EditText.vue is a part of Moosync.
  
  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <b-container v-if="prefKey" fluid class="w-100">
    <b-row>
      <PreferenceHeader v-if="title" :title="title" :tooltip="tooltip" />
      <b-col cols="auto" class="new-keybind ml-4">
        <div @click="addKeybind">Add Hotkey</div>
      </b-col>
    </b-row>

    <b-row no-gutters class="w-100 mt-2 d-flex keybind-row">
      <b-row no-gutters class="w-100 mb-2">
        <b-col>
          <div>Actions</div>
        </b-col>
        <b-col class="keybind-title">
          <div>Keybinds</div>
        </b-col>
        <b-col>
          <div></div>
        </b-col>
      </b-row>
      <b-row no-gutters class="w-100 actions-row mt-2" v-for="(defined, index) of definedActions" :key="defined.value">
        <b-col>
          <b-row no-gutters>
            <b-dropdown block :text="getActiveTitle(index)" toggle-class="dropdown-button h-100" class="w-100">
              <b-dropdown-item v-for="action in getFilteredDropdownList()" :key="action.key"
                @click="setSelectedAction(index, action.key)">{{ action.title }}
              </b-dropdown-item>
            </b-dropdown>
          </b-row>
        </b-col>
        <b-col @click="toggleKeybindListener(index)">
          <div class="key-input" :style="{ color: getKeybindColor(index) }">{{ getKeybind(index) }}</div>
        </b-col>
        <b-col align-self="center" class="d-flex justify-content-end">
          <div class="cross-icon" @click="removeKeybind(index)">
            <CrossIcon color="#E62017" />
          </div>
        </b-col>
        <div class="divider"></div>
      </b-row>
    </b-row>
  </b-container>
</template>

<script lang="ts">
import { Component, Prop } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import PreferenceHeader from './PreferenceHeader.vue'
import { ExtensionPreferenceMixin } from '../mixins/extensionPreferenceMixin'
import { HotkeyEvents } from '@/utils/commonConstants'
import CrossIcon from '@/icons/CrossIcon.vue'
@Component({
  components: {
    PreferenceHeader,
    CrossIcon
  }
})
export default class HotkeyGroup extends mixins(ExtensionPreferenceMixin) {
  declare value: HotkeyPair[]

  shouldMergeDefaultValues = false

  get definedActions() {
    return this.value
  }

  private get HotKeyEventsExtras(): Record<HotkeyEvents, { title: string }> {
    return {
      [HotkeyEvents.PLAY]: {
        title: this.$t('hotkeys.play'),
      },
      [HotkeyEvents.PAUSE]: {
        title: this.$t('hotkeys.pause'),
      },
      [HotkeyEvents.PLAY_TOGGLE]: {
        title: this.$t('hotkeys.play_toggle'),
      },
      [HotkeyEvents.MUTE_ACTIVE]: {
        title: this.$t('hotkeys.mute_active'),
      },
      [HotkeyEvents.MUTE_INACTIVE]: {
        title: this.$t('hotkeys.mute_inactive'),
      },
      [HotkeyEvents.MUTE_TOGGLE]: {
        title: this.$t('hotkeys.mute_toggle'),
      },
      [HotkeyEvents.VOLUME_INC]: {
        title: this.$t('hotkeys.volume_inc'),
      },
      [HotkeyEvents.VOLUME_DEC]: {
        title: this.$t('hotkeys.volume_dec'),
      },
      [HotkeyEvents.REPEAT_ACTIVE]: {
        title: this.$t('hotkeys.repeat_active'),
      },
      [HotkeyEvents.REPEAT_INACTIVE]: {
        title: this.$t('hotkeys.repeat_inactive'),
      },
      [HotkeyEvents.REPEAT_TOGGLE]: {
        title: this.$t('hotkeys.repeat_toggle'),
      },
      [HotkeyEvents.QUEUE_OPEN]: {
        title: this.$t('hotkeys.queue_open'),
      },
      [HotkeyEvents.QUEUE_CLOSE]: {
        title: this.$t('hotkeys.queue_close'),
      },
      [HotkeyEvents.QUEUE_TOGGLE]: {
        title: this.$t('hotkeys.queue_toggle'),
      },
      [HotkeyEvents.RELOAD_PAGE]: {
        title: this.$t('hotkeys.reload_page'),
      },
      [HotkeyEvents.DEVTOOLS_TOGGLE]: {
        title: this.$t('hotkeys.devtools'),
      },
      [HotkeyEvents.HELP]: {
        title: this.$t('hotkeys.help'),
      },
      [HotkeyEvents.FULLSCREEN]: {
        title: this.$t('hotkeys.fullscreen'),
      },
      [HotkeyEvents.NEXT_SONG]: {
        title: this.$t('hotkeys.next_song'),
      },
      [HotkeyEvents.PREV_SONG]: {
        title: this.$t('hotkeys.prev_song'),
      },
      [HotkeyEvents.SEEK_FORWARD]: {
        title: this.$t('hotkeys.seek_forward'),
      },
      [HotkeyEvents.SEEK_BACKWARDS]: {
        title: this.$t('hotkeys.seek_backwards'),
      },
      [HotkeyEvents.SEEK_0]: {
        title: this.$t('hotkeys.seek_0'),
      },
      [HotkeyEvents.SEEK_1]: {
        title: this.$t('hotkeys.seek_1'),
      },
      [HotkeyEvents.SEEK_2]: {
        title: this.$t('hotkeys.seek_2'),
      },
      [HotkeyEvents.SEEK_3]: {
        title: this.$t('hotkeys.seek_3'),
      },
      [HotkeyEvents.SEEK_4]: {
        title: this.$t('hotkeys.seek_4'),
      },
      [HotkeyEvents.SEEK_5]: {
        title: this.$t('hotkeys.seek_5'),
      },
      [HotkeyEvents.SEEK_6]: {
        title: this.$t('hotkeys.seek_6'),
      },
      [HotkeyEvents.SEEK_7]: {
        title: this.$t('hotkeys.seek_7'),
      },
      [HotkeyEvents.SEEK_8]: {
        title: this.$t('hotkeys.seek_8'),
      },
      [HotkeyEvents.SEEK_9]: {
        title: this.$t('hotkeys.seek_9'),
      },
    }
  }


  getActiveTitle(index: number) {
    if (this.value) {
      this.value[index]?.value
      return this.HotKeyEventsExtras[this.value[index]?.value ?? 0]?.title
    }
  }

  getHotkeyValue() {
    return Object.values(HotkeyEvents).filter(
      (key) => typeof key === 'number' && this.value?.findIndex((val) => val.value === key) === -1
    ) as HotkeyEvents[]
  }

  getFilteredDropdownList() {
    return this.getHotkeyValue().map((val: HotkeyEvents) => {
      return { title: this.HotKeyEventsExtras[val].title, key: val }
    })
  }

  getKeybind(index: number) {
    if (this.value) {
      const combo = this.value[index]?.key?.at(0) ?? []
      return combo.length > 0 ? combo.map((val) => this.sanitizeKeys(val)).join(' + ') : 'Unassigned'
    }
  }

  setSelectedAction(index: number, item: HotkeyEvents) {
    this.value?.splice(index, 1, {
      key: this.value[index].key,
      value: item
    })
    this.onInputChange(this.value)
  }

  setSelectedKeybind(index: number, combo: HotkeyPair['key']) {
    this.value?.splice(index, 1, {
      key: combo,
      value: this.value[index].value
    })
    this.onInputChange(this.value)
  }

  private abortController: AbortController | null = null
  private listeningIndex = -1

  private sanitizeKeys(input: string) {
    input = input
      .replace('Key', '')
      .replaceAll('Numpad', 'Numpad ')
      .replaceAll('Subtract', ' -')
      .replaceAll('Minus', '-')
      .replaceAll('Equal', '=')
      .replaceAll('Add', ' +')
      .replaceAll('Multiply', ' *')
      .replaceAll('Divide', ' /')
      .replaceAll('Decimal', ' .')
      .replaceAll('Left', '')
      .replaceAll('Control', 'CTRL')
      .replace(/([a-z])([A-Z])/g, '$1 $2')

    const right = input.indexOf('Right')
    if (right != -1) {
      input = 'Right ' + input.substring(0, right)
    }

    return input
  }

  getKeybindColor(index: number) {
    if (index === this.listeningIndex) {
      return 'var(--accent)'
    }

    return 'var(--textPrimary)'
  }

  toggleKeybindListener(index: number) {
    console.log('here', this.abortController)
    if (this.listeningIndex !== -1) {
      this.stopListeningKeybind()
    } else {
      this.startListeningKeybind(index)
    }
  }

  private keyComboMap: Record<string, boolean> = {}

  stopListeningKeybind(e?: KeyboardEvent | MouseEvent) {
    e?.preventDefault()
    this.abortController?.abort()
    this.abortController = null
    this.listeningIndex = -1
    this.keyComboMap = {}
  }

  startListeningKeybind(index: number) {
    this.abortController = new AbortController()
    this.listeningIndex = index

    document.addEventListener(
      'keydown',
      (e: KeyboardEvent) => {
        e.preventDefault()
        this.keyComboMap[e.code] = true
        this.setSelectedKeybind(index, [Object.keys(this.keyComboMap)])
      },
      { signal: this.abortController?.signal }
    )

    document.addEventListener(
      'mousedown',
      (e: MouseEvent) => {
        if (e.button == 0 || e.button == 2) {
          return
        }

        e.stopPropagation()
        this.keyComboMap[`Mouse${e.button}`] = true
        this.setSelectedKeybind(index, [Object.keys(this.keyComboMap)])
      },
      { signal: this.abortController?.signal }
    )

    document.addEventListener('mouseup', this.stopListeningKeybind.bind(this), { signal: this.abortController?.signal })
    document.addEventListener('keyup', this.stopListeningKeybind.bind(this), { signal: this.abortController?.signal })
  }

  removeKeybind(index: number) {
    this.value?.splice(index, 1)
    this.onInputChange(this.value)
  }

  addKeybind() {
    this.value?.push({
      key: [],
      value: 0
    })
  }

  @Prop()
  title!: string

  @Prop()
  tooltip!: string
}
</script>

<style lang="sass" scoped>
.title
  font-size: 26px

.keybind-row
  text-align: left

.background
  background-color: var(--tertiary)

.actions-row
  margin-right: 15px

.divider
  height: 1px
  width: 100%
  background-color: var(--divider)
  margin-top: 15px
  margin-bottom: 15px

.key-input
  margin-left: 30px
  background-color: var(--tertiary)
  width: fit-content
  padding: 5px 15px 5px 15px
  border-radius: 8px
  transition: 0.2s color ease-in-out

.cross-icon
  width: 15px
  margin-right: 10px

.new-keybind
  font-size: 16px
  color: var(--accent)
  &:hover
    cursor: pointer

.keybind-title
  margin-left: 37px
</style>
