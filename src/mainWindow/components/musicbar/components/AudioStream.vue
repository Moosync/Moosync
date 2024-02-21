<!-- 
  AudioStream.vue is a part of Moosync.
  
  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
  Licensed under the GNU General Public License. 
  
  See LICENSE in the project root for license information.
-->

<template>
  <div class="h-100 w-100">
    <div ref="audioHolder" class="h-100 w-100">
      <div class="w-100 h-100 position-relative">
        <div class="yt-player" ref="ytAudioElement" id="yt-player"></div>
        <div class="yt-player-overlay h-100 w-100" v-if="isJukeboxModeActive"></div>
      </div>
      <audio id="dummy-yt-player" />
      <audio ref="audioElement" preload="auto" crossorigin="anonymous" />
      <video ref="dashPlayerDiv" class="dash-player" crossorigin="anonymous"></video>
      <video ref="hlsPlayerDiv" class="hls-player" crossorigin="anonymous"></video>
    </div>
  </div>
</template>

<script lang="ts">
const enum ButtonEnum {
  Play = 0,
  Pause = 1,
  Stop = 2,
  Record = 3,
  FastForward = 4,
  Rewind = 5,
  Next = 6,
  Previous = 7,
  ChannelUp = 8,
  ChannelDown = 9,
  Shuffle = 10,
  Repeat = 11,
  Seek = 12,
  PlayPause = 13,
  Position = 14
}

const SONG_CHANGE_DEBOUNCE = 100

import { Component, Prop, Ref, Watch } from 'vue-facing-decorator'
import { mixins } from 'vue-facing-decorator'
import { Player } from '@/utils/ui/players/player'
import { YoutubePlayer } from '@/utils/ui/players/youtube'
import { LocalPlayer } from '@/utils/ui/players/local'
import SyncMixin from '@/utils/ui/mixins/SyncMixin'
import CacheMixin from '@/utils/ui/mixins/CacheMixin'
import { vxm } from '@/mainWindow/store'
import ErrorHandler from '@/utils/ui/mixins/errorHandler'
import PlayerControls from '@/utils/ui/mixins/PlayerControls'
import { InvidiousPlayer } from '../../../../utils/ui/players/invidious'
import { DashPlayer } from '../../../../utils/ui/players/dash'
import JukeboxMixin from '@/utils/ui/mixins/JukeboxMixin'
import { HLSPlayer } from '@/utils/ui/players/hls'
import { YoutubeAlts } from '@/mainWindow/store/providers'
import { PipedPlayer } from '@/utils/ui/players/piped'
import ProviderMixin from '@/utils/ui/mixins/ProviderMixin'
import { GenericProvider } from '@/utils/ui/providers/generics/genericProvider'
import { SpotifyPlayer } from '@/utils/ui/players/spotify'
import { isEmpty } from '@/utils/common'
import { bus } from '@/mainWindow/main'
import { EventBus } from '@/utils/preload/ipc/constants'
import { nextTick } from 'vue'
import { convertProxy } from '@/utils/ui/common'
import { RepeatState } from '../../../../utils/commonConstants';
import { RodioPlayer } from '../../../../utils/ui/players/rodio';
import { convertFileSrc } from '@tauri-apps/api/core';

