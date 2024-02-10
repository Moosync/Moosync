/*
 *  dataFragmenter.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

const chunkLimit = 16384

export class FragmentSender {
  private data: ArrayBuffer
  private byteLength: number
  private offset: number
  private byteEnd: number

  private channel: RTCDataChannel

  private onDataSentCallback?: () => void

  constructor(data: ArrayBuffer | undefined | null, channel: RTCDataChannel, callback?: () => void) {
    if (data === null || data === undefined) {
      throw new Error('null data')
    }

    this.channel = channel
    this.data = data
    this.byteLength = this.data.byteLength
    this.offset = 0
    this.byteEnd = this.byteLength <= chunkLimit ? this.byteLength : chunkLimit
    this.onDataSentCallback = callback
  }

  private sendData() {
    if (this.offset < this.byteLength - 1) {
      this.channel?.send(this.data.slice(this.offset, this.byteEnd))

      console.debug('sending ', this.data.slice(this.offset, this.byteEnd))

      this.offset = this.byteEnd
      this.byteEnd = this.offset + chunkLimit

      if (this.offset >= this.byteLength) {
        this.channel.send(JSON.stringify({ type: 'end' }))
        this.channel.onbufferedamountlow = null
        this.onDataSentCallback ? this.onDataSentCallback() : null
      }

      if (this.byteEnd >= this.byteLength) {
        this.byteEnd = this.byteLength
      }
    }
  }

  public send() {
    if (this.data) {
      const header = JSON.stringify({ type: 'byteLength', message: this.byteLength })

      this.channel.send(header)
      this.channel.bufferedAmountLowThreshold = chunkLimit - 1
      this.channel.onbufferedamountlow = () => {
        this.sendData()
      }

      this.sendData()
    }
  }
}

export class FragmentReceiver {
  private file: BlobPart[] = []
  private byteLength = 0

  private onDataReceivedCallback?: (data: Blob) => void

  constructor(callback?: (data: Blob) => void) {
    this.onDataReceivedCallback = callback
  }

  private endTransfer() {
    this.onDataReceivedCallback ? this.onDataReceivedCallback(new Blob(this.file)) : null
    this.file = []
  }

  public receive(data: ArrayBuffer) {
    if (typeof data === 'string') {
      const tmp = JSON.parse(data) as fragmentedData
      switch (tmp.type) {
        case 'end':
          // TODO: End using byteLength instead
          this.endTransfer()
          break
        case 'byteLength':
          this.byteLength = tmp.message as number
          break
      }
    } else {
      this.file.push(data)
    }
  }
}
