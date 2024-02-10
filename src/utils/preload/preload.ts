// /*
//  *  preload.ts is a part of Moosync.
//  *
//  *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
//  *  Licensed under the GNU General Public License.
//  *
//  *  See LICENSE in the project root for license information.
//  */

// import {
//   ExtensionHostEvents,
//   IpcEvents,
//   LoggerEvents,
//   MprisEvents,
//   NotifierEvents,
//   PlaylistEvents,
//   PreferenceEvents,
//   RodioEvents,
//   ScannerEvents,
//   SearchEvents,
//   SongEvents,
//   StoreEvents,
//   UpdateEvents,
//   WindowEvents,
//   SpotifyEvents
// } from './ipc/constants'

// import { IpcRendererHolder } from '@/utils/preload/ipc/index'
// import { LogLevelDesc } from 'loglevel'

// const ipcRendererHolder = new IpcRendererHolder(ipcRenderer)

// contextBridge.exposeInMainWorld('DBUtils', {
//   createPlaylist: (playlist: Partial<Playlist>) =>
//     ipcRendererHolder.send<PlaylistRequests.CreatePlaylist>(IpcEvents.PLAYLIST, {
//       type: PlaylistEvents.CREATE_PLAYLIST,
//       params: { playlist },
//     }),

//   updatePlaylist: (playlist: Partial<Playlist>) =>
//     ipcRendererHolder.send<PlaylistRequests.CreatePlaylist>(IpcEvents.PLAYLIST, {
//       type: PlaylistEvents.UPDATE_PLAYLIST,
//       params: { playlist },
//     }),

//   addToPlaylist: (playlistID: string, ...songIDs: Song[]) =>
//     ipcRendererHolder.send<PlaylistRequests.AddToPlaylist>(IpcEvents.PLAYLIST, {
//       type: PlaylistEvents.ADD_TO_PLAYLIST,
//       params: { playlist_id: playlistID, song_ids: songIDs },
//     }),

//   removeFromPlaylist: (playlistID: string, ...songIDs: Song[]) =>
//     ipcRendererHolder.send<PlaylistRequests.AddToPlaylist>(IpcEvents.PLAYLIST, {
//       type: PlaylistEvents.REMOVE_FROM_PLAYLIST,
//       params: { playlist_id: playlistID, song_ids: songIDs },
//     }),

//   removePlaylist: (playlist: Playlist) =>
//     ipcRendererHolder.send<PlaylistRequests.RemoveExportPlaylist>(IpcEvents.PLAYLIST, {
//       type: PlaylistEvents.REMOVE_PLAYLIST,
//       params: { playlist },
//     }),

//   exportPlaylist: (playlist: Playlist) =>
//     ipcRendererHolder.send<PlaylistRequests.RemoveExportPlaylist>(IpcEvents.PLAYLIST, {
//       type: PlaylistEvents.EXPORT,
//       params: { playlist },
//     }),

//   storeSongs: (songs: Song[]) =>
//     ipcRendererHolder.send<SongRequests.Songs>(IpcEvents.SONG, {
//       type: SongEvents.STORE_SONG,
//       params: { songs: songs },
//     }),

//   updateSongs: (songs: Song[]) =>
//     ipcRendererHolder.send<SongRequests.Songs>(IpcEvents.SONG, {
//       type: SongEvents.UPDATE_SONG,
//       params: { songs: songs },
//     }),

//   updateArtist: (artist: Artists) =>
//     ipcRendererHolder.send<SongRequests.UpdateArtist>(IpcEvents.SONG, {
//       type: SongEvents.UPDATE_ARTIST,
//       params: { artist },
//     }),

//   updateAlbum: (album: Album) =>
//     ipcRendererHolder.send<SongRequests.UpdateAlbum>(IpcEvents.SONG, {
//       type: SongEvents.UPDATE_ALBUM,
//       params: { album },
//     }),

//   removeSongs: (songs: Song[]) =>
//     ipcRendererHolder.send<SongRequests.Songs>(IpcEvents.SONG, {
//       type: SongEvents.REMOVE_SONG,
//       params: { songs: songs },
//     }),

//   updateLyrics: (id: string, lyrics: string) =>
//     ipcRendererHolder.send<SongRequests.Lyrics>(IpcEvents.SONG, {
//       type: SongEvents.UPDATE_LYRICS,
//       params: { id, lyrics },
//     }),

