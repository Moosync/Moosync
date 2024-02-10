import EventEmitter from 'events'
import { vxm } from '@/mainWindow/store'
import { LocalPlayer } from './local'

export class PipedPlayer extends LocalPlayer {
  private customLoadEventEmitter = new EventEmitter()

  public provides(): PlayerTypes[] {
    return ['YOUTUBE', 'SPOTIFY']
  }

  get key() {
    return 'YOUTUBE'
  }

  public async canPlay(src: string): Promise<boolean> {
    return src.length === 11 || vxm.providers.youtubeProvider.matchSongUrl(src)
  }

  protected async _load(src?: string | undefined, volume?: number | undefined, autoplay?: boolean | undefined) {
    this.customLoadEventEmitter.emit('loading')
    if (src) {
      const playbackUrl = await vxm.providers._pipedProvider.getStreamUrl(src)
      if (playbackUrl) {
        super._load(playbackUrl, volume, autoplay)
      } else {
        this.customLoadEventEmitter.emit('error', new Error(`Invalid src: ${src}`))
      }
    }
    this.customLoadEventEmitter.emit('loaded')
  }

  protected listenOnLoad(callback: () => void): void {
    this.customLoadEventEmitter.on('loaded', callback)
    super.listenOnLoad(callback)
  }

  protected listenOnBuffer(callback: () => void): void {
    this.customLoadEventEmitter.on('loading', callback)
    super.listenOnBuffer(callback)
  }

  protected listenOnError(callback: (err: Error) => void): void {
    this.customLoadEventEmitter.on('error', callback)
    super.listenOnError(callback)
  }
}