@Component({
  emits: ['onTimeUpdate']
})
export default class AudioStream extends mixins(
  SyncMixin,
  PlayerControls,
  ErrorHandler,
  CacheMixin,
  JukeboxMixin,
  ProviderMixin
) {
  @Ref
  private audioElement!: HTMLAudioElement

  @Ref
  private ytAudioElement!: HTMLDivElement

  @Ref
  private dashPlayerDiv!: HTMLVideoElement

  @Ref
  private hlsPlayerDiv!: HTMLVideoElement

  @Prop({ default: '' })
  roomID!: string

  get currentSong(): Song | null | undefined {
    return vxm.player.currentSong
  }

  /**
   * Player responsible for handling current song
   * May switch between youtube and local
   */
  private activePlayer?: Player

  /**
   * Holds type of player which is current active
   */
  private activePlayerTypes?: PlayerTypes

  /**
   * True is page has just loaded and a new song is to be loaded into the player
   * Otherwise false
   */
  private isFirst = true

  private playersInitialized = false

  /**
   * True if vuex state change is not to be reflected on active player
   * When player is paused or played from an external source, the onStateChange event triggers
   * and the vuex player state is changed respectively. This flag is set to true to avoid setting
   * the same state on active player again
   */
  private ignoreStateChange = false

  private stateChangeQueued = false

  private playerBlacklist: string[] = []

  private _bufferTrap: ReturnType<typeof setTimeout> | undefined

  private get showYTPlayer() {
    return vxm.themes.showPlayer
  }

  private set showYTPlayer(show: number) {
    vxm.themes.showPlayer = show
  }

  /**
   * Method called when vuex player state changes
   * This method is responsible for reflecting that state on active player
   */
  async onPlayerStateChanged(newState: PlayerState) {
    if (!this.ignoreStateChange) {
      if (vxm.player.loading) {
        this.stateChangeQueued = true
        return
      }
      await this.handleActivePlayerState(newState)
      this.emitPlayerState(newState)

      await window.MprisUtils.updatePlaybackState(newState)
    }

    this.ignoreStateChange = false
  }

  /**
   * Method called when player type changes
   * This method is responsible of detaching old player
   * and setting new player as active
   */
  private async onPlayerTypesChanged(newType: PlayerTypes, song: Song): Promise<string | undefined> {
    let player: Player | undefined = undefined

    let tries = 0
    while (!(player && (song.path ?? song.playbackUrl)) && tries < vxm.playerRepo.allPlayers.length) {

      if (song.path && !this.playerBlacklist.includes('LOCAL')) {
        newType = 'LOCAL'
      }

      player = this.findPlayer(newType, this.playerBlacklist)
      console.debug('Found player', song, newType, player?.key)

      if (newType === 'LOCAL' && player) {
        if (song.duration < 0) {
          song.duration = (await this.getPlaybackDurationFromPlayer(song)) ?? 0
        }
        break
      }

      if (player) {
        await this.setPlaybackURLAndDuration(song, player.key)
      }

      if (!player) {
        console.error('No player found to play', song.path || song.playbackUrl)
        if (vxm.player.queueOrder.length > 1) {
          this.playerBlacklist = []
          this.nextSong()
        }
        return
      }

      if (!song.playbackUrl) {
        console.info('Blacklisting', player.key)
        this.playerBlacklist.push(player.key)
        player = undefined
        continue
      }

      console.debug('Checking player', player.key, 'for', song.playbackUrl)
      if (!(await player.canPlay(song.playbackUrl))) {
        this.playerBlacklist.push(player.key)
        player = undefined
      } else {
        console.debug('Found player', player?.key, 'and can play', song.playbackUrl)
      }

      tries += 1
    }

    if (this.activePlayer !== player) {
      console.debug('Unloading players')
      this.unloadAudio()
      this.clearAllListeners()

      if (player) {
        console.debug('Initializing player', player.key)
        await this.initializePlayer(player)
        console.debug('Initialized player', player.key)

        this.activePlayer = player

        this.activePlayer.volume = this.volume
        this.registerPlayerListeners()
        this.activePlayerTypes = newType

        console.log('using embeds', this.useEmbed)
        this.showYTPlayer =
          this.useEmbed && vxm.providers.youtubeAlt === YoutubeAlts.YOUTUBE && player.key === 'YOUTUBE' ? 2 : 0
        this.analyserNode = undefined

        return player.key
      } else {
        return
      }
    }
    return this.activePlayer?.key
  }

  private songChangeTimer: ReturnType<typeof setTimeout> | undefined

  private clearSongChangeTimer() {
    if (this.songChangeTimer) {
      clearTimeout(this.songChangeTimer)
      this.songChangeTimer = undefined
    }
  }

  /**
   * Method triggered when currentSong prop changes
   * This method is responsible for loading the current song in active player
   * or unloading the player if current song is empty
   */
  @Watch('currentSong', { immediate: true })
  onSongChanged(newSong: Song | null | undefined) {
    if (newSong) {
      this.playerBlacklist = []
      this.clearSongChangeTimer()
      this.songChangeTimer = setTimeout(() => this.loadAudio(newSong, false), SONG_CHANGE_DEBOUNCE)
    } else {
      this.unloadAudio()
      this.showYTPlayer = 0
      this.lastLoadedSong = undefined
      this.clearSongChangeTimer()
    }
  }

  /**
   * Method triggered when vuex volume changes
   */
  onVolumeChanged() {
    if (this.activePlayer) {
      this.activePlayer.volume = this.volume
    }
  }

  /**
   * Method triggered when user seeks on timeline and forceSeek prop changes
   */
  onSeek(newValue?: number) {
    if (typeof newValue === 'number') {
      if (this.activePlayer) {
        this.activePlayer.currentTime = newValue
        if (this.isSyncing) this.remoteSeek(newValue)
        vxm.player.forceSeek = undefined
      }
    }
  }

  async mounted() {
    await this.setupPlayers()

    this.playersInitialized = true

    this.setupSync()
    this.registerListeners()

    const lastLoadedPlaybackState =
      (await window.PreferenceUtils.loadSelectiveArrayItem<SystemSettings>('system.last_loaded_playback_state'))
        ?.enabled ?? false

    if (!lastLoadedPlaybackState) {
      vxm.player.playerState = 'PAUSED'
    }

    if (this.currentSong) this.loadAudio(this.currentSong, true)
  }

  closePlayers() {
    for (const p of vxm.playerRepo.allPlayers) {
      p.close()
    }
  }

  private useEmbed = true

  /**
   * Initial setup for all players
   */
  private async setupPlayers(): Promise<void> {
    const players = await new Promise<Player[]>((resolve) => {
      const players: Player[] = []

      // players.push(new RodioPlayer())
      players.push(new LocalPlayer())
      players.push(new DashPlayer())
      players.push(new HLSPlayer())
      players.push(new SpotifyPlayer())

      // youtubeAlt might be set after setupPlayer, so we watch it change
      vxm.providers.$watch(
        'youtubeAlt',
        async (val: YoutubeAlts) => {
          switch (val) {
            case YoutubeAlts.YOUTUBE:
              players.push(new YoutubePlayer())
              break
            case YoutubeAlts.INVIDIOUS:
              players.push(new InvidiousPlayer())
              break
            case YoutubeAlts.PIPED:
              players.push(new PipedPlayer())
              break
          }

          resolve(players)
        },
        { deep: false, immediate: true }
      )
    })

    vxm.playerRepo.clear()
    vxm.playerRepo.push(players)
  }

  private async initializePlayer(player: Player) {
    if (player.isInitialized) return

    if (player instanceof YoutubePlayer) {
      console.log("embeds", await window.PreferenceUtils.loadSelectiveArrayItem<Checkbox>('youtubeOptions.youtube_embeds'))
      this.useEmbed =
        (await window.PreferenceUtils.loadSelectiveArrayItem<Checkbox>('youtubeOptions.youtube_embeds'))?.enabled ??
        true

      await player.initialize({
        playerInstance: this.ytAudioElement,
        useEmbed: this.useEmbed
      })

      return
    }

    if (player instanceof InvidiousPlayer || player instanceof PipedPlayer || player instanceof LocalPlayer) {
      await player.initialize(this.audioElement)
      return
    }

    if (player instanceof SpotifyPlayer || player instanceof RodioPlayer) {
      await player.initialize()
    }

    if (player instanceof HLSPlayer) {
      await player.initialize(this.hlsPlayerDiv)
      return
    }

    if (player instanceof DashPlayer) {
      await player.initialize(this.dashPlayerDiv)
      return
    }
  }

  private setupSync() {
    this.setSongSrcCallback = (src: string) => this.activePlayer && this.activePlayer.load(src)
    this.onSeekCallback = (time: number) => this.activePlayer && (this.activePlayer.currentTime = time)
  }

  private registerRoomListeners() {
    // bus.on('join-room', (data: string) => this.joinRoom(data))
    // bus.on('create-room', () => this.createRoom())
  }

  private async onSongEnded() {
    vxm.player.playAfterLoad = true
    this.lastLoadedSong = undefined
    if (this.repeat !== RepeatState.DISABLED && this.currentSong) {
      // Re load entire audio instead of setting current time to 0
      this.loadAudio(this.currentSong, false)

      if (this.repeat === RepeatState.ONCE) {
        this.repeat = RepeatState.DISABLED
      }

    } else {
      vxm.player.currentSong = undefined
      await this.nextSong()
    }
  }

  private analyserNode: AnalyserNode | undefined

  // https://jameshfisher.com/2021/01/18/measuring-audio-volume-in-javascript
  private isSilent() {
    if (this.analyserNode) {
      const pcmData = new Float32Array(this.analyserNode.fftSize)

      this.analyserNode.getFloatTimeDomainData(pcmData)
      let sumSquares = 0.0
      for (const amplitude of pcmData) {
        sumSquares += amplitude * amplitude
      }
      const amplitude = parseFloat(Math.sqrt(sumSquares / pcmData.length).toFixed(4))
      console.debug('Got silence')
      return amplitude === 0
    }
    return false
  }

  private timeSkipSeconds = 0

  /**
   * Register all listeners related to players
   */
  private registerPlayerListeners() {
    if (vxm.player.loading) {
      vxm.player.loading = false
    }

    if (this.activePlayer) {
      const preloadDuration = this.activePlayer.key === 'SPOTIFY' ? 45 : 10
      this.activePlayer.onTimeUpdate = async (time) => {
        this.$emit('onTimeUpdate', time)

        this.updateMprisPosition(time)

        if (this.currentSong) {
          if (time >= this.currentSong.duration - preloadDuration - this.timeSkipSeconds) {
            await this.preloadNextSong()
            if (this.isSilent()) {
              this.onSongEnded()
            }
          }

          if (this.timeSkipSeconds && time >= this.currentSong.duration - this.timeSkipSeconds) {
            this.onSongEnded()
          }
        }
      }

      this.activePlayer.onError = async (err) => {
        console.error('Player error', err.message, 'while playing', this.currentSong?.playbackUrl)
        console.error(`${this.currentSong?._id}: ${this.currentSong?.title} unplayable, skipping.`)

        // Blacklist current player and try to find alternative
        if (this.currentSong && this.activePlayer?.key) {
          console.debug('Blacklisting player', this.activePlayer.key)
          this.playerBlacklist.push(this.activePlayer.key)
          vxm.player.playAfterLoad = true
          this.loadAudio(this.currentSong, true, true)
        } else {
          if (vxm.player.queueOrder.length > 1) this.nextSong()
        }

        // await this.nextSong()
        // await this.removeFromQueue(vxm.player.queueIndex - 1)
        vxm.player.loading = false
      }

      this.activePlayer.onStateChange = (state) => {
        // Cued event of youtube embed seems to fire only once and is not reliable
        // Stop loading when state of player changes
        vxm.player.loading = false
        this.cancelBufferTrap()

        if (state === 'STOPPED') {
          // this.onSongEnded()
          return
        }

        if (state !== vxm.player.playerState) {
          this.ignoreStateChange = true
          vxm.player.playerState = state
        }
      }

      this.activePlayer.onLoad = async () => {
        console.debug('Clearing player blacklist')
        this.playerBlacklist = []

        const preferences = await window.PreferenceUtils.loadSelective<Checkbox[]>('audio')
        if (preferences) {
          const gapless = preferences.find((val) => val.key === 'gapless_playback')
          if (gapless && gapless.enabled) {
            if (!this.analyserNode) {
              const context = this.activePlayer?.createAudioContext()
              if (context) {
                this.analyserNode = context.createAnalyser()
                this.activePlayer?.connectAudioContextNode(this.analyserNode)
              }
            }
          } else {
            this.analyserNode = undefined
          }
        }
        vxm.player.loading = false
      }

      vxm.player.loading = false
      this.cancelBufferTrap()

      this.activePlayer.onBuffer = () => {
        vxm.player.loading = true
        this.setBufferTrap()
      }

      this.activePlayer.onEnded = () => {
        this.onSongEnded()
      }
    }

    vxm.player.$watch('volume', this.onVolumeChanged)
    vxm.player.$watch('loading', (newVal) => {
      if (!newVal && this.stateChangeQueued) {
        this.onPlayerStateChanged(vxm.player.playerState)
        this.stateChangeQueued = false
      }
    })
  }

  /**
   * If the player is buffering for a long time then try changing its playback quality
   */
  private setBufferTrap() {
    if (!this._bufferTrap) {
      this._bufferTrap = setTimeout(() => {
        if (
          this.activePlayerTypes &&
          this.activePlayer?.provides().includes(this.activePlayerTypes) &&
          this.activePlayer instanceof YoutubePlayer
        ) {
          // this.activePlayer.setPlaybackQuality('small')
          this?.pause()
          nextTick(() => this.play())

          console.debug('Triggered buffer trap')
        }
      }, 3000)
    }
  }

  private cancelBufferTrap() {
    if (this._bufferTrap) {
      clearTimeout(this._bufferTrap)
      this._bufferTrap = undefined
    }
  }

  private handleSeek(seek: number, relative: boolean) {
    if (seek) {
      const parsed = seek / 10e5
      const newPos = relative ? vxm.player.currentTime + parsed : parsed
      bus.emit('forceSeek', newPos)
      vxm.player.forceSeek = newPos
    }
  }

  private registerMediaControlListener() {
    window.MprisUtils.listenMediaButtonPress((button, arg) => {
      switch (button) {
        case ButtonEnum.Play:
          this.play()
          break
        case ButtonEnum.Pause:
          this.pause()
          break
        case ButtonEnum.Stop:
          this.pause()
          break
        case ButtonEnum.Next:
          this.nextSong()
          break
        case ButtonEnum.Previous:
          this.prevSong()
          break
        case ButtonEnum.Shuffle:
          this.shuffle()
          break
        case ButtonEnum.Repeat:
          this.toggleRepeat()
          break
        case ButtonEnum.PlayPause:
          this.togglePlay()
          break
        case ButtonEnum.Seek:
          this.handleSeek(arg as number, true)
          break
        case ButtonEnum.Position:
          this.handleSeek((arg as { position: number })?.position, false)
          break
      }
    })
  }

  private async updateMprisPosition(position: number) {
    await window.MprisUtils.updatePosition(position)
  }

  private async registerListeners() {
    this.registerRoomListeners()
    this.registerMediaControlListener()

    vxm.player.$watch('playerState', this.onPlayerStateChanged, { immediate: true, deep: false })
    vxm.player.$watch('forceSeek', this.onSeek)

    bus.on(EventBus.FORCE_LOAD_SONG, () => {
      if (this.currentSong) {
        this.loadAudio(this.currentSong, true, true)
      }
    })

    this.timeSkipSeconds = await window.PreferenceUtils.loadSelective<number>('gapless.skip', false) ?? 0
    window.PreferenceUtils.listenPreferenceChanged<number>('gapless.skip', true, (_, value) => {
      this.timeSkipSeconds = value
    })
  }

  /**
   * Sets current player's state to vuex player state
   */
  private handleFirstPlayback(loadedState: boolean) {
    if (this.isFirst || vxm.player.queueOrder.length === 1) {
      if (!loadedState) {
        vxm.player.playerState = 'PLAYING'
      }
      this.isFirst = false
    }
  }

  private async getPlaybackDurationFromPlayer(song: Song) {
    try {
      const data = await new Promise<number | undefined>((resolve, reject) => {
        const url = song.path ? convertFileSrc(song.path, "media") : song.playbackUrl

        if (url) {
          const audio = new Audio()
          audio.onloadedmetadata = () => {
            if (url) resolve(audio.duration)
          }
          audio.onerror = reject

          audio.src = url
        } else {
          resolve(undefined)
        }
      })
      return data
    } catch (e) {
      console.error('Failed to get duration for url', song.playbackUrl, e)
    }
  }

  private async getPlaybackUrlAndDuration(
    provider: GenericProvider | undefined,
    song: Song,
    player: string
  ): Promise<{ url: string | undefined; duration?: number } | undefined> {
    if (provider) {
      console.debug('Fetching playback URL and duration from', provider.key)
      const res = await provider.getPlaybackUrlAndDuration(song, player)
      if (res) return res
    }

    const duration = await this.getPlaybackDurationFromPlayer(song)
    if (duration) {
      return { duration, url: song.playbackUrl }
    }
  }

  /**
   * Set media info which is recognised by different applications and OS specific API
   */
  private async setMediaInfo(song: Song) {
    const raw = convertProxy(song)
    await window.MprisUtils.updateSongInfo({
      id: raw._id,
      title: raw.title,
      duration: raw.duration,
      albumName: raw.album?.album_name,
      albumArtist: raw.album?.album_artist,
      artistName: raw.artists && raw.artists.map((val) => val.artist_name).join(', '),
      genres: raw.genre?.map(val => val.genre_name),
      thumbnail:
        raw.song_coverPath_high ??
        raw.album?.album_coverPath_high ??
        raw.song_coverPath_low ??
        raw.album?.album_coverPath_low
    })
  }

  get enableTrackControls() {
    return this.isSyncing ? vxm.sync.queueOrder.length > 1 : vxm.player.queueOrder.length > 1
  }

  @Watch('enableTrackControls', { immediate: true, deep: false })
  private async onEnableTrackControls() {
    await window.MprisUtils.setButtonStatus({
      play: true,
      pause: true,
      next: this.enableTrackControls,
      prev: this.enableTrackControls,
      shuffle: true,
      loop: 'None'
    })
  }

  @Watch('repeat', { immediate: true, deep: false })
  private async onRepeatChanged() {
    await window.MprisUtils.setButtonStatus({
      loop: this.repeat ? 'Track' : 'None'
    })
  }

  private async getLocalSong(songID: string) {
    const songs = await window.SearchUtils.searchSongsByOptions({
      song: {
        _id: songID
      }
    })

    if (songs.length > 0) {
      return songs[0]
    }
  }

  private preloadStatus: 'PRELOADING' | 'PRELOADED' | undefined

  private async preloadNextSong() {
    if (this.preloadStatus === 'PRELOADING' || this.preloadStatus === 'PRELOADED') {
      return
    }

    console.debug('Preloading next track')

    this.preloadStatus = 'PRELOADING'

    const nextSong = vxm.player.queueData[vxm.player.queueOrder[vxm.player.queueIndex + 1]?.songID]
    if (nextSong && !nextSong.path) {
      const blacklist = []
      let audioPlayer: Player | undefined = undefined

      let tries = vxm.playerRepo.allPlayers.length

      while (!audioPlayer && tries > 0) {
        audioPlayer = this.findPlayer(nextSong.type, blacklist)

        if (audioPlayer) {
          await this.setPlaybackURLAndDuration(nextSong, audioPlayer.key)

          if (!nextSong.playbackUrl || !nextSong.duration) {
            // await this.removeFromQueue(vxm.player.queueIndex + 1)
            this.preloadStatus = undefined
            return
          }
        }

        if (audioPlayer && nextSong.playbackUrl) {
          if (!(await audioPlayer.canPlay(nextSong.playbackUrl))) {
            blacklist.push(audioPlayer.key)
            audioPlayer = undefined
          }
        }
        tries -= 1
      }

      if (!audioPlayer) {
        console.error('Failed to find player for song', nextSong, 'not preloading')
      }

      if (!nextSong.playbackUrl) {
        console.error('Failed to find playback URL for song', nextSong, 'not preloading')
        this.preloadStatus = undefined
        return
      }
      audioPlayer?.preload(nextSong.playbackUrl)
    }

    this.preloadStatus = 'PRELOADED'
  }

  private async updateSongURLInDB(song: Song) {
    const songExists = (
      await window.SearchUtils.searchSongsByOptions({
        song: {
          _id: convertProxy(song._id)
        }
      })
    )[0]

    if (songExists) {
      await window.DBUtils.updateSongs([convertProxy(song)])
    }
  }

  private async setPlaybackURLAndDuration(song: Song, player: string) {
    const provider = this.getProviderBySong(song)

    let res: { url?: string; duration?: number } | undefined = { url: song.playbackUrl, duration: song.duration }

    if (!song.playbackUrl || isEmpty(song.duration)) {
      res = this.getItem(`url_duration:${song._id}`)
      console.debug('cache url and duration', res)
    }

    let shouldRefetch = true
    if (res?.url && res?.duration && (await provider?.validatePlaybackURL(res.url, player))) {
      shouldRefetch = false
    }

    console.debug('Should refetch playback url and duration', shouldRefetch)
    if (shouldRefetch) {
      console.debug('playback url and duration not in cache or missing')
      res = await this.getPlaybackUrlAndDuration(provider, song, player)
    }

    console.debug('Got playback url and duration', res)

    if (res && res.duration && res.url) {
      // song is a reference to vxm.player.currentSong or vxm.sync.currentSong.
      // Mutating those properties should also mutate song and vice-versa
      if (vxm.player.currentSong && song) {
        song.duration = res.duration
        song.playbackUrl = res.url

        this.setItem(`url_duration:${song._id}`, res)

        // Song item will be updated only if it exists in db
        this.updateSongURLInDB(song)
      }
    }
  }

  private lastLoadedSong?: Song

  private async loadAudio(song: Song, loadedState: boolean, force = false) {
    if (!this.playersInitialized) {
      return
    }

    if (this.isSyncing) {
      const tmp = await this.getLocalSong(song._id)
      if (tmp) {
        song = tmp
      }
    }

    if (!force && song._id === this.lastLoadedSong?._id) {
      console.debug('Got duplicate song', song)
      return
    }

    console.debug('Loading new song', song)

    // this.unloadAudio()

    const PlayerTypes = song.type

    this.lastLoadedSong = song

    vxm.player.loading = true

    const changedPlayer = await this.onPlayerTypesChanged(PlayerTypes, song)
    if (!changedPlayer) {
      return this.nextSong()
    }

    // Don't proceed if song has changed while we were fetching playback url and duration
    if (song._id !== this.currentSong?._id) {
      console.debug('Current song has changed, skipping request')
      return
    }

    if (!song.path && (!song.playbackUrl || !song.duration)) {
      console.error('Failed to get playbackURL', song)
      // await this.nextSong()
      // await this.removeFromQueue(vxm.player.queueIndex - 1)
      vxm.player.loading = false
      return
    }

    try {
      this.activePlayer?.load(
        song.path ? convertFileSrc(song.path, "asset") : song.playbackUrl,
        this.volume,
        vxm.player.playAfterLoad || this.playerState === 'PLAYING'
      )
    } catch (e) {
      console.error('failed to load song', e)
    }

    console.debug('Loaded song at', song.path ? convertFileSrc(song.path, "asset") : song.playbackUrl)
    vxm.player.loading = false


    vxm.player.playAfterLoad = false

    if (this.handleBroadcasterAudioLoad()) return

    this.handleFirstPlayback(loadedState)

    await this.setMediaInfo(song)

    // Clear preload status after song has changed
    this.preloadStatus = undefined

    await window.MprisUtils.updatePlaybackState(
      vxm.player.playAfterLoad || this.playerState !== 'PAUSED' ? 'PLAYING' : 'PAUSED'
    )

    // Increment play count for song
    window.DBUtils.incrementPlayCount(song._id)
  }

  private unloadAudio() {
    console.debug('Unloading audio')
    this.activePlayer?.stop()
    window.MprisUtils.updateSongInfo({})
  }

  private async handleActivePlayerState(newState: PlayerState) {
    if (!this.currentSong) return

    try {
      switch (newState) {
        case 'PLAYING':
          return this.activePlayer?.play()
        case 'PAUSED':
          return this.activePlayer?.pause()
        case 'STOPPED':
          return this.unloadAudio()
      }
    } catch (e) {
      console.debug(e)
      await this.nextSong()
    }
  }
}
</script>

<!-- Add "scoped" attribute to limit CSS to this component only -->
<style scoped>
h3 {
  margin: 40px 0 0;
}

ul {
  list-style-type: none;
  padding: 0;
}

li {
  display: inline-block;
  margin: 0 10px;
}

a {
  color: #42b983;
}
</style>

<style lang="sass">
.yt-player
  position: absolute
  border-radius: 16px
  z-index: 1 !important

.dash-player
  width: 0 !important

.yt-player-overlay
  z-index: 2
  position: absolute
</style>