//   incrementPlayCount: (song_id: string) =>
//     ipcRendererHolder.send<SongRequests.PlayCount>(IpcEvents.SONG, {
//       type: SongEvents.INCREMENT_PLAY_COUNT,
//       params: { song_id },
//     }),

//   incrementPlayTime: (song_id: string, duration: number) =>
//     ipcRendererHolder.send<SongRequests.PlayTime>(IpcEvents.SONG, {
//       type: SongEvents.INCREMENT_PLAY_TIME,
//       params: { song_id, duration },
//     }),
// })

// contextBridge.exposeInMainWorld('PreferenceUtils', {
//   saveSelective: (key: string, value: unknown, isExtension?: boolean) =>
//     ipcRendererHolder.send<PreferenceRequests.Save>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.SAVE_SELECTIVE_PREFERENCES,
//       params: { key, value, isExtension },
//     }),

//   loadSelective: <T>(key: string, isExtension?: boolean, defaultValue?: T) =>
//     ipcRendererHolder.send<PreferenceRequests.Load>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.LOAD_SELECTIVE_PREFERENCES,
//       params: { key, isExtension, defaultValue },
//     }),

//   loadSelectiveArrayItem: <T>(key: string, defaultValue?: T) =>
//     ipcRendererHolder.send<PreferenceRequests.Load>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.LOAD_SELECTIVE_ARRAY,
//       params: { key, isExtension: false, defaultValue },
//     }),

//   notifyPreferenceChanged: (key: string, value: unknown) =>
//     ipcRendererHolder.send<PreferenceRequests.PreferenceChange>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.PREFERENCE_REFRESH,
//       params: { key, value },
//     }),

//   listenPreferenceChanged: (key: string, isMainWindow: boolean, callback: (key: string, value: unknown) => void) => {
//     ipcRendererHolder
//       .send<PreferenceRequests.ListenKey>(IpcEvents.PREFERENCES, {
//         type: PreferenceEvents.LISTEN_PREFERENCE,
//         params: { key, isMainWindow },
//       })
//       .then((channel) => ipcRendererHolder.on(channel as string, callback))
//   },
//   resetToDefault: () =>
//     ipcRendererHolder.send(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.RESET_TO_DEFAULT,
//       params: undefined,
//     }),
// })

// contextBridge.exposeInMainWorld('Store', {
//   getSecure: (key: string) =>
//     ipcRendererHolder.send<StoreRequests.Get>(IpcEvents.STORE, {
//       type: StoreEvents.GET_SECURE,
//       params: { service: key },
//     }),

//   setSecure: (key: string, value: string) =>
//     ipcRendererHolder.send<StoreRequests.Set>(IpcEvents.STORE, {
//       type: StoreEvents.SET_SECURE,
//       params: { service: key, token: value },
//     }),

//   removeSecure: (key: string) =>
//     ipcRendererHolder.send<StoreRequests.Get>(IpcEvents.STORE, {
//       type: StoreEvents.REMOVE_SECURE,
//       params: { service: key },
//     }),
// })

// contextBridge.exposeInMainWorld('FileUtils', {
//   scan: (forceScan = false) =>
//     ipcRendererHolder.send<ScannerRequests.ScanSongs>(IpcEvents.SCANNER, {
//       type: ScannerEvents.SCAN_MUSIC,
//       params: { forceScan },
//     }),

//   getScanProgress: () =>
//     ipcRendererHolder.send<void>(IpcEvents.SCANNER, { type: ScannerEvents.GET_PROGRESS, params: undefined }),

//   listenScanProgress: (callback: (progress: Progress) => void) =>
//     ipcRendererHolder.on(ScannerEvents.PROGRESS_CHANNEL, callback),

//   scanSinglePlaylist: (playlistPath: string) =>
//     ipcRendererHolder.send<ScannerRequests.ScanSinglePlaylist>(IpcEvents.SCANNER, {
//       type: ScannerEvents.SCAN_SINGLE_PLAYLIST,
//       params: { playlistPath },
//     }),

//   scanSingleSong: (songPath: string) =>
//     ipcRendererHolder.send<ScannerRequests.ScanSingleSong>(IpcEvents.SCANNER, {
//       type: ScannerEvents.SCAN_SINGLE_SONG,
//       params: { songPath },
//     }),

//   saveAudioToFile: (path: string, blob: Buffer) =>
//     ipcRendererHolder.send<SongRequests.SaveBuffer>(IpcEvents.SONG, {
//       type: SongEvents.SAVE_AUDIO_TO_FILE,
//       params: { path: path, blob: blob },
//     }),

