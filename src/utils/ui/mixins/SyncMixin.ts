/*
 *  SyncMixin.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { Component, mixins } from 'vue-facing-decorator'

import { bus } from '@/mainWindow/main'
import { vxm } from '@/mainWindow/store'
import { PeerMode } from '@/mainWindow/store/syncState'
import { RepeatState } from '@/utils/commonConstants'
import ImgLoader from '@/utils/ui/mixins/ImageLoader'
import ModelHelper from '@/utils/ui/mixins/ModelHelper'
import { SyncHolder } from '../sync/syncHandler'

@Component
export default class SyncMixin extends mixins(ModelHelper, ImgLoader) {
  private isFetching = false
  private peerHolder: SyncHolder = new SyncHolder()
  private isRemoteStateChange = false
  private isRemoteTrackChange = false
  public setSongSrcCallback!: (src: string) => void
  public onSeekCallback!: (time: number) => void

  private _resolve!: () => void
  private _reject!: (r: string) => void
  private initialized = new Promise<void>(this.attachPromise.bind(this))

  private isReadyRequested = false

  private attachPromise(resolve: () => void, reject: (r: string) => void) {
    this._resolve = resolve
    this._reject = reject
  }

  created() {
    // this.peerHolder.initialize().then(() => {
    //   this.peerHolder.start()
    //   vxm.sync.socketID = this.peerHolder.socketID
    //   console.log('initialized', vxm.sync.socketID)
    //   this._resolve()
    // }).catch(err => {
    //   this._reject(err)
    // })
  }

  mounted() {
    // this.initialized.then(this.syncListeners).catch(err => console.error(err))
  }

  get isWatching() {
    return vxm.sync.mode === PeerMode.WATCHER
  }

  get isSyncing() {
    return vxm.sync.mode !== PeerMode.UNDEFINED
  }

  private isYoutube(song: RemoteSong): boolean {
    return song.type === 'YOUTUBE' || song.type === 'SPOTIFY'
  }

  private async setLocalCover(event: RemoteSong, from: string) {
    let cover: string | undefined

    if (event.senderSocket === this.peerHolder.socketID) {
      cover = (
        await window.SearchUtils.searchSongsByOptions({
          song: {
            _id: event._id,
          },
        })
      )[0].song_coverPath_high
    } else {
      cover = await window.FileUtils.isImageExists(event._id)
    }

    if (cover) vxm.sync.setCover(`media://${cover}`)
    else {
      vxm.sync.setCover(undefined)
      this.peerHolder.requestCover(from, event._id)
    }
  }

  private async checkLocalAudio(event: RemoteSong) {
    if (event.senderSocket !== this.peerHolder.socketID) {
      const isAudioExists = await window.FileUtils.isAudioExists(event._id)
      if (isAudioExists) {
        if (this.isReadyRequested) this.peerHolder.emitReady()
        this.setSongSrcCallback(`media://${isAudioExists}`)
      }
    }
  }

  private async checkYoutubeAudio() {
    if (this.isReadyRequested) this.peerHolder.emitReady()
  }

  private async setYoutubeCover(event: RemoteSong) {
    if (event.song_coverPath_low?.startsWith('http') || event.song_coverPath_high?.startsWith('http'))
      vxm.sync.setCover(event.song_coverPath_high ?? event.song_coverPath_low ?? '')
    else vxm.sync.setCover('')
  }

  private async setRemoteTrackInfo(from: string, songIndex: number) {
    vxm.sync.queueIndex = songIndex
    const song = vxm.sync.queueTop

    console.debug('Got remote track info', song, songIndex, from, this.peerHolder.socketID)

    if (song) {
      vxm.sync.playQueueSong(songIndex)

      if (this.isSyncing) {
        if (this.peerHolder.socketID !== from) {
          this.isRemoteTrackChange = true
          vxm.player.playerState = 'PAUSED'
        } else {
          this.peerHolder.requestReadyStatus()
          vxm.player.loading = true
        }

        if (this.isYoutube(song)) {
          await this.setYoutubeCover(song)
          await this.checkYoutubeAudio()
        } else {
          await this.setLocalCover(song, from)
          await this.checkLocalAudio(song)
        }
      }
    }
  }

  private setRemoteCover(event: Blob) {
    if (this.isSyncing && vxm.sync.currentSong) {
      const reader = new FileReader()
      const songID = vxm.sync.currentSong._id
      reader.onload = async () => {
        if (reader.readyState === 2) {
          const buffer = Buffer.from(reader.result as ArrayBuffer)
          const filePath = await window.FileUtils.saveImageToFile(songID, buffer)
          vxm.sync.setCover(`media://${filePath}`)
        }
      }
      reader.readAsArrayBuffer(event)
    }
  }

  private async getLocalCover(songID: string) {
    const songs = await window.SearchUtils.searchSongsByOptions({
      song: {
        _id: songID,
      },
    })

    if (songs.length > 0 && songs[0]) {
      const song = songs[0]
      if (song) {
        const cover = this.getValidImageHigh(song) ?? this.getValidImageLow(song)
        if (cover) {
          const resp = await fetch(this.getImgSrc(cover))
          const buf = await resp.arrayBuffer()
          return buf
        }
      }
    }
    return null
  }

  private saveRemoteStream(event: Blob) {
    const reader = new FileReader()
    reader.onload = async () => {
      if (reader.readyState === 2) {
        const buffer = Buffer.from(reader.result as ArrayBuffer)
        const filePath = await window.FileUtils.saveAudioToFile(vxm.sync.currentFetchSong, buffer)
        this.isFetching = false
        if (vxm.sync.currentSong?._id === vxm.sync.currentFetchSong) {
          if (this.isReadyRequested) this.peerHolder.emitReady()
          if (this.setSongSrcCallback) this.setSongSrcCallback(`media://${filePath}`)
        }
      }
    }
    reader.readAsArrayBuffer(event)
  }

  private async onLocalSongRequested(songID: string) {
    const songs = await window.SearchUtils.searchSongsByOptions({
      song: {
        _id: songID,
      },
    })

    if (songs.length > 0 && songs[0]) {
      const song = songs[0]
      if (song) {
        const resp = await fetch(`media://${song.path}`)
        const buf = await resp.arrayBuffer()
        return buf
      }
    }
    return null
  }

  private async handleRemotePlayerState(state: PlayerState) {
    console.debug('got state', vxm.player.playerState)
    if (vxm.player.playerState !== state) {
      this.isRemoteStateChange = true
      vxm.player.playerState = state
    }
  }

  private onRemoteSeek(time: number) {
    this.onSeekCallback(time)
  }

  private handleReadyEmitted() {
    this.isReadyRequested = false
  }

  private async handleReadyRequest() {
    this.isReadyRequested = true
    if (vxm.sync.currentSong) {
      if (vxm.sync.currentSong.type === 'LOCAL') {
        const isAudioExists = await window.FileUtils.isAudioExists(vxm.sync.currentSong._id)
        if (!this.isFetching) {
          /*
           * If the room is already streaming and another user joins in, everyone's state will be set to LOADING.
           * The users who already were playing the song might not be fetching and should only check if the audio exists
           */
          if (isAudioExists) this.peerHolder.emitReady()
        } else {
          /*
           * If the user is fetching a song, check if it matches the current playing.
           * If it does, then let it fetch and emitReady will be handled by saveRemoteStream
           * Otherwise check if audio exists and emitReady if it does
           */
          if (vxm.sync.currentFetchSong !== vxm.sync.currentSong._id) {
            if (isAudioExists) this.peerHolder.emitReady()
          }
        }
      } else {
        this.peerHolder.emitReady()
      }
    }
  }

  private syncListeners() {
    this.peerHolder.onRemoteTrackInfo = this.setRemoteTrackInfo
    this.peerHolder.onRemoteCover = this.setRemoteCover
    this.peerHolder.getLocalCover = this.getLocalCover
    this.peerHolder.onRemoteStream = this.saveRemoteStream
    this.peerHolder.getRequestedSong = this.playRequested
    this.peerHolder.getLocalSong = this.onLocalSongRequested
    this.peerHolder.fetchCurrentSong = () => vxm.player.queueIndex
    this.peerHolder.onPlayerStateChange = this.handleRemotePlayerState
    this.peerHolder.onQueueOrderChange = this.onRemoteQueueOrderChange
    this.peerHolder.onQueueDataChange = this.onRemoteQueueDataChange
    // TODO: Handle this event somewhere
    this.peerHolder.peerConnectionStateHandler = (id, state) => bus.emit('onPeerConnectionStateChange', id, state)
    this.peerHolder.onSeek = this.onRemoteSeek
    this.peerHolder.onReadyRequested = this.handleReadyRequest
    this.peerHolder.onReadyEmitted = this.handleReadyEmitted
    this.peerHolder.onRepeatChange = this.handleRepeat
    this.peerHolder.onAllReady = () => this.handleAllReady

    vxm.sync.$watch('queueIndex', this.triggerQueueChange)
    vxm.sync.$watch('queueOrder', this.triggerQueueChange)
    vxm.player.$watch('repeat', this.triggerRepeatChange)
  }

  private handleAllReady() {
    vxm.player.loading = false
  }

  private isRemoteRepeatChange = false

  private triggerRepeatChange(repeat: RepeatState) {
    if (!this.isRemoteRepeatChange) {
      this.peerHolder.emitRepeat(repeat)
    } else {
      this.isRemoteRepeatChange = false
    }
  }

  private handleRepeat(repeat: RepeatState) {
    this.isRemoteRepeatChange = true
    vxm.player.Repeat = repeat
  }

  private playRequested(songIndex: number) {
    const song = vxm.sync.queueData[vxm.sync.queueOrder[songIndex].songID]
    console.debug('Play requested for', song)
    if (song) {
      vxm.sync.setSong(song)
    }
  }

  private async fetchRemoteSong() {
    console.debug('fetching status', this.isFetching)
    if (!this.isFetching) {
      console.debug('fetching song')
      this.isFetching = true
      for (const fetch of vxm.sync.queueOrder) {
        const song = vxm.sync.queueData[fetch.songID]
        if (song && song.type === 'LOCAL') {
          const isExists = await window.FileUtils.isAudioExists(song._id)
          if (!isExists) {
            vxm.sync.setCurrentFetchSong(song._id)
            this.peerHolder.requestSong(song.senderSocket, song._id)
            return
          }
        }
      }
    }
  }

  private _ignoreRemoteChange = false

  private triggerQueueChange() {
    if (!this._ignoreRemoteChange) {
      this.peerHolder.emitQueueChange(vxm.sync.queueOrder, vxm.sync.queueData, vxm.sync.queueIndex)
    } else {
      this._ignoreRemoteChange = false
    }
  }

  private onRemoteQueueOrderChange(order: QueueOrder, index: number) {
    this._ignoreRemoteChange = true
    vxm.sync.queueOrder = order
    vxm.sync.queueIndex = index

    this.fetchRemoteSong()
  }

  private onRemoteQueueDataChange(data: QueueData<RemoteSong>) {
    vxm.sync.queueData = data
  }

  protected handleBroadcasterAudioLoad(): boolean {
    if (this.isSyncing) {
      if (this.isRemoteTrackChange) {
        this.isRemoteTrackChange = false
        return true
      }

      vxm.player.playerState = 'PAUSED'
      vxm.sync.setCover('')
      this.peerHolder.playSong(vxm.sync.queueIndex)

      return true
    }
    return false
  }

  private initializeRTC(mode: PeerMode) {
    this.peerHolder.peerMode = mode
    vxm.sync.setMode(mode)

    this.peerHolder.onJoinedRoom = (id: string, isCreator: boolean) => {
      vxm.sync.setRoom(id)

      if (isCreator) {
        this.triggerQueueChange()
      }
    }
  }

  protected joinRoom(id: string) {
    console.debug('joining room', id)
    this.initializeRTC(PeerMode.WATCHER)
    this.peerHolder.joinRoom(id)
  }

  protected createRoom() {
    this.initializeRTC(PeerMode.BROADCASTER)
    this.peerHolder.createRoom()
  }

  protected remoteSeek(time: number) {
    this.peerHolder.emitSeek(time)
  }

  protected emitPlayerState(newState: PlayerState) {
    if (this.isSyncing && !this.isRemoteStateChange) {
      this.peerHolder.emitPlayerState(newState)
    }
    this.isRemoteStateChange = false
  }
}
