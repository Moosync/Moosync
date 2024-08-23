class YTPlayer {
  static playerInstance = undefined;
  static element = undefined;
  static _isReady = false;

  queue = [];
  listeners = {};

  _instanceWatcher;

  _timeEmitter;

  constructor(element) {
    YTPlayer.element = element;

    this._instanceWatcher = setInterval(() => {
      if (YTPlayer._isReady) {
        console.log("Creating YT player instance");
        clearInterval(this._instanceWatcher);
        this._instanceWatcher = undefined;

        this.playerInstance = new YT.Player(YTPlayer.element, {
          height: "390",
          width: "390",
          playerVars: {
            autoplay: 1,
            controls: 2,
            disablekb: 0,
            enablejsapi: 1,
            fs: 1,
            modestbranding: 1,
            origin: window.location.origin,
            playsinline: 1,
            rel: 0,
            wmode: "opaque",
          },
          events: {
            onReady: () => this.onReady,
            onStateChange: (data) => this.onStateChange(data),
            onPlaybackQualityChange: (data) =>
              this.onPlaybackQualityChange(data),
            onPlaybackRateChange: (data) => this.onPlaybackRateChange(data),
            onError: (data) => this.onError(data),
          },
        });
      }
    }, 100);
  }

  startTimeEmit() {
    if (this._timeEmitter) {
      this.stopTimeEmit();
    }

    this._timeEmitter = setInterval(() => {
      this.emit("timeUpdate", this.playerInstance.getCurrentTime());
    }, 300);
  }

  stopTimeEmit() {
    if (this._timeEmitter) {
      clearInterval(this._timeEmitter);
      this._timeEmitter = undefined;
    }
  }

  flushQueue() {
    for ({ command, args } of this.queue) {
      // Execute all queued commands
      console.log("Executing", command, args);
      this.playerInstance[command](...args);
    }
  }

  run(command, ...args) {
    if (YTPlayer._isReady && this.playerInstance) {
      this.playerInstance[command](...args);
    } else {
      this.queue.push({ command, args });
    }
  }

  on(event, callback) {
    if (!this.listeners[event]) {
      this.listeners[event] = [];
    }

    this.listeners[event].push(callback);
  }

  off(event, callback) {
    if (!this.listeners[event]) {
      return;
    }

    this.listeners[event].splice(callback, 1);
  }

  removeAllListeners() {
    this.listeners = {};
  }

  once(event, callback) {
    const cb = (...args) => {
      callback(...args);
      this.off(event, cb);
    };
    this.on(event, cb);
  }

  emit(event, data) {
    const callbacks = this.listeners[event];
    if (!callbacks) {
      return;
    }

    for (const cb of callbacks) {
      cb(data);
    }
  }

  play() {
    this.run("playVideo");
  }

  pause() {
    this.run("pauseVideo");
  }

  stop() {
    this.run("stopVideo");
  }

  load(videoId, autoplay = false, start = 0) {
    console.log("Playing video ID", videoId);
    if (autoplay) {
      this.run("loadVideoById", videoId, start);
      this.play();
    } else {
      this.run("loadVideoById", videoId, start);
    }
  }

  seek(time) {
    this.run("seekTo", time, true);
  }

  getVolume() {
    const volume = this.playerInstance.getVolume();
    if (typeof volume === "undefined") {
      return 0;
    }
    return volume;
  }

  setVolume(volume) {
    this.run("setVolume", volume);
  }

  onReady() {
    this.flushQueue();
  }

  onStateChange(data) {
    const state = Number(data.data);
    switch (state) {
      case YT.PlayerState.PLAYING:
        this.startTimeEmit();
        break;
      case -1:
      case YT.PlayerState.PAUSED:
      case YT.PlayerState.ENDED:
      case YT.PlayerState.BUFFERING:
        this.stopTimeEmit();
        break;
    }
    this.emit("stateChange", state);
  }

  onError(err) {
    this.emit("error", JSON.stringify(err));
  }

  onPlaybackQualityChange(data) {
    this.emit("playbackQualityChange", data.data);
  }

  onPlaybackRateChange(data) {
    this.emit("playbackRateChange", data.data);
  }

  onError(data) {
    console.log(data);
    const code = Number(data.data);
    this.emit("error", data.data);
  }

  static createInstance() {
    console.log("YT api ready");
    YTPlayer._isReady = true;
  }
}

window.__MOOSYNC__ = {
  ...window.__MOOSYNC__,
  YTPlayer: YTPlayer,
};

function onYouTubeIframeAPIReady() {
  YTPlayer.createInstance();
}