//   saveImageToFile: (path: string, blob: Buffer) =>
//     ipcRendererHolder.send<SongRequests.SaveBuffer>(IpcEvents.SONG, {
//       type: SongEvents.SAVE_IMAGE_TO_FILE,
//       params: { path: path, blob: blob },
//     }),

//   isAudioExists: (path: string) =>
//     ipcRendererHolder.send<SongRequests.FileExists>(IpcEvents.SONG, {
//       type: SongEvents.AUDIO_EXISTS,
//       params: { path: path },
//     }),

//   isImageExists: (path: string) =>
//     ipcRendererHolder.send<SongRequests.FileExists>(IpcEvents.SONG, {
//       type: SongEvents.IMAGE_EXISTS,
//       params: { path: path },
//     }),

//   savePlaylistCover: (b64: string) =>
//     ipcRendererHolder.send<PlaylistRequests.SaveCover>(IpcEvents.PLAYLIST, {
//       type: PlaylistEvents.SAVE_COVER,
//       params: { b64: b64 },
//     }),

//   listenInitialFileOpenRequest: (callback: (paths: string[]) => void) =>
//     ipcRendererHolder.on(SongEvents.GOT_FILE_PATH, callback),

//   resetScanTask: () =>
//     ipcRendererHolder.send(IpcEvents.SCANNER, {
//       type: ScannerEvents.RESET_SCAN_TASK,
//       params: undefined,
//     }),

//   getCPUs: () =>
//     ipcRendererHolder.send(IpcEvents.SCANNER, {
//       type: ScannerEvents.GET_RECOMMENDED_CPUS,
//       params: undefined,
//     }),
// })

// contextBridge.exposeInMainWorld('SearchUtils', {
//   searchSongsByOptions: async (options?: SongAPIOptions, fullFetch = false) => {
//     const songs = (await ipcRendererHolder.send<SearchRequests.SongOptions>(IpcEvents.SEARCH, {
//       type: SearchEvents.SEARCH_SONGS_BY_OPTIONS,
//       params: { options },
//     })) as Song[]

//     if (!fullFetch) {
//       return songs.map((val) => ({
//         _id: val._id,
//         album: val.album,
//         artists: val.artists,
//         date: val.date,
//         date_added: val.date_added,
//         duration: val.duration,
//         genre: val.genre,
//         icon: val.icon,
//         path: val.path,
//         playCount: val.playCount,
//         playbackUrl: val.playbackUrl,
//         providerExtension: val.providerExtension,
//         showInLibrary: val.showInLibrary,
//         song_coverPath_high: val.song_coverPath_high,
//         song_coverPath_low: val.song_coverPath_low,
//         title: val.title,
//         type: val.type,
//         track_no: val.track_no,
//         url: val.url,
//       }))
//     }
//     return songs
//   },

//   searchEntityByOptions: <T extends Artists | Album | Genre | Playlist>(options: EntityApiOptions<T>) =>
//     ipcRendererHolder.send<SearchRequests.EntityOptions>(IpcEvents.SEARCH, {
//       type: SearchEvents.SEARCH_ENTITY_BY_OPTIONS,
//       params: { options },
//     }),

//   searchAll: (term: string) =>
//     ipcRendererHolder.send<SearchRequests.Search>(IpcEvents.SEARCH, {
//       type: SearchEvents.SEARCH_ALL,
//       params: { searchTerm: term },
//     }),

//   searchYT: (title: string, artists?: string[], matchTitle = true, scrapeYTMusic = true, scrapeYoutube = false) =>
//     ipcRendererHolder.send<SearchRequests.SearchYT>(IpcEvents.SEARCH, {
//       type: SearchEvents.SEARCH_YT,
//       params: { title, artists, matchTitle, scrapeYTMusic, scrapeYoutube },
//     }),

//   getYTSuggestions: (videoID: string) =>
//     ipcRendererHolder.send<SearchRequests.YTSuggestions>(IpcEvents.SEARCH, {
//       type: SearchEvents.YT_SUGGESTIONS,
//       params: { videoID },
//     }),

//   getYTAudioURL: (videoID: string) =>
//     ipcRendererHolder.send<SearchRequests.YTSuggestions>(IpcEvents.SEARCH, {
//       type: SearchEvents.GET_YT_AUDIO_URL,
//       params: { videoID },
//     }),

//   getYTPlaylist: (id: string) =>
//     ipcRendererHolder.send<SearchRequests.YTPlaylist>(IpcEvents.SEARCH, {
//       type: SearchEvents.GET_YT_PLAYLIST,
//       params: { id },
//     }),

