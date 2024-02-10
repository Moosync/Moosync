import EventEmitter from 'events'
import { vxm } from '../../../mainWindow/store/index'
import { LocalPlayer } from './local'

export class InvidiousPlayer extends LocalPlayer {
  private customLoadEventEmitter = new EventEmitter()
  private lastAutoPlay = false

  private errorTries = 0

  public provides(): PlayerTypes[] {
    return ['YOUTUBE', 'SPOTIFY']
  }

  get key() {
    return 'YOUTUBE'
  }

  public async canPlay(src: string): Promise<boolean> {
    return src.length === 11 || vxm.providers.youtubeProvider.matchSongUrl(src)
  }

  protected async _load(src?: string, volume?: number, autoplay?: boolean, errorTries = 0) {
    this.customLoadEventEmitter.emit('loading')
    let playbackURL = await this.fetchPlaybackURL(src)
    if (playbackURL) {
      const shouldProxy = (await window.PreferenceUtils.loadSelectiveArrayItem<Checkbox>('invidious.always_proxy'))
        ?.enabled

      if (shouldProxy ?? true) {
        playbackURL = await this.proxyVideoOnInvidious(playbackURL)
      }

      this.customLoadEventEmitter.emit('loaded')
      this.lastAutoPlay = autoplay ?? this.lastAutoPlay

      super._load(playbackURL, volume, this.lastAutoPlay)
    }
  }

  private async fetchPlaybackURL(str: string | undefined) {
    let videoId = str
    if (str) {
      if (str.startsWith('http')) {
        videoId = vxm.providers._invidiousProvider.getVideoIdFromURL(str)
      }

      if (videoId) {
        // This won't make a request to youtube
        const resp: InvidiousSong | undefined = await vxm.providers._invidiousProvider.getSongDetails(videoId)
        if (resp?.invidiousPlaybackUrl) {
          return resp.invidiousPlaybackUrl
        }
      } else {
        this.customLoadEventEmitter.emit('error', new Error('Invalid URL'))
      }
    }
  }

  private async proxyVideoOnInvidious(src: string) {
    const baseUrl = new URL((await window.PreferenceUtils.loadSelective<string>('invidious_instance')) ?? '')
    const currentSrc = new URL(src)

    if (baseUrl.host && baseUrl.host !== currentSrc.host) {
      currentSrc.host = new URL(baseUrl).host
    }

    return currentSrc.toString()
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
    this.playerInstance.onerror = async (event, source, line, col, err) => {
      if (this.errorTries < 3) {
        try {
          this.customLoadEventEmitter.emit('loading')
          const newUrl = await this.proxyVideoOnInvidious(((event as ErrorEvent)?.target as HTMLAudioElement)?.src)
          this.errorTries += 1
          this.load(newUrl, this.errorTries)
        } catch (e) {
          console.error(e)
        }
      }

      if (err) {
        callback(err)
      }
    }
  }
}
