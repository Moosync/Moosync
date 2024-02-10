/*
 *  dash.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Player } from './player'
import dashjs from 'dashjs'

export class DashPlayer extends Player {
  private playerInstance!: dashjs.MediaPlayerClass
  private htmlElement!: HTMLVideoElement

  private isAttachedView = false

  private track: MediaElementAudioSourceNode | undefined
  private context: AudioContext | undefined

  public provides(): PlayerTypes[] {
    return ['DASH']
  }

  get key() {
    return 'DASH'
  }

  public async canPlay(): Promise<boolean> {
    return true
  }

  protected async _initialize(element: HTMLVideoElement): Promise<void> {
    this.htmlElement = element
    this.playerInstance = dashjs.MediaPlayer().create()
  }

  _load(src?: string, volume?: number, autoplay?: boolean): void {
    if (src) {
      this.playerInstance.initialize(this.htmlElement, src, autoplay)
      this.isAttachedView = true
      if (volume) this.volume = volume
      if (autoplay) this.playerInstance.play()
    } else {
      this.stop()
    }
  }

  async _play(): Promise<void> {
    this.isAttachedView && this.playerInstance?.play()
  }

  _pause(): void {
    this.isAttachedView && this.playerInstance?.pause()
  }

  _stop(): void {
    this.isAttachedView = false
    this.playerInstance.reset()
  }

  get currentTime(): number {
    return this.isAttachedView ? this.playerInstance.time() : 0
  }

  set currentTime(time: number) {
    this.isAttachedView && this.playerInstance.seek(time)
  }

  get volume(): number {
    return this.isAttachedView ? this.playerInstance.getVolume() : 0
  }

  set volume(volume: number) {
    this.isAttachedView && this.playerInstance.setVolume(volume / 100)
  }

  protected listenOnEnded(callback: () => void): void {
    this.playerInstance.on('ended', callback)
  }

  protected listenOnTimeUpdate(callback: (time: number) => void): void {
    this.playerInstance.on('playbackTimeUpdated', (e) => callback(e.time ?? 0))
  }

  protected listenOnLoad(callback: () => void): void {
    this.playerInstance.on('bufferLoaded', callback)
    this.playerInstance.on('manifestLoaded', callback)
  }

  protected listenOnError(callback: (err: Error) => void): void {
    this.playerInstance.on('error', (e) => callback(new Error(JSON.stringify(e.error))))
    this.playerInstance.on('playbackError', (e) => callback(new Error(e.error.toString())))
  }

  private listeners: { [key: string]: () => void } = {}

  protected listenOnStateChange(callback: (state: PlayerState) => void): void {
    const play = () => callback('PLAYING')
    const pause = () => callback('PAUSED')
    const stop = () => callback('STOPPED')

    this.playerInstance.on('playbackStarted', play)
    this.playerInstance.on('playbackPaused', pause)
    this.playerInstance.on('playbackEnded', stop)

    this.listeners['playbackStarted'] = play
    this.listeners['playbackPaused'] = pause
    this.listeners['ended'] = stop
  }

  protected listenOnBuffer(callback: () => void): void {
    this.playerInstance.on('bufferStalled', callback)
  }

  removeAllListeners(): void {
    // TODO
  }

  createAudioContext() {
    if (!this.context) {
      this.context = new AudioContext()
      this.track = this.context.createMediaElementSource(this.playerInstance.getVideoElement())
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