//   // getYTPlaylistContent: (id: string, continuation: ytpl.Continuation) =>
//   //   ipcRendererHolder.send<SearchRequests.YTPlaylistContent>(IpcEvents.SEARCH, {
//   //     type: SearchEvents.GET_YT_PLAYLIST_CONTENT,
//   //     params: { id, nextPageToken: continuation },
//   //   }),

//   scrapeLastFM: (url: string) =>
//     ipcRendererHolder.send<SearchRequests.LastFMSuggestions>(IpcEvents.SEARCH, {
//       type: SearchEvents.SCRAPE_LASTFM,
//       params: { url },
//     }),

//   searchLyrics: (song: Song) =>
//     ipcRendererHolder.send<SearchRequests.LyricsScrape>(IpcEvents.SEARCH, {
//       type: SearchEvents.SCRAPE_LYRICS,
//       params: { song },
//     }),

//   requestInvidious: <T extends InvidiousResponses.InvidiousApiResources, K extends InvidiousResponses.SearchTypes>(
//     resource: T,
//     search: InvidiousResponses.SearchObject<T, K>,
//     authorization: string,
//     invalidateCache: boolean,
//   ) =>
//     ipcRendererHolder.send<SearchRequests.InvidiousRequest>(IpcEvents.SEARCH, {
//       type: SearchEvents.REQUEST_INVIDIOUS,
//       params: { resource, search, authorization, invalidateCache },
//     }),

//   getPlayCount: (...songIds: string[]) =>
//     ipcRendererHolder.send<SearchRequests.PlayCount>(IpcEvents.SEARCH, {
//       type: SearchEvents.GET_PLAY_COUNT,
//       params: { songIds },
//     }),
// })

// contextBridge.exposeInMainWorld('ThemeUtils', {
//   saveTheme: (theme: ThemeDetails) =>
//     ipcRendererHolder.send<PreferenceRequests.Theme>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.SET_THEME,
//       params: { theme },
//     }),

//   removeTheme: (id: string) =>
//     ipcRendererHolder.send<PreferenceRequests.ThemeID>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.REMOVE_THEME,
//       params: { id },
//     }),

//   getTheme: (id?: string) =>
//     ipcRendererHolder.send<PreferenceRequests.ThemeID>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.GET_THEME,
//       params: { id: id ?? 'default' },
//     }),

//   transformCSS: (cssPath: string) =>
//     ipcRendererHolder.send<PreferenceRequests.TransformCSS>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.TRANSFORM_CSS,
//       params: { cssPath },
//     }),

//   packTheme: (id: string) =>
//     ipcRendererHolder.send<PreferenceRequests.ThemeID>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.PACK_THEME,
//       params: { id },
//     }),

//   importTheme: (themeZipPath: string) =>
//     ipcRendererHolder.send<PreferenceRequests.ImportTheme>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.IMPORT_THEME,
//       params: { themeZipPath },
//     }),

//   getAllThemes: () =>
//     ipcRendererHolder.send<undefined>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.GET_ALL_THEMES,
//       params: undefined,
//     }),

//   setActiveTheme: (id: string) =>
//     ipcRendererHolder.send<PreferenceRequests.ThemeID>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.SET_ACTIVE_THEME,
//       params: { id },
//     }),

//   getActiveTheme: () =>
//     ipcRendererHolder.send(IpcEvents.PREFERENCES, { type: PreferenceEvents.GET_ACTIVE_THEME, params: undefined }),

//   setSongView: (menu: songMenu) =>
//     ipcRendererHolder.send<PreferenceRequests.SongView>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.SET_SONG_VIEW,
//       params: { menu },
//     }),

//   getSongView: () =>
//     ipcRendererHolder.send(IpcEvents.PREFERENCES, { type: PreferenceEvents.GET_SONG_VIEW, params: undefined }),

//   setLanguage: (key: string) =>
//     ipcRendererHolder.send<PreferenceRequests.LanguageKey>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.SET_LANGUAGE,
//       params: { key },
//     }),

//   listenGenerateIconRequest: (callback: (params: IpcRequest<PreferenceRequests.GenerateIcon>) => void) =>
//     ipcRendererHolder.on(PreferenceEvents.GENERATE_ICON, callback),

//   replyToGenerateIconRequest: (buffer: string, channel: string) =>
//     ipcRenderer.send(PreferenceEvents.GENERATE_ICON, { channel, buffer }),

