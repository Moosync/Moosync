/*
 *  HtmlAudioElement.d.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

interface HTMLAudioElement {
  // https://developer.mozilla.org/en-US/docs/Web/API/HTMLMediaElement/captureStream
  captureStream(frameRate?: number): MediaStream
}
