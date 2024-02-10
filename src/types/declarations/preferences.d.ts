/*
 *  preferences.d.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

type togglePaths = { path: string; enabled: boolean }[]

type songMenu = 'compact' | 'classic'

type SystemSettings = Checkbox

interface Preferences {
  isFirstLaunch: boolean
  musicPaths: togglePaths
  exclude_musicPaths: togglePaths
  thumbnailPath: string
  artworkPath: string
  youtubeAlt: Checkbox[]
  youtubeOptions: Checkbox[]
  youtube?: {
    client_id?: string
    client_secret?: string
  }
  invidious: Checkbox[]
  invidious_instance?: string
  spotify?: {
    client_id?: string
    client_secret?: string
    options?: {
      use_librespot?: boolean
    }
  }
  piped_instance?: string
  system_language?: Checkbox[]
  audio: Checkbox[]
  system: SystemSettings[]
  themes: { [key: string]: ThemeDetails }
  activeTheme: string
  hotkeys: HotkeyPair[]
  zoomFactor: string
  logs: Checkbox[]
  lyrics_fetchers: Checkbox[]
}

type HotkeyPair = {
  key: KeyboardEvent['code'][][]
  value: import('@/utils/commonConstants').HotkeyEvents
}