//   setTempTheme: (theme: ThemeDetails) =>
//     ipcRendererHolder.send<PreferenceRequests.Theme>(IpcEvents.PREFERENCES, {
//       type: PreferenceEvents.SET_TEMP_THEME,
//       params: {
//         theme,
//       },
//     }),

//   onThemeRefresh: (callback: (theme: ThemeDetails) => void) =>
//     ipcRendererHolder.on(PreferenceEvents.THEME_REFRESH, callback),
// })

// contextBridge.exposeInMainWorld('WindowUtils', {
//   openWindow: (isMainWindow: boolean, args?: unknown) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.OPEN_WIN,
//       params: { isMainWindow, args },
//     }),

//   closeWindow: (isMainWindow: boolean) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.CLOSE_WIN,
//       params: { isMainWindow },
//     }),

//   minWindow: (isMainWindow: boolean) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.MIN_WIN,
//       params: { isMainWindow },
//     }),

//   maxWindow: (isMainWindow: boolean) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.MAX_WIN,
//       params: { isMainWindow },
//     }),

//   toggleFullscreen: (isMainWindow: boolean) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.TOGGLE_FULLSCREEN,
//       params: { isMainWindow },
//     }),

//   enableFullscreen: (isMainWindow: boolean) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.ENABLE_FULLSCREEN,
//       params: { isMainWindow },
//     }),

//   disableFullscreen: (isMainWindow: boolean) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.DISABLE_FULLSCREEN,
//       params: { isMainWindow },
//     }),

//   hasFrame: () =>
//     ipcRendererHolder.send(IpcEvents.BROWSER_WINDOWS, { type: WindowEvents.HAS_FRAME, params: undefined }),

//   showTitlebarIcons: () =>
//     ipcRendererHolder.send(IpcEvents.BROWSER_WINDOWS, { type: WindowEvents.SHOW_TITLEBAR_ICONS, params: undefined }),

//   isWindowMaximized: (isMainWindow: boolean) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.IS_MAXIMIZED,
//       params: { isMainWindow },
//     }),

//   // openFileBrowser: (isMainWindow: boolean, file: boolean, filters?: Electron.FileFilter[]) =>
//   //   ipcRendererHolder.send<WindowRequests.FileBrowser>(IpcEvents.BROWSER_WINDOWS, {
//   //     type: WindowEvents.OPEN_FILE_BROWSER,
//   //     params: { file, filters, isMainWindow },
//   //   }),

//   toggleDevTools: (isMainWindow: boolean) =>
//     ipcRendererHolder.send<WindowRequests.MainWindowCheck>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.TOGGLE_DEV_TOOLS,
//       params: { isMainWindow },
//     }),

//   openExternal: (url: string) =>
//     ipcRendererHolder.send<WindowRequests.URL>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.OPEN_URL_EXTERNAL,
//       params: { url: url },
//     }),

//   registerOAuthCallback: (path: string) =>
//     ipcRendererHolder.send<WindowRequests.Path>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.REGISTER_OAUTH_CALLBACK,
//       params: { path },
//     }),

//   deregisterOAuthCallback: (path: string) =>
//     ipcRendererHolder.send<WindowRequests.Path>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.DEREGISTER_OAUTH_CALLBACK,
//       params: { path },
//     }),

//   triggerOAuthCallback: (path: string) =>
//     ipcRendererHolder.send<WindowRequests.Path>(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.TRIGGER_OAUTH_CALLBACK,
//       params: { path },
//     }),

//   listenOAuth: (channelID: string, callback: (data: URL) => void) => ipcRendererHolder.once(channelID, callback),

//   listenArgs: (callback: (args: unknown) => void) => ipcRendererHolder.once(WindowEvents.GOT_EXTRA_ARGS, callback),

//   mainWindowHasMounted: () =>
//     ipcRendererHolder.send(IpcEvents.BROWSER_WINDOWS, {
//       type: WindowEvents.MAIN_WINDOW_HAS_MOUNTED,
//       params: undefined,
//     }),

//   dragFile: (path: string) =>
//     ipcRendererHolder.send(IpcEvents.BROWSER_WINDOWS, { type: WindowEvents.DRAG_FILE, params: { path } }),

//   restartApp: () =>
//     ipcRendererHolder.send(IpcEvents.BROWSER_WINDOWS, { type: WindowEvents.RESTART_APP, params: undefined }),

//   updateZoom: () =>
//     ipcRendererHolder.send(IpcEvents.BROWSER_WINDOWS, { type: WindowEvents.UPDATE_ZOOM, params: undefined }),

