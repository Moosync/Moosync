type LogLines = {
  id: number
  time: string
  level: LogLevels
  process: string
  message: string
}

interface InvidiousSong extends Song {
  invidiousPlaybackUrl?: string
}

interface ExtendedPlaylist extends Playlist {
  extension?: string
  isLocal: boolean
}

type StrippedAccountDetails = Omit<Omit<AccountDetails, 'signinCallback'>, 'signoutCallback'>

type ExtendedLoginModalData = LoginModalOptions & {
  packageName: string
}

type Progress = { total: number; current: number; status: import('@/utils/commonConstants').ScanStatus }

declare namespace NodeJS {
  export interface ProcessEnv {
    ELECTRON_NODE_INTEGRATION: boolean
    MOOSYNC_VERSION: string
    DEBUG_LOGGING: boolean
    APPIMAGE: string
    YoutubeClientID: string
    YoutubeClientSecret: string
    LastFmApiKey: string
    LastFmSecret: string
    SpotifyClientID: string
    SpotifyClientSecret: string
    FanartTVApiKey: string
  }
}

type ValueOf<T> = T[keyof T]
type Optional<T, K extends keyof T> = Pick<Partial<T>, K> & Omit<T, K>
type Unpromise<T> = T extends Promise<infer U> ? U : never
