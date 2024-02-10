/*
 *  PlayerControls.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { RepeatState, VolumePersistMode } from '@/utils/commonConstants'
import { Component, Vue } from 'vue-facing-decorator'

import { vxm } from '@/mainWindow/store'
import { PeerMode } from '@/mainWindow/store/syncState'
import { toast } from 'vue3-toastify'
import { Player } from '../players/player'

const maxp = 100

@Component
export default class PlayerControls extends Vue {
  get playerState() {
    return vxm.player.playerState
  }

  get isSyncing() {
    return vxm.sync.mode !== PeerMode.UNDEFINED
  }

  public async nextSong() {
    if (this.isSyncing) await vxm.sync.nextSong()
    else await vxm.player.nextSong()
  }

  public prevSong() {
    if (this.isSyncing) vxm.sync.prevSong()
    else vxm.player.prevSong()
  }

  public showQueueSongsToast(length: number) {
    if (length > 0) toast(this.$t('toasts.queued', length))
  }

  public async queueSong(songs: Song[], showToast = true) {
    if (this.isSyncing) {
      await vxm.sync.pushInQueue({ item: songs, top: false, skipImmediate: false })
    } else {
      await vxm.player.pushInQueue({ item: songs, top: false, skipImmediate: false })
    }

    showToast && this.showQueueSongsToast(songs.length)
  }

  public async playTop(songs: Song[]) {
    if (this.isSyncing) {
      await vxm.sync.pushInQueue({ item: songs.slice(), top: true, skipImmediate: vxm.sync.queueOrder.length > 0 })
    } else {
      await vxm.player.pushInQueue({ item: songs.slice(), top: true, skipImmediate: vxm.player.queueOrder.length > 0 })
    }

    if (!this.isSyncing) vxm.player.playAfterLoad = true

    this.showQueueSongsToast(songs.length)
    this.play()
  }

  public async playNext(songs: Song[]) {
    if (this.isSyncing) {
      await vxm.sync.pushInQueue({ item: songs.slice(), top: true, skipImmediate: false })
    } else {
      await vxm.player.pushInQueue({ item: songs.slice(), top: true, skipImmediate: false })
    }

    if (!this.isSyncing) vxm.player.playAfterLoad = true

    this.showQueueSongsToast(songs.length)
  }

  public clearQueue() {
    vxm.player.clearQueue()
  }

  public play() {
    vxm.player.playerState = 'PLAYING'
  }

  public pause() {
    vxm.player.playerState = 'PAUSED'
  }

  public togglePlay() {
    if (!vxm.player.loading) {
      vxm.player.playerState = vxm.player.playerState === 'PLAYING' ? 'PAUSED' : 'PLAYING'
    }
  }

  public shuffle() {
    vxm.themes.queueSortBy = undefined
    vxm.player.shuffle()
    toast(this.$t('toasts.shuffled'), {
      autoClose: 1000,
    })
  }

  public togglePlayerState() {
    if (this.playerState === 'PAUSED' || this.playerState === 'STOPPED') {
      vxm.player.playerState = 'PLAYING'
    } else {
      vxm.player.playerState = 'PAUSED'
    }
  }

  public stop() {
    vxm.player.playerState = 'STOPPED'
  }

  public playFromQueue(index: number) {
    if (this.isSyncing) {
      vxm.sync.playQueueSong(index)
    } else {
      vxm.player.playQueueSong(index)
    }
  }

  public async removeFromQueue(index: number) {
    await vxm.player.pop(index)
  }

  public setSongIndex(oldIndex: number, newIndex: number) {
    vxm.player.setSongIndex({ oldIndex, newIndex, ignoreMove: false })
  }

  get repeat() {
    return vxm.player.Repeat
  }

  set repeat(val: RepeatState) {
    vxm.player.Repeat = val
  }

  public toggleRepeat() {
    switch (this.repeat) {
      case RepeatState.ONCE:
        this.repeat = RepeatState.ALWAYS
        break
      case RepeatState.ALWAYS:
        this.repeat = RepeatState.DISABLED
        break
      default:
        this.repeat = RepeatState.ONCE
        break
    }
  }

  private oldVolume = 50

  get volume() {
    if (vxm.player.volume === 0) {
      return 0
    }

    const maxv = Math.log(this.clamp)
    const scale = maxv / maxp

    const volume = Math.min(Math.max(vxm.player.volume, 0), 100)

    if (volume > 0) {
      return Math.log(volume) / scale
    }

    return volume
  }

  set volume(value: number) {
    let parsedVolume = value

    const maxv = Math.log(this.clamp)
    const scale = maxv / maxp

    if (value > 0) {
      parsedVolume = Math.exp(scale * value)
    }

    vxm.player.volume = parsedVolume
    if (parsedVolume !== 0) {
      this.oldVolume = parsedVolume
    }
  }

  private get clamp() {
    if (vxm.player.volumeMode === VolumePersistMode.CLAMP_MAP) {
      const currentSong = vxm.player.currentSong
      if (currentSong) {
        return (
          vxm.player.clampMap[
            currentSong?.type.toLowerCase() ?? currentSong?.providerExtension?.replaceAll('.', '_').toLowerCase()
          ]?.clamp ?? 100
        )
      }
    }

    return 100
  }

  public muteToggle() {
    if (this.volume !== 0) {
      this.mute()
    } else {
      this.unmute()
    }
  }

  public mute() {
    this.oldVolume = this.volume
    this.volume = 0
  }

  public unmute() {
    this.volume = this.oldVolume
  }

  public findPlayer(canPlay: PlayerTypes, blacklist: string[] = []) {
    let lowest: [Player | undefined, number] = [undefined, vxm.playerRepo.allPlayers.length]
    for (const p of vxm.playerRepo.allPlayers) {
      const index = p.provides().indexOf(canPlay)
      if (index >= 0 && index < lowest[1] && !blacklist.includes(p.key)) {
        lowest = [p, index]
      }
    }

    return lowest[0]
  }

  public clearAllListeners() {
    for (const p of vxm.playerRepo.allPlayers) {
      p.removeAllListeners()
    }
  }
}
