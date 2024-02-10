/*
 *  KeyHandleMixin.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { HotkeyEvents, RepeatState, defaultKeybinds } from '@/utils/commonConstants'

import { bus } from '@/mainWindow/main'
import { vxm } from '@/mainWindow/store'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import { Component } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'

@Component
export default class KeyHandlerMixin extends mixins(PlayerControls) {
  private pressedKeys: Record<string, boolean> = {}

  private keyboardHotKeyMap: readonly HotkeyPair[] = []

  async created() {
    this.keyboardHotKeyMap = Object.freeze(
      (await window.PreferenceUtils.loadSelective('hotkeys', false, defaultKeybinds)) as HotkeyPair[],
    )

    window.PreferenceUtils.listenPreferenceChanged('hotkeys', true, (_, val: HotkeyPair[]) => {
      this.keyboardHotKeyMap = Object.freeze(val)
    })
  }

  private onlyRequiredKeysPressed(requiredKeys: string[]) {
    const pressedKeys = Object.keys(this.pressedKeys)
    for (const val of requiredKeys) {
      if (!pressedKeys.includes(val)) {
        return false
      }
    }

    for (const val of pressedKeys) {
      if (!requiredKeys.includes(val)) {
        return false
      }
    }

    return true
  }

  private performAction(action: HotkeyEvents) {
    switch (action) {
      case HotkeyEvents.PLAY:
        this.play()
        break
      case HotkeyEvents.PAUSE:
        this.pause()
        break
      case HotkeyEvents.PLAY_TOGGLE:
        this.togglePlay()
        break
      case HotkeyEvents.VOLUME_INC:
        this.volume += 5
        break
      case HotkeyEvents.VOLUME_DEC:
        this.volume -= 5
        break
      case HotkeyEvents.MUTE_TOGGLE:
        this.muteToggle()
        break
      case HotkeyEvents.MUTE_ACTIVE:
        this.mute()
        break
      case HotkeyEvents.MUTE_INACTIVE:
        this.unmute()
        break
      case HotkeyEvents.REPEAT_ACTIVE:
        this.repeat = RepeatState.ALWAYS
        break
      case HotkeyEvents.REPEAT_INACTIVE:
        this.repeat = RepeatState.DISABLED
        break
      case HotkeyEvents.REPEAT_TOGGLE:
        this.toggleRepeat()
        break
      case HotkeyEvents.RELOAD_PAGE:
        window.SpotifyPlayer.close()
          .then(() => window.RodioUtils.stop())
          .then(() => window.WindowUtils.handleReload())
          .then(() => location.reload())
        break
      case HotkeyEvents.DEVTOOLS_TOGGLE:
        window.WindowUtils.toggleDevTools(true)
        break
      case HotkeyEvents.HELP:
        window.WindowUtils.openExternal('https://github.com/Moosync/Moosync#readme')
        break
      case HotkeyEvents.QUEUE_CLOSE:
        bus.emit('onToggleSlider', false)
        break
      case HotkeyEvents.QUEUE_OPEN:
        bus.emit('onToggleSlider', true)
        break
      case HotkeyEvents.QUEUE_TOGGLE:
        bus.emit('onToggleSlider')
        break
      case HotkeyEvents.FULLSCREEN:
        window.WindowUtils.toggleFullscreen(true)
        break
      case HotkeyEvents.NEXT_SONG:
        this.nextSong()
        break
      case HotkeyEvents.PREV_SONG:
        this.prevSong()
        break
      case HotkeyEvents.SEEK_BACKWARDS:
        vxm.player.forceSeek = vxm.player.currentTime - 5
        break
      case HotkeyEvents.SEEK_FORWARD:
        vxm.player.forceSeek = vxm.player.currentTime + 5
        break
      default: {
        const duration = vxm.player.currentSong?.duration
        if (duration && (action >= 22 || action <= 31)) {
          const seekPercent = action % 22
          const seekTime = ((seekPercent * 10) / 100) * duration
          vxm.player.forceSeek = seekTime
        }
      }
    }
  }

  private isHotkeyActive() {
    for (const combinations of this.keyboardHotKeyMap) {
      for (const key of combinations.key) {
        if (this.onlyRequiredKeysPressed(key)) {
          this.performAction(combinations.value)
          this.pressedKeys = {}
        }
      }
    }
  }

  protected registerKeyboardHotkeys() {
    document.addEventListener('keydown', (e) => {
      if ((e.target as HTMLElement).tagName !== 'INPUT') {
        this.pressedKeys[e.code] = true
        this.isHotkeyActive()
      }
    })

    document.addEventListener('keyup', (e) => {
      delete this.pressedKeys[e.code]
    })

    document.addEventListener('mousedown', (e) => {
      if ((e.target as HTMLElement).tagName !== 'INPUT') {
        this.pressedKeys[`Mouse${e.button}`] = true
        this.isHotkeyActive()
      }
    })

    document.addEventListener('mouseup', (e) => {
      delete this.pressedKeys[`Mouse${e.button}`]
    })
  }
}