//   getPlatform: () =>
//     ipcRendererHolder.send(IpcEvents.BROWSER_WINDOWS, { type: WindowEvents.GET_PLATFORM, params: undefined }),

//   clearRSS: clearCache,

//   handleReload: () =>
//     ipcRendererHolder.send(IpcEvents.BROWSER_WINDOWS, { type: WindowEvents.HANDLE_RELOAD, params: undefined }),
// })

// contextBridge.exposeInMainWorld('LoggerUtils', {
//   info: (...message: unknown[]) =>
//     ipcRendererHolder.send<LoggerRequests.LogEvents>(IpcEvents.LOGGER, {
//       type: LoggerEvents.INFO,
//       params: { message: message },
//     }),

//   error: (...message: unknown[]) =>
//     ipcRendererHolder.send<LoggerRequests.LogEvents>(IpcEvents.LOGGER, {
//       type: LoggerEvents.ERROR,
//       params: { message: message },
//     }),

//   warn: (...message: unknown[]) =>
//     ipcRendererHolder.send<LoggerRequests.LogEvents>(IpcEvents.LOGGER, {
//       type: LoggerEvents.WARN,
//       params: { message: message },
//     }),

//   debug: (...message: unknown[]) =>
//     ipcRendererHolder.send<LoggerRequests.LogEvents>(IpcEvents.LOGGER, {
//       type: LoggerEvents.DEBUG,
//       params: { message: message },
//     }),

//   trace: (...message: unknown[]) =>
//     ipcRendererHolder.send<LoggerRequests.LogEvents>(IpcEvents.LOGGER, {
//       type: LoggerEvents.TRACE,
//       params: { message: message },
//     }),

//   watchLogs: (callback: (data: unknown) => void) => {
//     ipcRendererHolder.on(LoggerEvents.WATCH_LOGS, callback)
//     return ipcRendererHolder.send<void>(IpcEvents.LOGGER, { type: LoggerEvents.WATCH_LOGS, params: undefined })
//   },

//   unwatchLogs: () => ipcRendererHolder.send(IpcEvents.LOGGER, { type: LoggerEvents.UNWATCH_LOGS, params: undefined }),

//   setLogLevel: (level: LogLevelDesc) =>
//     ipcRendererHolder.send<LoggerRequests.LogLevels>(IpcEvents.LOGGER, {
//       type: LoggerEvents.WATCH_LOGS,
//       params: { level },
//     }),
// })

// contextBridge.exposeInMainWorld('NotifierUtils', {
//   watchFileChanges: (path: string, watch: boolean, mainWindow: boolean) =>
//     ipcRendererHolder.send<NotifierRequests.FileChanges>(IpcEvents.NOTIFIER, {
//       type: NotifierEvents.WATCH_FILE_CHANGES,
//       params: {
//         path,
//         watch,
//         mainWindow,
//       },
//     }),
//   onFileChanged: (callback: (path: string) => void) => ipcRendererHolder.on(NotifierEvents.FILE_CHANGED, callback),
// })

// contextBridge.exposeInMainWorld('ExtensionUtils', {
//   install: (...path: string[]) =>
//     ipcRendererHolder.send<ExtensionHostRequests.Install>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.INSTALL,
//       params: { path: path },
//     }),

//   uninstall: (packageName: string) =>
//     ipcRendererHolder.send<ExtensionHostRequests.RemoveExtension>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.REMOVE_EXT,
//       params: { packageName },
//     }),

//   getAllExtensions: () =>
//     ipcRendererHolder.send(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.GET_ALL_EXTENSIONS,
//       params: undefined,
//     }),

//   getExtensionIcon: (packageName: string) =>
//     ipcRendererHolder.send<ExtensionHostRequests.RemoveExtension>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.GET_EXTENSION_ICON,
//       params: { packageName },
//     }),

//   listenRequests: (callback: (request: extensionUIRequestMessage) => void) =>
//     ipcRendererHolder.on(ExtensionHostEvents.EXTENSION_REQUESTS, callback),
//   replyToRequest: (data: extensionReplyMessage) => ipcRenderer.send(ExtensionHostEvents.EXTENSION_REQUESTS, data),

//   toggleExtStatus: (packageName: string, enabled: boolean) =>
//     ipcRendererHolder.send<ExtensionHostRequests.ToggleExtensionStatus>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.TOGGLE_EXT_STATUS,
//       params: { packageName, enabled },
//     }),

