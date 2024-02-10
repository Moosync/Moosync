/*
 *  syncHandler.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { ManagerOptions, Socket, io } from 'socket.io-client'
import { FragmentReceiver, FragmentSender } from './dataFragmenter'

import { PeerMode } from '@/mainWindow/store/syncState'
import { RepeatState } from '@/utils/commonConstants'

enum peerConnectionState {
  CONNECTED = 0,
  CONNECTING = 1,
  DISCONNECTED = 2,
}

const STUN = {
  urls: [
    'stun:stun.l.google.com:19302',
    'stun:stun.l.google.com:19302',
    'stun:stun1.l.google.com:19302',
    'stun:stun2.l.google.com:19302',
    'stun:stun3.l.google.com:19302',
    'stun:stun4.l.google.com:19302',
    'stun:stun.ekiga.net',
    'stun:stun.ideasip.com',
    'stun:stun.rixtelecom.se',
    'stun:stun.schlund.de',
    'stun:stun.stunprotocol.org:3478',
    'stun:stun.voiparound.com',
    'stun:stun.voipbuster.com',
    'stun:stun.voipstunt.com',
    'stun:stun.voxgratia.org',
  ],
}

const TURN = {
  urls: 'turn:retardnetwork.cf:7888',
  username: 'oveno',
  credential: '1234',
}

const connectionOptions: Partial<ManagerOptions> = {
  forceNew: true,
  reconnection: true,
  reconnectionAttempts: 2,
  timeout: 10000,
  transports: ['websocket'],
}

export class SyncHolder {
  private peerConnection: {
    [key: string]: {
      peer?: RTCPeerConnection
      coverChannel?: RTCDataChannel
      streamChannel?: RTCDataChannel
    }
  } = {}
  private socketConnection?: Socket

  private mode: PeerMode = PeerMode.UNDEFINED

  set peerMode(mode: PeerMode) {
    this.mode = mode
  }

  get peerMode() {
    return this.mode
  }

  private BroadcasterID = ''
  private isNegotiating: { [id: string]: boolean } = {}
  public socketID = ''

  private isListeningReady = false

  private initialized = false

  private readyPeers: string[] = []

  private onJoinedRoomCallback?: (id: string, isCreator: boolean) => void
  private onRemoteTrackInfoCallback?: (from: string, songIndex: number) => void
  private onRemoteTrackCallback?: (event: RTCTrackEvent) => void
  private onRemoteCoverCallback?: (event: Blob) => void
  private onRemoteStreamCallback?: (event: Blob) => void
  private onPlayerStateChangeCallback?: (state: PlayerState) => void
  private onSeekCallback?: (time: number) => void
  private onPeerStateChangeCallback?: (id: string, state: peerConnectionState) => void
  private onDataSentCallback?: (id: string) => void
  private onAllReadyCallback?: () => void
  private onReadyRequestedCallback?: () => void
  private onReadyEmittedCallback?: () => void
  private onQueueOrderChangeCallback?: (order: QueueOrder, index: number) => void
  private onQueueDataChangeCallback?: (data: QueueData<RemoteSong>) => void
  private onRepeatChangeCallback?: (repeat: RepeatState) => void
  private playRequestedSongCallback?: (songIndex: number) => void
  private getCurrentSongIndex?: () => number
  private getLocalSongCallback?: (songID: string) => Promise<ArrayBuffer | null>
  private getLocalCoverCallback?: (songID: string) => Promise<ArrayBuffer | null>

  constructor() {
    const handler = {
      get: function (obj: SyncHolder, methodName: keyof SyncHolder) {
        return typeof obj[methodName] !== 'function'
          ? obj[methodName]
          : function (...args: unknown[]) {
              if (obj.isInitialized(methodName)) {
                return (obj[methodName] as (...args: unknown[]) => void)(...args)
              }
            }
      },
    }

    // rome-ignore lint/correctness/noConstructorReturn: Need to proxyify this object
    return new Proxy(this, handler)
  }

  private isInitialized(methodName: string) {
    if (methodName !== 'isInitialized' && methodName !== 'initialize') {
      if (!this.socketConnection) {
        throw new Error('Handler not initialized, call initialize()')
      }

      return this.initialized
    }
    return true
  }

  public async initialize(url?: string): Promise<boolean> {
    return new Promise<boolean>((resolve, reject) => {
      this.socketConnection = io(url ? url : 'http://localhost:4000', connectionOptions)
      this.socketConnection.on('connect', () => {
        if (this.socketConnection?.id) {
          this.socketID = this.socketConnection.id
          this.initialized = true
          resolve(true)
        }
      })

      let tries = 0

      this.socketConnection?.on('connect_error', (error: Error) => {
        tries++
        if (tries === 3) reject(error)
      })

      this.joinedRoom()
    })
  }

  set onJoinedRoom(callback: typeof this.onJoinedRoomCallback) {
    this.onJoinedRoomCallback = callback
  }

  set onRemoteTrackInfo(callback: typeof this.onRemoteTrackInfoCallback) {
    this.onRemoteTrackInfoCallback = callback
  }

  set onRemoteTrack(callback: typeof this.onRemoteTrackCallback) {
    this.onRemoteTrackCallback = callback
  }

  set getLocalCover(callback: typeof this.getLocalCoverCallback) {
    this.getLocalCoverCallback = callback
  }

  set onRemoteCover(callback: typeof this.onRemoteCoverCallback) {
    this.onRemoteCoverCallback = callback
  }

  set onRemoteStream(callback: typeof this.onRemoteStreamCallback) {
    this.onRemoteStreamCallback = callback
  }

  set getLocalSong(callback: typeof this.getLocalSongCallback) {
    this.getLocalSongCallback = callback
  }

  set fetchCurrentSong(callback: typeof this.getCurrentSongIndex) {
    this.getCurrentSongIndex = callback
  }

  set onSeek(callback: typeof this.onSeekCallback) {
    this.onSeekCallback = callback
  }

  set onPlayerStateChange(callback: typeof this.onPlayerStateChangeCallback) {
    this.onPlayerStateChangeCallback = callback
  }

  set peerConnectionStateHandler(callback: typeof this.onPeerStateChangeCallback) {
    this.onPeerStateChangeCallback = callback
  }

  set onDataSent(callback: typeof this.onDataSentCallback) {
    this.onDataSentCallback = callback
  }

  set onAllReady(callback: typeof this.onAllReadyCallback) {
    this.onAllReadyCallback = callback
  }

  set onReadyRequested(callback: typeof this.onReadyRequestedCallback) {
    this.onReadyRequestedCallback = callback
  }

  set onReadyEmitted(callback: typeof this.onReadyEmittedCallback) {
    this.onReadyEmittedCallback = callback
  }

  set getRequestedSong(callback: typeof this.playRequestedSongCallback) {
    this.playRequestedSongCallback = callback
  }

  set onQueueOrderChange(callback: typeof this.onQueueOrderChangeCallback) {
    this.onQueueOrderChangeCallback = callback
  }

  set onQueueDataChange(callback: typeof this.onQueueDataChangeCallback) {
    this.onQueueDataChangeCallback = callback
  }

  set onRepeatChange(callback: typeof this.onRepeatChangeCallback) {
    this.onRepeatChangeCallback = callback
  }

  private handleCoverChannel(channel: RTCDataChannel) {
    const fragmentReceiver = new FragmentReceiver(this.onRemoteCoverCallback)

    channel.onmessage = (event) => {
      fragmentReceiver.receive(event.data)
    }
  }

  private handleStreamChannel(channel: RTCDataChannel) {
    const fragmentReceiver = new FragmentReceiver(this.onRemoteStreamCallback)

    channel.onmessage = (event) => {
      fragmentReceiver.receive(event.data)
    }
  }

  private addRemoteCandidate() {
    this.socketConnection?.on('candidate', (id: string, candidate: RTCIceCandidate) => {
      this.peerConnection[id].peer?.addIceCandidate(new RTCIceCandidate(candidate))
    })
  }

  private onLocalCandidate(id: string, peer: RTCPeerConnection) {
    peer.onicecandidate = (event) => {
      if (event.candidate) {
        this.socketConnection?.emit('candidate', id, event.candidate)
      }
    }
  }

  private joinedRoom() {
    this.socketConnection?.on('joinedRoom', (roomID: string, isCreator: boolean) => {
      this.onJoinedRoomCallback?.(roomID, isCreator)
    })
  }

  private listenPeerConnected(id: string, peer: RTCPeerConnection) {
    peer.onconnectionstatechange = (e) => {
      if (this.onPeerStateChangeCallback) {
        switch ((e.target as RTCPeerConnection).connectionState) {
          case 'connected':
            this.onPeerStateChangeCallback(id, peerConnectionState.CONNECTED)
            break
          case 'disconnected':
          case 'failed':
            this.onPeerStateChangeCallback(id, peerConnectionState.DISCONNECTED)
        }
      }
    }
  }

  private onDataSentHandler(id: string) {
    // TODO: Show state of each user on ui
    this.onDataSentCallback ? this.onDataSentCallback(id) : null
  }

  private sendStream(id: string, stream: ArrayBuffer | null, channel: RTCDataChannel) {
    if (channel.readyState === 'open') {
      try {
        const fragmentSender = new FragmentSender(stream, channel, () => this.onDataSentHandler(id))
        fragmentSender.send()
      } catch (e) {
        console.trace(e)
      }
      return
    }
    console.debug('data channel', channel.label, 'is in state: ', this.peerConnection[id].streamChannel?.readyState)
  }

  public playSong(index: number) {
    console.debug('Playing song', index)
    this.sendSongDetails(index)
  }

  public createRoom() {
    this.socketConnection?.emit('createRoom')
  }

  private onStream(id: string, peer: RTCPeerConnection) {
    peer.ontrack = (event: RTCTrackEvent) => {
      if (this.mode === PeerMode.WATCHER && id === this.BroadcasterID) {
        this.onRemoteTrackCallback?.(event)
      }
    }
  }

  private onDataChannel(id: string, peer: RTCPeerConnection) {
    peer.ondatachannel = (event) => {
      const channel = event.channel
      if (channel.label === 'cover-channel') {
        this.handleCoverChannel(channel)
        this.setCoverChannel(id, channel)
      } else if (channel.label === 'stream-channel') {
        this.handleStreamChannel(channel)
        this.setStreamChannel(id, channel)
      }
    }
  }

  public joinRoom(id: string) {
    this.socketConnection?.emit('joinRoom', id)
  }

  /**
   * Requests ready from all peers in room
   * Should be called after trackChange when playerState is set to LOADING
   * [Broadcaster method]
   */
  public requestReadyStatus() {
    if (Object.keys(this.peerConnection).length === 0) {
      this.checkAllReady()
      return
    }
    this.isListeningReady = true
    this.socketConnection?.emit('requestReady')

    console.debug('Requesting ready status from peers')
  }

  /**
   * Listen for readyRequest from Broadcaster
   * The receiving peer should then emit "ready" whenever it has the required song
   * [Watcher Method]
   */
  private listenReadyRequest() {
    this.socketConnection?.on('requestReady', () => {
      this.onReadyRequestedCallback?.()
    })
  }

  private sendSongBuffer(id: string, songID: string) {
    const channel = this.peerConnection[id].streamChannel
    if (channel) {
      this.getLocalSongCallback?.(songID).then((buf) => this.sendStream(id, buf, channel))
    }
  }

  private sendCoverBuffer(id: string, songID: string) {
    const channel = this.peerConnection[id].streamChannel

    if (channel) {
      this.getLocalCoverCallback?.(songID).then((buf) => this.sendStream(id, buf, channel))
    }
  }

  /**
   * Listen to events related to fetching of song and cover
   */
  private listenBufferRequests() {
    this.socketConnection?.on('requestedSong', (id: string, songID: string) => {
      console.debug('Song was requested', id, songID)
      this.sendSongBuffer(id, songID)
    })

    this.socketConnection?.on('requestedCover', (id: string, songID: string) => {
      this.sendCoverBuffer(id, songID)
    })
  }

  /**
   * Listens play requests emitted by websocket server
   * The peer receiving this method should become the broadcaster and trackChange to the song
   * [Broadcaster method]
   */
  private listenPlayRequests() {
    this.socketConnection?.on('requestedPlay', (songIndex: number) => {
      this.playRequestedSongCallback?.(songIndex)
    })
  }

  /**
   * Listens for track change
   */
  private listenTrackChange() {
    this.socketConnection?.on('onTrackChange', (from: string, song_index: number) => {
      this.onRemoteTrackInfoCallback?.(from, song_index)
    })
  }

  /**
   * Listens to ready emitted by peer in room
   * [Broadcaster Method]
   */
  private listenPeerReady() {
    this.socketConnection?.on('ready', (id: string) => {
      console.debug('Got ready call from', id)
      if (this.isListeningReady) {
        this.setPeerReady(id)
        this.checkAllReady()
      }
    })
  }

  /**
   * Listens to check if the broadcaster of current song has emitted allReady
   * [Watcher method]
   */
  private listenAllReady() {
    this.socketConnection?.on('allReady', () => {
      this.onAllReadyCallback?.()
      this.onPlayerStateChangeCallback?.('PLAYING')
    })
  }

  /**
   * Checks if all peers in room emitted ready
   * [Broadcaster method]
   * TODO: Add a timeout after which allReady will be emitted irrespective of who emitted ready
   */
  private checkAllReady() {
    if (this.readyPeers.length === Object.keys(this.peerConnection).length) {
      this.socketConnection?.emit('allReady')
      this.readyPeers = []
      this.isListeningReady = false
      this.onPlayerStateChangeCallback?.('PLAYING')
      this.onAllReadyCallback?.()
    }
  }

  /**
   * Listens to websocket events for change in playerState
   */
  private listenPlayerState() {
    this.socketConnection?.on('playerStateChange', (state: PlayerState) => {
      this.onPlayerStateChangeCallback?.(state)
    })

    this.socketConnection?.on('onRepeatChange', (repeat: RepeatState) => {
      this.onRepeatChangeCallback?.(repeat)
    })
  }

  /**
   * Listens to websocket events for seek
   */
  private listenSeek() {
    this.socketConnection?.on('forceSeek', (time: number) => {
      this.onSeekCallback?.(time)
    })
  }

  private listenQueueUpdate() {
    this.socketConnection?.on('onQueueOrderChange', (order: QueueOrder, index: number) => {
      this.onQueueOrderChangeCallback?.(order, index)
    })

    this.socketConnection?.on('onQueueDataChange', (data: QueueData<RemoteSong>) => {
      this.onQueueDataChangeCallback?.(data)
    })
  }

  /**
   * Requests song (File/Buffer) from peer
   * @param id of peer to get song from
   * @param songID id of song
   */
  public requestSong(id: string, songID: string) {
    this.socketConnection?.emit('requestSong', id, songID)
  }

  /**
   * Requests cover from peer
   * @param id of peer to get cover from
   * @param songID id of song to which track belongs to
   */
  public requestCover(id: string, songID: string) {
    this.socketConnection?.emit('requestCover', id, songID)
  }

  public emitRepeat(repeat: RepeatState) {
    this.socketConnection?.emit('repeatChange', repeat)
  }

  public emitQueueChange(order: QueueOrder, data: QueueData<RemoteSong>, index: number) {
    this.socketConnection?.emit('queueChange', order, data, index)
  }

  /**
   * Emits ready
   */
  public emitReady() {
    console.debug('Socket emitting ready')
    this.socketConnection?.emit('ready')
    this.onReadyEmittedCallback ? this.onReadyEmittedCallback() : null
  }

  /**
   * Emits player state to all peers in room
   * Should be called when local player is set to pause/play or unloads audio
   * @param state
   */
  public emitPlayerState(state: PlayerState) {
    this.socketConnection?.emit('playerStateChange', state)
  }

  /**
   * Sends time to all peers in room.
   * Should be called after local player seeks to a time
   * @param time
   */
  public emitSeek(time: number) {
    this.socketConnection?.emit('forceSeek', time)
  }

  /**
   * Send song details to room over websocket
   * @param trackInfo
   * @param song_index index of song in room queue
   */
  private sendSongDetails(songIndex: number) {
    this.socketConnection?.emit('trackChange', songIndex)
  }

  private setPeerReady(id: string) {
    if (!this.readyPeers.includes(id)) {
      this.readyPeers.push(id)
    }
  }

  /**
   * Signalling methods
   */

  public start() {
    this.addRemoteCandidate()
    this.onOffer()
    this.onUserJoined()
    this.onAnswer()
    this.listenPeerReady()
    this.listenBufferRequests()
    this.listenTrackChange()
    this.listenQueueUpdate()
    this.listenPlayerState()
    this.listenSeek()
    this.listenReadyRequest()
    this.listenAllReady()
    this.listenPlayRequests()
  }

  private makePeer(id: string): RTCPeerConnection {
    // Creates new peer
    const peer = new RTCPeerConnection({ iceServers: [STUN, TURN] })

    // Report changes to connection state
    this.listenPeerConnected(id, peer)
    if (this.onPeerStateChangeCallback) this.onPeerStateChangeCallback(id, peerConnectionState.CONNECTING)

    this.onLocalCandidate(id, peer)
    return peer
  }

  private setPeer(id: string, peer: RTCPeerConnection) {
    if (this.peerConnection[id]) this.peerConnection[id].peer = peer
    else this.peerConnection[id] = { peer: peer }
  }
  private setCoverChannel(id: string, coverChannel: RTCDataChannel) {
    if (this.peerConnection[id]) this.peerConnection[id].coverChannel = coverChannel
    else this.peerConnection[id] = { coverChannel: coverChannel }
  }

  private setStreamChannel(id: string, streamChannel: RTCDataChannel) {
    if (this.peerConnection[id]) this.peerConnection[id].streamChannel = streamChannel
    else this.peerConnection[id] = { streamChannel: streamChannel }
  }

  private makeDataChannel(id: string, peer: RTCPeerConnection) {
    const coverChannel = peer.createDataChannel('cover-channel')
    const streamChannel = peer.createDataChannel('stream-channel')

    this.handleCoverChannel(coverChannel)
    this.handleStreamChannel(streamChannel)
    this.setCoverChannel(id, coverChannel)
    this.setStreamChannel(id, streamChannel)
  }

  private onOffer() {
    this.socketConnection?.on('offer', (id: string, description: RTCSessionDescription) => {
      this.setupWatcher(id, description)
    })
  }

  private onUserJoined() {
    this.socketConnection?.on('userJoined', (id: string) => {
      this.onPlayerStateChange ? this.onPlayerStateChange('PAUSED') : null
      this.setupInitiator(id)
      this.requestReadyStatus()
    })
  }

  private listenSignalingState(id: string, peer: RTCPeerConnection): void {
    peer.onsignalingstatechange = (e) => {
      this.isNegotiating[id] = (e.target as RTCPeerConnection).signalingState !== 'stable'
    }
  }

  private setupInitiator(id: string) {
    const peer = this.makePeer(id)
    this.listenSignalingState(id, peer)
    this.makeDataChannel(id, peer)

    this.needsNegotiation(id, peer)

    this.setPeer(id, peer)
  }

  private makeOffer(id: string, peer: RTCPeerConnection) {
    // Send offer to signalling server
    peer
      .createOffer()
      .then((sdp) => peer.setLocalDescription(sdp))
      .then(() => this.socketConnection?.emit('offer', id, peer.localDescription))
  }

  private onAnswer() {
    this.socketConnection?.on('answer', (id: string, description: RTCSessionDescription) => {
      if (this.isNegotiating[id]) this.peerConnection[id].peer?.setRemoteDescription(description)
    })
  }

  private needsNegotiation(id: string, peer: RTCPeerConnection) {
    peer.onnegotiationneeded = () => {
      if (!this.isNegotiating[id]) {
        this.isNegotiating[id] = true
        this.makeOffer(id, peer)
      }
    }
  }

  private setupWatcher(id: string, description: RTCSessionDescription) {
    let peer: RTCPeerConnection

    const existingPeer = this.peerConnection[id]?.peer
    if (existingPeer) peer = existingPeer
    else peer = this.makePeer(id)

    this.listenSignalingState(id, peer)
    this.onDataChannel(id, peer)
    this.onStream(id, peer)

    peer
      .setRemoteDescription(description)
      .then(() => peer.createAnswer())
      .then((sdp) => peer.setLocalDescription(sdp))
      .then(() => this.socketConnection?.emit('answer', id, peer.localDescription))

    this.setPeer(id, peer)
  }
}
