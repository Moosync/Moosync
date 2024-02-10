import { RodioEvents } from '@/utils/preload/ipc/constants'
import { Player } from './player'

export class RodioPlayer extends Player {
  private _volume = 1
  private _currentTime = 0

  private callbacks: Record<string, Function> = {}

  private registerListeners() {
    window.RodioUtils.listenEvents((event, ...args: unknown[]) => {
      this.callbacks[event](...args)
    })
  }

  protected async _initialize(config?: unknown): Promise<void> {
    this.registerListeners()
    await window.RodioUtils.initialize()
  }

  public provides(): PlayerTypes[] {
    return ['LOCAL']
  }

  get key() {
    return 'RODIO'
  }

  protected async _load(
    src?: string | undefined,
    volume?: number | undefined,
    autoplay?: boolean | undefined,
  ): Promise<void> {
    if (src) await window.RodioUtils.setSrc(src.replace('media://', ''))
    if (volume) await window.RodioUtils.setVolume(volume)
    if (autoplay) await window.RodioUtils.play()
  }

  protected async _play(): Promise<void> {
    await window.RodioUtils.play()
  }

  protected async _pause(): Promise<void> {
    await window.RodioUtils.pause()
  }

  protected async _stop(): Promise<void> {
    await window.RodioUtils.stop()
  }

  get currentTime(): number {
    return this._currentTime
  }

  set currentTime(time: number) {
    window.RodioUtils.seek(time * 1000)
  }

  get volume(): number {
    return this._volume
  }

  set volume(volume: number) {
    this._volume = volume
    window.RodioUtils.setVolume(volume)
  }
  protected listenOnEnded(callback: () => void): void {
    this.callbacks[RodioEvents.ON_ENDED] = callback
  }

  protected listenOnTimeUpdate(callback: (time: number) => void): void {
    this.callbacks[RodioEvents.ON_TIME_UPDATE] = (time: number) => {
      this._currentTime = time
      callback(time)
    }
  }

  protected listenOnLoad(callback: () => void): void {
    this.callbacks[RodioEvents.ON_LOADED] = callback
  }

  protected listenOnError(callback: (err: Error) => void): void {
    this.callbacks[RodioEvents.ON_ERROR] = callback
  }

  protected listenOnStateChange(callback: (state: PlayerState) => void): void {
    this.callbacks[RodioEvents.ON_PLAY] = () => callback('PLAYING')
    this.callbacks[RodioEvents.ON_PAUSE] = () => callback('PAUSED')
    this.callbacks[RodioEvents.ON_STOP] = () => callback('STOPPED')
  }

  protected listenOnBuffer(callback: () => void): void { }

  removeAllListeners(): void {
    this.callbacks = {}
  }

  createAudioContext(): AudioContext | undefined {
    return undefined
  }

  connectAudioContextNode(node: AudioNode): void { }

  preload(src: string): void { }

  public async canPlay(src: string): Promise<boolean> {
    return src.startsWith('media://')
  }
}