//   sendEvent: <T extends ExtraExtensionEventTypes>(event: ExtraExtensionEvents<T>) =>
//     ipcRendererHolder.send<ExtensionHostRequests.ExtraEvent>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.SEND_EXTRA_EVENT,
//       params: { event },
//     }),

//   downloadExtension: (ext: FetchedExtensionManifest) =>
//     ipcRendererHolder.send<ExtensionHostRequests.DownloadExtension>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.DOWNLOAD_EXTENSION,
//       params: { ext },
//     }),

//   getContextMenuItems: (type: ContextMenuTypes) =>
//     ipcRendererHolder.send<ExtensionHostRequests.ContextMenuItems>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.GET_EXT_CONTEXT_MENU,
//       params: { type },
//     }),

//   fireContextMenuHandler: (id: string, packageName: string, arg: ExtensionContextMenuHandlerArgs<ContextMenuTypes>) =>
//     ipcRendererHolder.send<ExtensionHostRequests.ContextMenuHandler>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.ON_CONTEXT_MENU_ITEM_CLICKED,
//       params: { id, packageName, arg },
//     }),

//   listenExtInstallStatus: (callback: (data: ExtInstallStatus) => void) =>
//     ipcRendererHolder.on(ExtensionHostEvents.EXT_INSTALL_STATUS, callback),

//   getRegisteredAccounts: (packageName: string) =>
//     ipcRendererHolder.send<ExtensionHostRequests.ProviderScopes>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.GET_REGISTERED_ACCOUNTS,
//       params: { packageName },
//     }),

//   listenAccountRegistered: (
//     callback: (details: { packageName: string; data: StrippedAccountDetails }) => void,
//     packageName: string,
//   ) => {
//     ipcRendererHolder.on(ExtensionHostEvents.ON_ACCOUNT_REGISTERED, (details: Parameters<typeof callback>[0]) => {
//       if (packageName) {
//         if (details.packageName === packageName) {
//           callback(details)
//         }
//       } else {
//         callback(details)
//       }
//     })
//   },

//   performAccountLogin: (packageName: string, accountId: string, login: boolean) =>
//     ipcRendererHolder.send<ExtensionHostRequests.AccountLogin>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.PERFORM_ACCOUNT_LOGIN,
//       params: { packageName, accountId, login },
//     }),

//   listenExtensionsChanged: (callback: () => void) =>
//     ipcRendererHolder.on(ExtensionHostEvents.ON_EXTENSIONS_CHANGED, callback),

//   getExtensionProviderScopes: (packageName: string) =>
//     ipcRendererHolder.send<ExtensionHostRequests.ProviderScopes>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.GET_EXTENSION_PROVIDER_SCOPES,
//       params: { packageName },
//     }),

//   getExtensionDisplayName: (packageName: string) =>
//     ipcRendererHolder.send<ExtensionHostRequests.ProviderScopes>(IpcEvents.EXTENSION_HOST, {
//       type: ExtensionHostEvents.GET_DISPLAY_NAME,
//       params: { packageName },
//     }),
// })

// contextBridge.exposeInMainWorld('UpdateUtils', {
//   check: () => ipcRendererHolder.send(IpcEvents.UPDATE, { type: UpdateEvents.CHECK_UPDATES, params: undefined }),

//   listenUpdate: (callback: (hasUpdate: boolean) => void) => ipcRendererHolder.on(UpdateEvents.GOT_UPDATE, callback),

//   updateNow: () => ipcRendererHolder.send(IpcEvents.UPDATE, { type: UpdateEvents.UPDATE_NOW, params: undefined }),
// })

// contextBridge.exposeInMainWorld('MprisUtils', {
//   // updateSongInfo: (data: MprisRequests.SongInfo) =>
//   //   ipcRendererHolder.send<MprisRequests.SongInfo>(IpcEvents.MPRIS, {
//   //     type: MprisEvents.SONG_INFO_CHANGED,
//   //     params: data,
//   //   }),

//   // updatePlaybackState: (state: PlayerState) =>
//   //   ipcRendererHolder.send<MprisRequests.PlaybackState>(IpcEvents.MPRIS, {
//   //     type: MprisEvents.PLAYBACK_STATE_CHANGED,
//   //     params: { state },
//   //   }),

//   // setButtonStatus: (status: PlayerButtons) =>
//   //   ipcRendererHolder.send<MprisRequests.ButtonStatus>(IpcEvents.MPRIS, {
//   //     type: MprisEvents.BUTTON_STATUS_CHANGED,
//   //     params: status,
//   //   }),

