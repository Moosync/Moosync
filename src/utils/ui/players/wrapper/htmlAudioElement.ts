type WrappedAudioInstance = {
  setSrc: (src: string, autoPlay?: boolean) => void
  src: string
  play: () => void
  load: () => void
  stop: () => void
  addEventListener: (ev: string, listener: (...args: unknown[]) => void) => void
  removeAttribute: (key: string) => void
  srcObject: unknown
}

export function wrapHTMLAudioElement(elem: HTMLAudioElement | CustomAudioInstance): CustomAudioInstance {
  if (elem instanceof HTMLAudioElement) {
    const prototype = Reflect.getPrototypeOf(elem) as WrappedAudioInstance
    if (prototype) {
      prototype.setSrc = function (src: string, autoPlay?: boolean) {
        this.src = src
        this.load()
        autoPlay && this.play()
      }

      prototype.stop = function () {
        this.removeAttribute('src')
        this.srcObject = null
        this.load()
      }
    }

    Reflect.setPrototypeOf(elem, prototype)
  }

  return elem as unknown as CustomAudioInstance
}
