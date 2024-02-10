import YTPlayer from 'yt-player'

export class YTPlayerWrapper implements CustomAudioInstance {
  isCustomAudio = true
  private supposedVolume
  private instance: YTPlayer

  listeners: Record<string, never> = {}
  private elementIdentifier: string

  // stop() should not be called if player is going to be reused
  // This var will make sure that the 'paused' event is not fired when the
  // song is supposed to be stopped.
  private pauseAsStop = false

  constructor(element: string | HTMLElement) {
    if (element) {
      this.instance = new YTPlayer(element, {
        modestBranding: true,
        related: false,
        annotations: false,
        keyboard: false,
        controls: false,
      })
      this.elementIdentifier = element instanceof HTMLElement ? element?.id : element
      this.supposedVolume = this.volume
      this.instance.on('playing', () => {
        this.volume = this.supposedVolume
      })
    } else {
      throw new Error('Empty element passed to YTPlayer')
    }
  }

  dispatchEvent(ev: Event) {
    this.instance.emit(ev.type)
  }

  public load() {
    return
  }

  async setSrc(src: string, autoPlay = true) {
    this.shouldFireOnLoad = true
    this.instance.load(src, autoPlay)
  }

  set volume(volume: number) {
    this.supposedVolume = volume
    this.instance.setVolume(volume * 100)
  }

  get volume() {
    return this.instance.getVolume() / 100
  }

  get currentTime() {
    return this.instance.getCurrentTime()
  }

  set currentTime(time: number) {
    this.instance.seek(time)
  }

  public async play() {
    this.instance?.play()
  }

  public pause() {
    this.instance?.pause()
  }

  public stop() {
    this.pauseAsStop = true
    this.instance.pause()
  }

  get paused() {
    return (
      this.instance.getState() === 'paused' ||
      this.instance.getState() === 'ended' ||
      this.instance.getState() === 'unstarted' ||
      this.instance.getState() === 'cued'
    )
  }

  set srcObject(o: unknown) {
    if (!o) {
      this.stop()
    }
    return
  }

  private removeListener(key: string) {
    this.removeEventListener(key)
  }

  set onended(callback: () => void) {
    if (!callback) {
      this.removeListener('ended')
      return
    }

    const mod = () => {
      this.shouldFireOnLoad = false
      callback()
    }

    this.instance.addListener('ended', mod)
  }

  set ontimeupdate(callback: never) {
    if (!callback) {
      this.removeListener('timeupdate')
      return
    }
    this.instance.addListener('timeupdate', callback)
  }

  private shouldFireOnLoad = false

  set onload(callback: () => void) {
    if (!callback) {
      this.removeListener('cued')
      return
    }
    this.instance.addListener('cued', callback)

    const mod = () => {
      if (this.shouldFireOnLoad) {
        callback()
        this.shouldFireOnLoad = false
      }
    }
    this.instance.addListener('playing', mod)
  }

  set onloadeddata(callback: never) {
    // this.instance.on('ended', callback)
  }

  set onerror(callback: never) {
    if (!callback) {
      this.removeListener('error')
      this.removeListener('unplayable')
      return
    }
    this.instance.addListener('error', callback)
    this.instance.addListener('unplayable', callback)
  }

  set onloadstart(callback: never) {
    if (!callback) {
      this.removeListener('buffering')
      return
    }
    this.instance.addListener('buffering', callback)
  }

  removeAttribute(): void {
    return
  }

  addEventListener(ev: string, callback: (...args: unknown[]) => void) {
    let modEv = ev
    if (ev === 'play') {
      modEv = 'playing'
    }

    if (ev === 'pause') {
      modEv = 'paused'
    }

    const mod = (...args: unknown[]) => {
      if (modEv === 'paused' && this.pauseAsStop) {
        this.pauseAsStop = false
        return
      }
      callback(...args)
    }

    this.listeners[ev] = mod as never
    this.instance.addListener(modEv, mod)
  }

  removeEventListener(ev: string) {
    console.debug('Youtube Player: Removing listener', ev)
    this.instance.removeAllListeners(ev)
  }

  setPlaybackQuality(quality: Parameters<typeof this.instance.setPlaybackQuality>[0]) {
    this.instance.setPlaybackQuality(quality)
  }
}