//   // listenMediaButtonPress: (callback: (args: typeof ButtonEnum) => void) =>
//   //   ipcRendererHolder.on(MprisEvents.ON_BUTTON_PRESSED, callback),

//   // updatePosition: (position: number) =>
//   //   ipcRendererHolder.send<MprisRequests.Position>(IpcEvents.MPRIS, {
//   //     type: MprisEvents.POSITION_CHANGED,
//   //     params: { position },
//   //   }),
// })

// contextBridge.exposeInMainWorld('SpotifyPlayer', {
//   // connect: (config: ConstructorConfig) =>
//   //   ipcRendererHolder.send<SpotifyRequests.Config>(IpcEvents.SPOTIFY, {
//   //     type: SpotifyEvents.CONNECT,
//   //     params: config,
//   //   }),

//   // on: <T extends PlayerEventTypes>(event: T, listener: (event: PlayerEvent<T>) => void) => {
//   //   const responseChannel = window.crypto?.randomUUID() ?? Date.now().toString()
//   //   ipcRendererHolder.send<SpotifyRequests.EventListener>(IpcEvents.SPOTIFY, {
//   //     type: SpotifyEvents.LISTEN_EVENT,
//   //     params: { event },
//   //     responseChannel,
//   //   })

//   //   ipcRendererHolder.on(responseChannel, listener)
//   //   return responseChannel
//   // },

//   // off: <T extends PlayerEventTypes>(responseChannel: string, event: T, listener: (event: PlayerEvent<T>) => void) => {
//   //   ipcRendererHolder.send<SpotifyRequests.EventListener>(IpcEvents.SPOTIFY, {
//   //     type: SpotifyEvents.REMOVE_EVENT,
//   //     params: { event },
//   //   })

//   //   ipcRendererHolder.off(responseChannel, listener)
//   // },

//   // command: <T extends SpotifyRequests.SpotifyCommands>(command: T, args: never[]) =>
//   //   ipcRendererHolder.send<SpotifyRequests.Command<T>>(IpcEvents.SPOTIFY, {
//   //     type: SpotifyEvents.COMMAND,
//   //     params: {
//   //       command,
//   //       args,
//   //     },
//   //   }),

//   // close: () =>
//   //   ipcRendererHolder.send(IpcEvents.SPOTIFY, {
//   //     type: SpotifyEvents.CLOSE,
//   //     params: undefined,
//   //   }),

//   // getToken: (scopes: TokenScope[]) =>
//   //   ipcRendererHolder.send<SpotifyRequests.Token>(IpcEvents.SPOTIFY, {
//   //     type: SpotifyEvents.GET_TOKEN,
//   //     params: {
//   //       scopes,
//   //     },
//   //   }),
// })

// contextBridge.exposeInMainWorld('RodioUtils', {
//   initialize: () => ipcRendererHolder.send(IpcEvents.RODIO, { type: RodioEvents.INITIALIZE, params: undefined }),
//   setSrc: (path: string) =>
//     ipcRendererHolder.send<RodioRequests.SetSrc>(IpcEvents.RODIO, { type: RodioEvents.SET_SRC, params: { path } }),
//   play: () => ipcRendererHolder.send(IpcEvents.RODIO, { type: RodioEvents.PLAY, params: undefined }),
//   pause: () => ipcRendererHolder.send(IpcEvents.RODIO, { type: RodioEvents.PAUSE, params: undefined }),
//   stop: () => ipcRendererHolder.send(IpcEvents.RODIO, { type: RodioEvents.STOP, params: undefined }),
//   seek: (pos: number) =>
//     ipcRendererHolder.send<RodioRequests.Seek>(IpcEvents.RODIO, { type: RodioEvents.SEEK, params: { pos } }),
//   setVolume: (volume: number) => {
//     ipcRendererHolder.send<RodioRequests.Volume>(IpcEvents.RODIO, { type: RodioEvents.SET_VOLUME, params: { volume } })
//   },
//   // getCurrentTime: () => ipcRendererHolder.send(IpcEvents.RODIO, { type: RodioEvents.GET_POSITION, params: undefined }),
//   // getVolume: () => ipcRendererHolder.send(IpcEvents.RODIO, { type: RodioEvents.GET_VOLUME, params: undefined }),
//   listenEvents: (callback: (event: RodioEvents) => void) => ipcRendererHolder.on(IpcEvents.RODIO, callback),
// })

// function clearCache() {
//   webFrame.clearCache()
// }
