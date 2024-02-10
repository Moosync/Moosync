/*
 *  sync.d.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

interface fragmentedData {
  type: string
  message: Buffer | number
}

interface RemoteSong extends Song {
  senderSocket: string
}
