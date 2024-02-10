/*
 *  syncState.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { action, mutation } from 'vuex-class-component'

import { VuexModule } from './module'
import { stripSong, toRemoteSong } from '@/utils/common'
import { v1 } from 'uuid'

export enum PeerMode {
  WATCHER = 0,
  BROADCASTER = 1,
  UNDEFINED = 2,
}

class Queue implements GenericQueue<RemoteSong> {
  data: QueueData<RemoteSong> = {}
  order: QueueOrder = []
  index = -1
}

export class SyncStore extends VuexModule.With({ namespaced: 'sync' }) {
  mode: PeerMode = PeerMode.UNDEFINED
  currentSong: RemoteSong | null | undefined = null
  currentCover: string | undefined
  currentFetchSong = ''
  roomID = ''
  _socketID = ''

  private songQueue = new Queue()

  get socketID() {
    return this._socketID
  }

  set socketID(id: string) {
    this._socketID = id
  }

  get queueOrder() {
    return this.songQueue.order
  }

  set queueOrder(order: QueueOrder) {
    this.songQueue.order = order
  }

  get queueIndex() {
    return this.songQueue.index
  }

  set queueIndex(value: number) {
    this.songQueue.index = value
  }

  get queueData() {
    return this.songQueue.data
  }

  set queueData(data: QueueData<RemoteSong>) {
    this.songQueue.data = data
  }

  @mutation
  private _setSongQueueOrder(order: QueueOrder) {
    this.songQueue.order = order
  }

  @mutation
  private clearCurrentSong() {
    this.currentSong = null
  }

  @action
  async setQueueOrder(order: QueueOrder) {
    if (order.length === 0) {
      this.clearCurrentSong()
    }

    const oldOrder = this.songQueue.order
    this._setSongQueueOrder(order)

    const diff = oldOrder.filter((x) => !order.includes(x))
    for (const item of diff) {
      this.removeFromQueueData(item)
    }
  }

  get queueTop(): RemoteSong | null | undefined {
    if (this.songQueue.index > -1 && this.songQueue.data) {
      const songID = this.songQueue.order[this.songQueue.index]
      if (songID) return this.songQueue.data[songID.songID]
    }
    return null
  }

  @mutation
  private addSong(item: Song[]) {
    for (const s of item) {
      const song = stripSong(toRemoteSong(s, this._socketID))
      if (song && !this.songQueue.data[song._id]) {
        this.songQueue.data[song._id] = song
      }
    }
  }

  @mutation
  private removeFromQueueData(orderData: QueueOrder[0] | undefined) {
    if (orderData) {
      if (this.songQueue.order.findIndex((val) => val.songID === orderData.songID) === -1) {
        delete this.songQueue.data[orderData.songID]
      }
    }
  }

  @mutation
  private removeFromQueue(index: number) {
    if (index > -1) {
      this.songQueue.order.splice(index, 1)
      if (this.songQueue.order.length === 0) {
        this.currentSong = null
      }
    }
  }

  @action
  public async pop(index: number) {
    if (index > -1) {
      this.removeFromQueue(index)
      this.removeFromQueueData(this.songQueue.order[index])

      if (this.songQueue.index === index) {
        await this.loadSong(this.queueTop)
      }
    }
  }

  @mutation
  private addInSongQueue(item: Song[]) {
    this.songQueue.order.push(
      ...item.map((obj) => {
        return { id: v1(), songID: obj._id }
      }),
    )
  }

  @action
  async pushInQueue(payload: { item: Song[]; top: boolean; skipImmediate: boolean }) {
    if (payload.item.length > 0) {
      const currentSongExists = !!this.currentSong
      if (!currentSongExists) {
        // Add first item immediately to start playing
        this.addSong([payload.item[0]])
        payload.top ? this.addInQueueTop([payload.item[0]]) : this.addInSongQueue([payload.item[0]])
        payload.item.splice(0, 1)
        await this.nextSong()
      }

      this.addSong(payload.item)
      payload.top ? this.addInQueueTop(payload.item) : this.addInSongQueue(payload.item)
      if (payload.skipImmediate) await this.nextSong()
    }
  }

  @mutation
  private addInQueueTop(item: Song[]) {
    this.songQueue.order.splice(
      this.songQueue.index + 1,
      0,
      ...item.map((obj) => {
        return { id: v1(), songID: obj._id }
      }),
    )
  }

  @mutation
  private incrementQueue() {
    if (this.songQueue.index < this.songQueue.order.length - 1) this.songQueue.index += 1
    else this.songQueue.index = 0
  }

  @action
  async nextSong() {
    this.incrementQueue()
    this.loadSong(this.queueTop)
  }

  @mutation
  private decrementQueue() {
    if (this.songQueue.index > 0) this.songQueue.index -= 1
    else this.songQueue.index = this.songQueue.order.length - 1
  }

  @action
  async prevSong() {
    this.decrementQueue()
    this.loadSong(this.queueTop)
  }

  @mutation loadSong(song: RemoteSong | null | undefined) {
    this.currentSong = song
  }

  @mutation
  private moveIndexTo(index: number) {
    if (index >= 0) this.songQueue.index = index
  }

  @action async playQueueSong(index: number) {
    this.moveIndexTo(index)
    this.loadSong(this.queueTop)
  }

  @mutation
  setMode(mode: PeerMode) {
    this.mode = mode
  }

  @mutation
  setRoom(id: string) {
    this.roomID = id
  }

  @mutation
  setSong(song: RemoteSong | null | undefined) {
    this.currentSong = song
  }

  @mutation
  setCover(cover: string | undefined) {
    this.currentCover = cover
  }

  @mutation
  setCurrentFetchSong(id: string) {
    this.currentFetchSong = id
  }

  @mutation
  setSongIndex({ oldIndex, newIndex, ignoreMove }: { oldIndex: number; newIndex: number; ignoreMove: boolean }) {
    if (newIndex < 0) {
      newIndex = this.songQueue.order.length - -newIndex
    }

    if (newIndex >= this.songQueue.order.length) {
      newIndex = this.songQueue.order.length - 1
    }

    if (!ignoreMove) {
      const data = this.songQueue.order[oldIndex]
      this.songQueue.order.splice(oldIndex, 1)
      this.songQueue.order.splice(newIndex, 0, data)
    }

    if (oldIndex === this.songQueue.index) {
      this.songQueue.index = newIndex
      return
    }

    if (oldIndex < this.songQueue.index) {
      if (newIndex >= this.songQueue.index) this.songQueue.index -= 1
    } else if (oldIndex > this.songQueue.index) {
      if (newIndex <= this.songQueue.index) this.songQueue.index += 1
    }
  }
}
