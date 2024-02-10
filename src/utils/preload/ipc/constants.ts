/*
 *  constants.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

export enum IpcEvents {
  SCANNER = 'scanner',
  SONG = 'song',
  ALBUM = 'album',
  ARTIST = 'artist',
  PLAYLIST = 'playlist',
  GENRE = 'genre',
  CONTEXT_MENU = 'contextMenu',
  BROWSER_WINDOWS = 'browserWindows',
  PREFERENCES = 'preferences',
  SEARCH = 'search',
  STORE = 'store',
  SERVICE_PROVIDERS = 'serviceProviders',
  LOGGER = 'logger',
  NOTIFIER = 'notifier',
  EXTENSION_HOST = 'extensionHost',
  UPDATE = 'update',
  MPRIS = 'mpris',
  SPOTIFY = 'spotify',
  RODIO = 'rodio',
}

export enum StoreEvents {
  SET_DATA = 'setData',
  GET_DATA = 'getData',
  SET_SECURE = 'setSecure',
  GET_SECURE = 'getSecure',
  REMOVE_SECURE = 'removeSecure',
}

export enum SearchEvents {
  SEARCH_SONGS_BY_OPTIONS = 'searchSongsByOptions',
  SEARCH_ENTITY_BY_OPTIONS = 'searchEntityByOptions',
  SEARCH_ALL = 'searchAll',
  SEARCH_YT = 'searchYT',
  YT_SUGGESTIONS = 'YTSuggestions',
  GET_YT_AUDIO_URL = 'getYTAudioURL',
  SCRAPE_LASTFM = 'scrapeLastFM',
  SCRAPE_LYRICS = 'scrapeLyrics',
  REQUEST_INVIDIOUS = 'requestInvidious',
  GET_YT_PLAYLIST = 'getYTPlaylist',
  GET_YT_PLAYLIST_CONTENT = 'getYTPlaylistContent',
  GET_PLAY_COUNT = 'getPlayCount',
}

export enum PreferenceEvents {
  SAVE_SELECTIVE_PREFERENCES = 'saveSelectivePreferences',
  LOAD_SELECTIVE_PREFERENCES = 'loadSelectivePreferences',
  LOAD_SELECTIVE_ARRAY = 'loadSelectiveArray',
  PREFERENCE_REFRESH = 'preferenceRefresh',
  SET_THEME = 'setTheme',
  GET_THEME = 'getTheme',
  REMOVE_THEME = 'removeTheme',
  SET_ACTIVE_THEME = 'setActiveTheme',
  GET_ACTIVE_THEME = 'getActiveTheme',
  GET_ALL_THEMES = 'getAllThemes',
  SET_SONG_VIEW = 'setSongView',
  GET_SONG_VIEW = 'getSongView',
  THEME_REFRESH = 'themeRefresh',
  SONG_VIEW_REFRESH = 'songViewRefresh',
  SET_LANGUAGE = 'setLanguage',
  LANGUAGE_REFRESH = 'languageRefresh',
  LISTEN_PREFERENCE = 'listenPreference',
  RESET_TO_DEFAULT = 'resetToDefault',
  TRANSFORM_CSS = 'transformCSS',
  PACK_THEME = 'packTheme',
  IMPORT_THEME = 'importTheme',
  GENERATE_ICON = 'generateIcon',
  SET_TEMP_THEME = 'setTempTheme',
}

export enum WindowEvents {
  TOGGLE_DEV_TOOLS = 'toggleDevTools',
  OPEN_FILE_BROWSER = 'openFileBrowser',

  CLOSE_MAIN = 'closeMain',
  MAX_MAIN = 'maxMain',
  MIN_MAIN = 'minMain',

  OPEN_WIN = 'openPref',
  CLOSE_WIN = 'closePref',
  MAX_WIN = 'maxPref',
  MIN_WIN = 'minPref',
  OPEN_URL_EXTERNAL = 'openUrlExternal',
  REGISTER_OAUTH_CALLBACK = 'registerOAuthCallback',
  DEREGISTER_OAUTH_CALLBACK = 'deregisterOAuthCallback',
  TRIGGER_OAUTH_CALLBACK = 'triggerOauthCallback',
  MAIN_WINDOW_HAS_MOUNTED = 'mainWindowsHasMounted',

  DRAG_FILE = 'dragFile',

  IS_MAXIMIZED = 'isMaximized',
  HAS_FRAME = 'hasFrame',
  SHOW_TITLEBAR_ICONS = 'showTitlebarIcons',

  GOT_EXTRA_ARGS = 'gotExtraArgs',

  RESTART_APP = 'restartApp',

  UPDATE_ZOOM = 'updateZoom',

  GET_PLATFORM = 'getPlatform',

  TOGGLE_FULLSCREEN = 'toggleFullscreen',
  ENABLE_FULLSCREEN = 'enableFullscreen',
  DISABLE_FULLSCREEN = 'disableFullscreen',

  HANDLE_RELOAD = 'handleReload',
}

export enum AlbumEvents {
  GET_ALL_ALBUMS = 'getAlbums',
  GET_ALBUM = 'getAlbum',
}

export enum GenreEvents {
  GET_ALL_GENRE = 'getAllGenre',
  GET_GENRE = 'getGenre',
}

export enum ScannerEvents {
  SCAN_MUSIC = 'scanMusic',
  SCAN_SINGLE_SONG = 'scanSingleSong',
  SCAN_SINGLE_PLAYLIST = 'scanSinglePlaylist',
  RESET_SCAN_TASK = 'resetScanTask',
  GET_PROGRESS = 'getProgress',
  GET_RECOMMENDED_CPUS = 'getRecommendedCpus',

  PROGRESS_CHANNEL = 'progressChannel',
}

export enum PlaylistEvents {
  CREATE_PLAYLIST = 'createPlaylist',
  UPDATE_PLAYLIST = 'updatePlaylist',
  ADD_TO_PLAYLIST = 'AddToPlaylist',
  REMOVE_FROM_PLAYLIST = 'removeFromPlaylist',
  GET_ALL_PLAYLISTS = 'getPlaylists',
  GET_PLAYLIST = 'getPlaylist',
  ADDED_PLAYLIST = 'addedPlaylist',
  REMOVE_PLAYLIST = 'removePlaylist',
  SAVE_COVER = 'saveCover',
  EXPORT = 'export',
}

export enum ArtistEvents {
  GET_ALL_ARTISTS = 'getArtists',
  GET_ARTIST = 'getArtist',
}

export enum SongEvents {
  GET_ALL_SONGS = 'getAllSongs',
  STORE_SONG = 'storeSong',
  REMOVE_SONG = 'removeSong',
  UPDATE_SONG = 'updateSong',
  UPDATE_ALBUM = 'updateAlbum',
  UPDATE_ARTIST = 'updateArtist',
  UPDATE_LYRICS = 'updateLyrics',
  SAVE_AUDIO_TO_FILE = 'saveAudioToFile',
  SAVE_IMAGE_TO_FILE = 'saveImageToFile',
  AUDIO_EXISTS = 'fileExists',
  IMAGE_EXISTS = 'imageExists',
  GOT_FILE_PATH = 'gotSongPath',

  INCREMENT_PLAY_COUNT = 'incrementPlayCount',
  INCREMENT_PLAY_TIME = 'incrementPlayTime',
}

export enum LoggerEvents {
  INFO = 'info',
  ERROR = 'error',
  DEBUG = 'debug',
  WARN = 'warn',
  TRACE = 'trace',
  WATCH_LOGS = 'watchLogs',
  UNWATCH_LOGS = 'unwatchLogs',
  TOGGLE_DEBUG = 'toggleDebug',
}

export enum SpotifyEvents {
  CONNECT = 'connect',
  LISTEN_EVENT = 'listenEvent',
  REMOVE_EVENT = 'removeEvent',
  COMMAND = 'command',
  CLOSE = 'close',
  GET_TOKEN = 'getToken',
}

export enum NotifierEvents {
  WATCH_FILE_CHANGES = 'watchFileChanges',
  FILE_CHANGED = 'fileChanged',
}

export enum MprisEvents {
  PLAYBACK_STATE_CHANGED = 'playbackStateChanged',
  SONG_INFO_CHANGED = 'songInfoChanged',
  BUTTON_STATUS_CHANGED = 'buttonStatusChanged',
  POSITION_CHANGED = 'positionChanged',
  ON_BUTTON_PRESSED = 'onButtonPressed',
}

export enum RodioEvents {
  INITIALIZE = 'initialize',
  SET_SRC = 'setSrc',
  PLAY = 'play',
  PAUSE = 'pause',
  STOP = 'stop',
  SET_VOLUME = 'setVolume',
  GET_VOLUME = 'getVolume',
  GET_POSITION = 'getPosition',
  SEEK = 'seek',

  ON_PLAY = 'onPlay',
  ON_PAUSE = 'onPause',
  ON_STOP = 'onStop',
  ON_TIME_UPDATE = 'onTimeUpdate',
  ON_LOADED = 'onLoaded',
  ON_ENDED = 'onEnded',
  ON_ERROR = 'onError',
}

export enum ExtensionHostEvents {
  GET_ALL_EXTENSIONS = 'getAllExtensions',
  INSTALL = 'install',
  EXTENSION_REQUESTS = 'extensionRequests',
  TOGGLE_EXT_STATUS = 'toggleExtStatus',
  REMOVE_EXT = 'removeExt',
  GET_EXTENSION_ICON = 'getExtensionIcon',
  SEND_EXTRA_EVENT = 'sendExtraEvent',
  DOWNLOAD_EXTENSION = 'downloadExtension',
  EXT_INSTALL_STATUS = 'extInstallStatus',
  GET_EXT_CONTEXT_MENU = 'getExtContextMenu',
  ON_CONTEXT_MENU_ITEM_CLICKED = 'onContextMenuItemClicked',
  ON_ACCOUNT_REGISTERED = 'onAccountRegistered',
  GET_REGISTERED_ACCOUNTS = 'getRegisteredAccounts',
  PERFORM_ACCOUNT_LOGIN = 'performAccountLogin',
  ON_EXTENSIONS_CHANGED = 'onExtensionsChanged',
  GET_EXTENSION_PROVIDER_SCOPES = 'getExtensionProviderScopes',
  GET_DISPLAY_NAME = 'getDisplayName',
}

export enum ServiceProviderEvents {
  LOGIN = 'login',
}

export enum UpdateEvents {
  CHECK_UPDATES = 'checkUpdates',
  GOT_UPDATE = 'gotUpdate',
  UPDATE_NOW = 'updateNow',
}

export enum EventBus {
  UPDATE_AUDIO_TIME = 'timestamp-update',
  SONG_SELECTED = 'song-select',
  COVER_SELECTED = 'cover-select',
  SHOW_NEW_PLAYLIST_MODAL = 'show-new-playlist-modal',
  SHOW_DELETE_MODAL = 'show-delete-modal',
  SHOW_SONG_FROM_URL_MODAL = 'show-song-from-url',
  SHOW_PLAYLIST_FROM_URL_MODAL = 'show-playlist-from-url',
  SHOW_SETUP_MODAL = 'show-setup-modal',
  SHOW_SONG_INFO_MODAL = 'show-song-info-modal',
  SHOW_ENTITY_INFO_MODAL = 'show-entity-info-modal',
  SHOW_OAUTH_MODAL = 'show-oauth-modal',
  SHOW_PIN_ENTRY_MODAL = 'show-pin-entry-modal',
  HIDE_OAUTH_MODAL = 'hide-oauth-modal',
  SHOW_FORM_MODAL = 'show-form-modal',
  SHOW_INCORRECT_PLAYBACK_MODAL = 'show-incorrect-playback-modal',
  REFRESH_ACCOUNTS = 'refresh-accounts',
  REFRESH_PAGE = 'refresh-page',
  REFRESH_LYRICS = 'refresh-lyrics',
  UPDATE_OPTIONAL_PROVIDER = 'update-optional-provider',
  FORCE_LOAD_SONG = 'force-load-song',
  IGNORE_MUSIC_INFO_SCROLL = 'ignore-music-info-scroll',
}
