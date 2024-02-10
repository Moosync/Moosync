/*
 *  models.d.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

interface marshaledSong {
  _id: string
  path?: string
  size?: number
  title: string
  song_coverPath_low?: string
  song_coverPath_high?: string
  album_id?: string
  album_name?: string
  album_coverPath_high?: string
  album_coverPath_low?: string
  album_song_count?: number
  album_extra_info?: string
  album_artist?: string
  lyrics?: string
  artists?: string
  artist_name?: string
  artists_id?: string
  artist_coverPath?: string
  genre_name?: string
  genere_id?: string
  date?: string
  year?: number | string
  album_year?: number
  track_no?: number
  bitrate?: number
  codec?: string
  container?: string
  duration: number
  sampleRate?: number
  hash?: string
  inode?: string
  deviceno?: string
  url?: string
  playbackUrl?: string
  date_added: number
  type: PlayerTypes
  icon?: string
  provider_extension?: string
  play_count?: number
  show_in_library: number
}

interface stats {
  path: string
  size: number
  inode: string
  deviceno: string
}

interface image {
  path: string
  data: Buffer
}

interface ThemeDetails {
  id: string
  name: string
  author: string
  theme: ThemeItem
}

type ThemeKey =
  | 'primary'
  | 'secondary'
  | 'tertiary'
  | 'textPrimary'
  | 'textSecondary'
  | 'textInverse'
  | 'accent'
  | 'divider'

type ThemeItem = {
  [key in ThemeKey]: string
} & {
  customCSS?: string
}
