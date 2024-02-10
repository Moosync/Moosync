/*
 *  local.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Player } from './player'
import { wrapHTMLAudioElement as wrapPlayerInstance } from './wrapper/htmlAudioElement'

export class LocalPlayer extends Player {
  playerInstance!: CustomAudioInstance
  private track: MediaElementAudioSourceNode | undefined
  private context: AudioContext | undefined

  public provides(): PlayerTypes[] {
    return ['LOCAL', 'URL']
  }

  get key() {
    return 'LOCAL'
  }

  // eslint-disable-next-line @typescript-eslint/no-unused-vars
  public async canPlay(_: string): Promise<boolean> {
    return true
  }

  // TODO: Typecheck playerInstance somehow
  protected async _initialize(playerInstance: unknown): Promise<void> {
    if (
      playerInstance &&
      (playerInstance instanceof HTMLAudioElement || (playerInstance as CustomAudioInstance).isCustomAudio)
    ) {
      this.playerInstance = wrapPlayerInstance(playerInstance as CustomAudioInstance)
    } else {
      throw new Error('passed player is not an instance of CustomAudioInstance')
    }
  }

  protected async _load(src?: string, volume?: number, autoplay?: boolean): Promise<void> {
    if (src) {
      this.playerInstance.setSrc(src, autoplay)
    }
    if (volume) this.volume = volume
  }

  protected async _play(): Promise<void> {
    if (this.playerInstance.paused) await this.playerInstance?.play()
  }

  protected _pause(): void {
    if (!this.playerInstance.paused) this.playerInstance?.pause()
  }

  protected _stop(): void {
    this.playerInstance.stop()
  }

  get currentTime(): number {
    return this.playerInstance.currentTime
  }

  set currentTime(time: number) {
    this.playerInstance.currentTime = time
  }

  get volume(): number {
    return this.playerInstance.volume * 100
  }

  set volume(volume: number) {
    this.playerInstance.volume = volume / 100
  }

  protected listenOnEnded(callback: () => void): void {
    this.playerInstance.onended = callback
  }

  protected listenOnTimeUpdate(callback: (time: number) => void): void {
    this.playerInstance.ontimeupdate = () => callback(this.currentTime)
  }

  protected listenOnLoad(callback: () => void): void {
    this.playerInstance.onload = callback
    this.playerInstance.onloadeddata = callback
  }

  protected listenOnError(callback: (err: Error) => void): void {
    this.playerInstance.onerror = (event, source, line, col, err) => {
      const finalErr = err ?? ((event as ErrorEvent).target as HTMLAudioElement).error
      console.error('error', event, source, line, col, finalErr)
      if (callback) {
        if (finalErr) {
          callback(finalErr as Error)
        } else {
          if (typeof event === 'string') {
            callback(new Error(event))
          } else if (event instanceof Event) {
            callback(new Error(`${event.type}: loading source`))
          }
        }
      }
    }
  }

  protected listenOnStateChange(callback: (state: PlayerState) => void): void {
    const play = () => callback('PLAYING')
    const pause = () => callback('PAUSED')
    const stop = () => callback('STOPPED')

    this.playerInstance.addEventListener('play', play)
    this.playerInstance.addEventListener('pause', pause)
    this.playerInstance.addEventListener('ended', stop)
  }

  protected listenOnBuffer(callback: () => void): void {
    this.playerInstance.onloadstart = callback
  }

  removeAllListeners(): void {
    if (this.playerInstance) {
      this.playerInstance.onended = null
      this.playerInstance.ontimeupdate = null
      this.playerInstance.onload = null
      this.playerInstance.onloadeddata = null
      this.playerInstance.onloadstart = null
      for (const [key, value] of Object.entries(this.playerInstance.listeners ?? {})) {
        this.playerInstance.removeEventListener(key as keyof HTMLMediaElementEventMap, value)
      }
    }
  }

  createAudioContext() {
    if (!this.context && this.playerInstance instanceof HTMLAudioElement) {
      this.context = new AudioContext()
      this.track = this.context.createMediaElementSource(this.playerInstance)
      this.track.connect(this.context.destination)
    }

    return this.context
  }

  connectAudioContextNode(node: AudioNode): void {
    if (this.context && this.track) {
      this.track.connect(node).connect(this.context.destination)
    }
  }

  // Hoping electron will cache the audio
  preload(src: string) {
    try {
      new URL(src)
      const audio = new Audio()
      audio.preload = 'auto'
      audio.volume = 0
      audio.src = src
      audio.load()
      audio.play()
    } catch (e) {
      console.debug('Not a valid URL, not preloading')
    }
  }
}
