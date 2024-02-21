/*
 *  ipc.d.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */
interface IpcRequest<T = unknown> {
	type: string;
	responseChannel: string;
	params: T;
}

interface IpcRequestOptionalChannel<T = unknown> {
	type: string;
	responseChannel?: string;
	params: T;
}

interface IpcChannelInterface {
	name: string;
	handle(event: IpcMainEvent, request: IpcRequest): void;
}

declare namespace ExtensionHostRequests {
	interface EventTrigger {
		data: extensionEventMessage;
	}

	interface Install {
		path: string[];
	}

	interface GetAllExtensions {
		packageName: string;
		enabled: boolean;
	}

	interface ToggleExtensionStatus {
		packageName: string;
		enabled: boolean;
	}

	interface RemoveExtension {
		packageName: string;
	}

	interface ExtraEvent {
		event: ExtraExtensionEvents;
	}

	interface DownloadExtension {
		ext: FetchedExtensionManifest;
	}

	interface ContextMenuItems {
		type: ContextMenuTypes;
	}

	interface ContextMenuHandler {
		id: string;
		packageName: string;
		arg: ExtensionContextMenuHandlerArgs;
	}

	interface AccountLogin {
		packageName: string;
		accountId: string;
		login: boolean;
	}

	interface ProviderScopes {
		packageName: string;
	}
}

declare namespace LoggerRequests {
	interface LogEvents {
		message: unknown[];
	}

	interface LogLevels {
		level: import("loglevel").LogLevelDesc;
	}
}

declare namespace SpotifyRequests {
	type Config = import("librespot-node").ConstructorConfig;
	type EventListener = {
		event: string;
	};

	interface Token {
		scopes: import("librespot-node").TokenScope[];
	}

	type SpotifyCommands =
		| "PLAY"
		| "PAUSE"
		| "SEEK"
		| "VOLUME"
		| "LOAD"
		| "ADD_TO_QUEUE"
		| "GET_CANVAS"
		| "GET_LYRICS";

	type ReturnType<T extends SpotifyCommands> = T extends "GET_CANVAS"
		? import("librespot-node").CanvazResponse
		: T extends "GET_LYRICS"
		  ? import("librespot-node").LyricsResponse
		  : undefined;

	interface Command<T extends SpotifyCommands> {
		command: T;
		args: unknown[];
	}
}

declare namespace PlaylistRequests {
	interface AddToPlaylist {
		playlist_id: string;
		song_ids: Song[];
	}

	interface CreatePlaylist {
		playlist: Partial<Playlist>;
	}

	interface SaveCover {
		b64: string;
	}

	interface RemoveExportPlaylist {
		playlist: Playlist;
	}
}

declare namespace ScannerRequests {
	interface ScanSongs {
		forceScan: boolean;
	}

	interface ScanSinglePlaylist {
		playlistPath: string;
	}

	interface ScanSingleSong {
		songPath: string;
	}
}

declare namespace PreferenceRequests {
	interface Save {
		key: string;
		value: unknown;
		isExtension?: boolean;
	}

	interface Load {
		key: string;
		isExtension?: boolean;
		defaultValue: unknown;
	}

	interface PreferenceChange {
		key: string;
		value: unknown;
	}

	interface ThemeID {
		id: string;
	}

	interface ImportTheme {
		themeZipPath: string;
	}

	interface LanguageKey {
		key: string;
	}

	interface Theme {
		theme: ThemeDetails;
	}

	interface TransformCSS {
		cssPath: string;
	}

	interface SongView {
		menu: songMenu;
	}

	interface ListenKey {
		key: string;
		isMainWindow: boolean;
	}

	interface GenerateIcon {
		colors: ThemeDetails;
		size: number;
	}
}

declare namespace SearchRequests {
	interface Search {
		searchTerm: string;
	}

	interface SearchYT {
		title: string;
		artists?: string[];
		matchTitle?: boolean;
		scrapeYTMusic?: boolean;
		scrapeYoutube?: boolean;
	}

	interface YTSuggestions {
		videoID: string;
	}

	interface YTPlaylist {
		id: string;
	}

	interface YTPlaylistContent {
		id: string;
		nextPageToken?: import("ytpl").Continuation;
	}

	interface LastFMSuggestions {
		url: string;
	}

	interface SongOptions {
		options?: SongAPIOptions;
	}

	interface EntityOptions {
		options: EntityApiOptions;
	}

	interface LyricsScrape {
		song: Song;
	}

	interface InvidiousRequest {
		resource: InvidiousResponses.ApiResources;
		search: InvidiousResponses.SearchObject;
		authorization: string | undefined;
		invalidateCache: boolean;
	}

	interface PlayCount {
		songIds: string[];
	}
}

declare namespace SongRequests {
	interface Songs {
		songs: Song[];
	}

	interface UpdateArtist {
		artist: Artist;
	}

	interface UpdateAlbum {
		album: Album;
	}

	interface SaveBuffer {
		path: string;
		blob: NodeJS.ArrayBufferView;
	}

	interface FileExists {
		path: string;
	}

	interface Lyrics {
		id: string;
		lyrics: string;
	}

	interface PlayCount {
		song_id: string;
	}

	interface PlayTime {
		song_id: string;
		duration: number;
	}
}

declare namespace StoreRequests {
	interface Set {
		token: string;
		service: string;
	}

	interface Get {
		service: string;
	}
}

declare namespace WindowRequests {
	interface MainWindowCheck {
		isMainWindow: boolean;
		args?: unknown;
	}

	interface FileBrowser extends MainWindowCheck {
		file: boolean;
		filters?: Electron.FileFilter[];
	}

	interface URL {
		url: string;
	}

	interface Path {
		path: string;
	}
}

declare namespace MprisRequests {
	type PlayerDetails = {
		id?: string;
		title?: string;
		artistName?: string;
		albumName?: string;
		albumArtist?: string;
		thumbnail?: string;
		genres?: string[];
		// In seconds
		duration?: number;
	};

	type PlayerButtons = {
		play?: boolean;
		pause?: boolean;
		next?: boolean;
		prev?: boolean;
		seek?: boolean;
		shuffle?: boolean;
		loop?: "None" | "Track" | "Playlist";
	};

	interface PlaybackState {
		state: PlayerState;
	}

	type SongInfo = PlayerDetails;
	type ButtonStatus = PlayerButtons;

	type Position = {
		position: number;
	};

	interface ShuffleRepeat {
		shuffle: boolean;
		repeat: "Playlist" | "Track" | "None";
	}
}

declare namespace NotifierRequests {
	interface FileChanges {
		path: string;
		watch: boolean;
		mainWindow: boolean | "both";
	}
}

declare namespace RodioRequests {
	interface SetSrc {
		path: string;
	}

	interface Volume {
		volume: number;
	}

	interface Seek {
		pos: number;
	}
}
