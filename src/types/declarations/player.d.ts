/*
 *  player.d.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

type AudioType = 'STREAMING' | 'LOCAL'

type PlayerState = 'PLAYING' | 'PAUSED' | 'STOPPED'

interface SongQueue {
  data: QueueData<Song>
  order: QueueOrder
  index: number
}

type playlistInfo = { [key: string]: string }

type QueueOrder = { id: string; songID: string }[]
type QueueData<T> = { [id: string]: T }

interface GenericQueue<T> {
  data: QueueData<T>
  order: QueueOrder
  index: number
}

interface CustomAudioInstance {
  currentTime: number
  volume: number
  paused: boolean
  srcObject: unknown

  isCustomAudio: boolean

  listeners: Record<string, never>

  pause(): void
  play(): Promise<void>
  stop(): void

  setSrc(src: string, autoPlay?: boolean): void

  onended: ((this: GlobalEventHandlers, ev: Event) => void) | null
  ontimeupdate: ((this: GlobalEventHandlers, ev: Event) => void) | null
  onload: ((this: GlobalEventHandlers, ev: Event) => void) | null
  onloadeddata: ((this: GlobalEventHandlers, ev: Event) => void) | null
  onerror: OnErrorEventHandler
  onloadstart: ((this: GlobalEventHandlers, ev: Event) => void) | null

  removeAttribute(str: string): void
  addEventListener(ev: string, callback: unknown)
  removeEventListener(ev: string, callback: unknown)

  dispatchEvent(ev: Event)
}
