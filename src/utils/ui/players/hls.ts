/*
 *  dash.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Player } from './player'
import Hls from 'hls.js'

export class HLSPlayer extends Player {
  private playerInstance!: Hls
  private htmlElement!: HTMLVideoElement

  private track: MediaElementAudioSourceNode | undefined
  private context: AudioContext | undefined

  public provides(): PlayerTypes[] {
    return ['HLS']
  }

  get key() {
    return 'HLS'
  }

  public async canPlay(): Promise<boolean> {
    return true
  }

  protected async _initialize(element: HTMLVideoElement) {
    this.htmlElement = element
    this.playerInstance = new Hls()
  }

  _load(src?: string, volume?: number, autoplay?: boolean): void {
    if (src) {
      this.playerInstance.loadSource(src)
      this.playerInstance.attachMedia(this.htmlElement)

      if (volume) this.volume = volume
      if (autoplay) this.htmlElement.play()
    } else {
      this.stop()
    }
  }

  async _play(): Promise<void> {
    this.htmlElement?.play()
  }

  _pause(): void {
    this.htmlElement?.pause()
  }

  _stop(): void {
    this.playerInstance.stopLoad()
    this.htmlElement.pause()
  }

  get currentTime(): number {
    return this.htmlElement.currentTime
  }

  set currentTime(time: number) {
    this.htmlElement.currentTime = time
  }

  get volume(): number {
    return this.htmlElement.volume * 100
  }

  set volume(volume: number) {
    this.htmlElement.volume = volume / 100
  }

  protected listenOnEnded(callback: () => void): void {
    this.htmlElement.onended = callback
  }

  protected listenOnTimeUpdate(callback: (time: number) => void): void {
    this.htmlElement.ontimeupdate = () => callback(this.currentTime)
  }

  protected listenOnLoad(callback: () => void): void {
    this.htmlElement.onload = callback
    this.htmlElement.onloadeddata = callback
  }

  protected listenOnError(callback: (err: Error) => void): void {
    this.htmlElement.onerror = (event, source, line, col, err) => err && callback && callback(err)
  }

  private listeners: { [key: string]: () => void } = {}

  protected listenOnStateChange(callback: (state: PlayerState) => void): void {
    const play = () => callback('PLAYING')
    const pause = () => callback('PAUSED')
    const stop = () => callback('STOPPED')

    this.htmlElement.addEventListener('play', play)
    this.htmlElement.addEventListener('pause', pause)
    this.htmlElement.addEventListener('ended', stop)

    this.listeners['play'] = play
    this.listeners['pause'] = pause
    this.listeners['ended'] = stop
  }

  protected listenOnBuffer(callback: () => void): void {
    this.htmlElement.onloadstart = callback
  }

  removeAllListeners(): void {
    if (this.htmlElement) {
      this.htmlElement.onended = null
      this.htmlElement.ontimeupdate = null
      for (const [key, value] of Object.entries(this.listeners)) {
        this.htmlElement.removeEventListener(key as keyof HTMLMediaElementEventMap, value)
      }
    }
  }

  createAudioContext() {
    if (!this.context) {
      this.context = new AudioContext()
      this.track = this.context.createMediaElementSource(this.htmlElement)
      this.track.connect(this.context.destination)
    }
    return this.context
  }
  connectAudioContextNode(node: AudioNode): void {
    if (this.context && this.track) {
      this.track.connect(node).connect(this.context.destination)
    }
  }

  preload(): void {
    return
  }
}
