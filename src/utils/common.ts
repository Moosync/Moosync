import { invoke } from "@tauri-apps/api/core";
import { listen, emit } from "@tauri-apps/api/event";

window.PreferenceUtils = {
	saveSelective: (key, value) => {
		console.log("saving", key, value);
		return invoke("save_selective", { key, value: value ?? null });
	},

	loadSelective: async (key: string, _, default_val) => {
		return (await invoke("load_selective", { key })) ?? default_val;
	},

	loadSelectiveArrayItem: async (key, default_val) => {
		return (await invoke("load_selective_array", { key })) ?? default_val;
	},

	listenPreferenceChanged: async () => {
		return;
	},
	notifyPreferenceChanged: async () => {
		return;
	},
};

window.LoggerUtils = {
	debug: (...message: unknown[]) => {
		return invoke("log_debug", { message });
	},
	error: (...message: unknown[]) => {
		return invoke("log_error", { message });
	},
	info: (...message: unknown[]) => {
		return invoke("log_info", { message });
	},
	warn: (...message: unknown[]) => {
		return invoke("log_warn", { message });
	},
	trace: (...message: unknown[]) => {
		return invoke("log_debug", { message });
	},
};

window.FileUtils = {
	scan: (force) => {
		return invoke("start_scan", { force, paths: null });
	},
	scanSingleSong: (path: string) => {
		return invoke("start_scan", { paths: [path], force: false });
	},
	listenInitialFileOpenRequest: async (callback) => {},
};

const defaultTheme: ThemeDetails = {
	id: "default",
	name: "Default",
	author: "Moosync",
	theme: {
		primary: "#212121",
		secondary: "#282828",
		tertiary: "#151515",
		textPrimary: "#ffffff",
		textSecondary: "#565656",
		textInverse: "#000000",
		accent: "#65CB88",
		divider: "rgba(79, 79, 79, 0.67)",
	},
};

window.ThemeUtils = {
	getAllThemes: async () => {
		return await invoke("load_all_themes");
	},
	getTheme: async (id) => {
		if (!id) return defaultTheme;

		return invoke("load_theme", { id });
	},
	saveTheme: async (theme) => {
		console.log("saving", theme);
		return invoke("save_theme", { theme });
	},

	removeTheme: async (id) => {
		return invoke("remove_theme", { id });
	},
	getActiveTheme: async () => {
		console.log("getting active theme");

		const id = await window.PreferenceUtils.loadSelective(
			"activeTheme",
			false,
			defaultTheme,
		);
		if (!id) return defaultTheme;
		return invoke("load_theme", { id });
	},

	setActiveTheme: async (id) => {
		await window.PreferenceUtils.saveSelective("activeTheme", id);
		const theme = await window.ThemeUtils.getTheme(id);
		emit("theme_refresh", theme);
	},
	getSongView: async () => {
		return (
			(await window.PreferenceUtils.loadSelective(
				"songView",
				false,
				"compact",
			)) ?? "compact"
		);
	},
	setSongView: async (value) => {
		return window.PreferenceUtils.saveSelective("songView", value);
	},
	onThemeRefresh: async (callback) => {
		listen("theme_refresh", async (event) => {
			callback(event.payload as ThemeDetails);
		});
	},
	setTempTheme: async (theme: ThemeDetails) => {
		emit("theme_refresh", theme);
	},
	importTheme: async (themePath) => invoke("import_theme", { themePath }),
	transformCSS: async (cssPath) => invoke("transform_css", { cssPath }),
};

window.MprisUtils = {
	updateSongInfo: async (metadata) => {
		invoke("set_metadata", { metadata });
	},
	updatePlaybackState: async (state) => invoke("set_playback_state", { state }),
	setButtonStatus: async () => {},
	listenMediaButtonPress: async (callback) => {
		listen("media_button_press", async (event) => {
			const payload = event.payload as [number, unknown];
			callback(payload[0], payload[1]);
		});
	},
	updatePosition: async (duration) => invoke("set_position", { duration }),
};

window.UpdateUtils = new Proxy(
	{},
	{
		get(target, prop) {
			console.log(target, prop);
			return () => {};
		},
	},
) as any;

window.RodioUtils = new Proxy(
	{},
	{
		get(target, prop) {
			console.log(target, prop);
			return () => {};
		},
	},
) as any;

window.WindowUtils = {
	registerOAuthCallback: async (path) => {
		return await invoke("register_oauth_path", { path });
	},
	deregisterOAuthCallback: async (path) => {
		return await invoke("unregister_oauth_path", { path });
	},
	listenOAuth: (channelID, callback) => {
		listen(
			channelID,
			(event) => {
				console.log("got event", event);
				const data = event.payload as string;
				callback(data);
			},
			{},
		);
	},

	isWindowMaximized: async () => {
		return await invoke("is_maximized");
	},

	hasFrame: async () => {
		return await invoke("has_frame");
	},
	closeWindow: async () => {
		return await invoke("close_window");
	},
	getPlatform: async () => {
		return await invoke("get_platform");
	},
	maxWindow: async () => {
		return await invoke("maximize_window");
	},
	minWindow: async () => {
		return await invoke("minimize_window");
	},
	clearRSS: async () => {
		// For compatibility
	},
	updateZoom: async () => {
		return await invoke("update_zoom");
	},
	openExternal: async (url) => {
		return await invoke("open_external", { url });
	},
	openWindow: async (isMainWindow, args) => {
		return await invoke("open_window", { isMainWindow });
	},
	listenArgs: (callback) => {
		callback({
			page: "paths",
		});
	},
	showTitlebarIcons: async () => {
		return false;
	},
	mainWindowHasMounted: async () => {},
	handleReload: async () => {},
	enableFullscreen: async () => invoke("enable_fullscreen"),
	disableFullscreen: async () => invoke("disable_fullscreen"),
	toggleFullscreen: async () => invoke("toggle_fullscreen"),
	toggleDevTools: async () => invoke("toggle_dev_tools"),
	openFileBrowser: async (_, file, filters) => {
		return await invoke("open_file_browser", {
			filters: filters ?? [],
			directory: !file,
			multiple: true,
		});
	},
	restartApp: async () => invoke("restart_app"),
	triggerOAuthCallback: async (url) =>
		invoke("trigger_oauth_callback", { url }),
};

window.DBUtils = {
	storeSongs: async (songs) => {
		console.log(songs);
		return await invoke("insert_songs", { songs });
	},
	removeSongs: async (...songs) =>
		await invoke("remove_songs", { songs: songs.map((val) => val._id) }),
	createPlaylist: async (playlist) =>
		await invoke("create_playlist", { playlist }),
	addToPlaylist: async (playlistID, ...songs) =>
		await invoke("add_to_playlist", { id: playlistID, songs }),
	removeFromPlaylist: async (playlistID, ...songs) =>
		await invoke("remove_from_playlist", {
			id: playlistID,
			songs: songs.map((val) => val._id),
		}),
	removePlaylist: async (id) => await invoke("remove_playlist", { id }),
	updateAlbum: async (album) => await invoke("update_album", { album }),
	updateArtist: async (artist) => await invoke("update_artist", { artist }),
	updatePlaylist: async (playlist) =>
		await invoke("update_playlist", { playlist }),
	updateSongs: async (songs) => await invoke("update_songs", { songs }),
	updateLyrics: async (id, lyrics) => await invoke("update_lyrics", { lyrics }),
	incrementPlayCount: async (id) =>
		await invoke("increment_play_count", { id }),
	incrementPlayTime: async (id, duration) =>
		await invoke("increment_play_time", { id, duration }),
	exportPlaylist: async (playlist) => {},
};

window.SearchUtils = {
	searchEntityByOptions: (options) => {
		return invoke("get_entity_by_options", { options });
	},
	searchSongsByOptions: async (options, _) => {
		return invoke("get_songs_by_options", { options });
	},
	searchYT: async (title, artists) => {
		return await invoke("search_yt", { title, artists: artists ?? [] });
	},
	getYTAudioURL: async (id) => {
		console.log("getting url for", id);
		return await invoke("get_video_url", { id });
	},
	searchAll: async (term) => {
		return await invoke("search_all", { term });
	},
	searchLyrics: async (song: Song) =>
		invoke("get_lyrics", {
			artists: song.artists?.map((val) => val.artist_name) ?? [],
			title: song.title,
			id: song._id,
			url: song.url,
		}),
};

window.ExtensionUtils = {
	getAllExtensions: async () => {
		return [];
	},
	listenExtensionsChanged: () => {},
	listenAccountRegistered: () => {},
	listenRequests: () => {},
	sendEvent: (event) => {
		return undefined as any;
	},
	getContextMenuItems: async () => {
		return [];
	},
};

window.Store = {
	setSecure: async (key, value) => {
		return await invoke("set_secure", { key, value });
	},
	getSecure: async (key) => {
		return await invoke("get_secure", { key });
	},
	removeSecure: async (key) => {
		return invoke("save_selective", { key, value: null });
	},
};

class SpotifyPosition {
	private interval: ReturnType<typeof setInterval> | undefined;
	private callback: ((pos: number) => void) | undefined;
	private position = 0;
	private position_update = 500;

	setCallback(callback?: (pos: number) => void) {
		this.callback = callback;
	}

	start() {
		if (this.interval) {
			clearInterval(this.interval);
			this.interval = undefined;
		}

		this.interval = setInterval(() => {
			this.position = this.position + this.position_update;
			this.callback?.(this.position);
		}, this.position_update);
	}

	stop() {
		if (this.interval) {
			clearInterval(this.interval);
			this.interval = undefined;
		}
	}

	updatePos(pos: unknown) {
		this.position = Number.parseInt((pos as string).toString());
		this.callback?.(this.position);
	}
}
const spotifyPositionHandler = new SpotifyPosition();

window.SpotifyPlayer = {
	connect: async (config) => {
		const id = Math.random()
			.toString(36)
			.substring(2, 8 + 2);
		await invoke("initialize_librespot", { config, id });
		return id;
	},
	close: async () => {
		return invoke("librespot_close");
	},
	command: async (command, args) => {
		switch (command) {
			case "PLAY":
				return invoke("librespot_play");
			case "PAUSE":
				return invoke("librespot_pause");
			case "SEEK":
				return invoke("librespot_seek", { pos: args[0] });
			case "LOAD":
				return invoke("librespot_load", { uri: args[0], autoplay: false });
			case "VOLUME":
				return invoke("librespot_volume", {
					volume: Number.parseInt(
						((Math.max(Math.min(args[0], 100), 0) / 100) * 65535).toFixed(0),
					),
				});
			case "GET_CANVAS":
				return invoke("get_canvaz", { uri: args[0] });
		}
	},
	getToken: async (scope) => {
		return invoke("librespot_get_token", { scopes: scope.join(",") });
	},
	on: async <T extends PlayerEventTypes>(
		event: T,
		id: string,
		callback: (event: PlayerEvent<T>) => void,
	) => {
		if (event === "TimeUpdated") {
			spotifyPositionHandler.setCallback((pos) => {
				callback({
					event,
					position_ms: pos,
				} as unknown as PlayerEvent<T>);
			});
			return () => spotifyPositionHandler.setCallback(undefined);
		}

		await invoke("register_event", { event: `librespot_event_${event}_${id}` });

		const unlisten = await listen(`librespot_event_${event}_${id}`, (event) => {
			const data = event.payload as PlayerEvent<T>;
			switch (data.event) {
				case "Playing":
					spotifyPositionHandler.updatePos(
						(data as PlayerEvent<"Playing">).position_ms,
					);
					spotifyPositionHandler.start();
					break;
				case "Paused":
					spotifyPositionHandler.updatePos(
						(data as PlayerEvent<"Paused">).position_ms,
					);
					spotifyPositionHandler.stop();
					break;
				case "Stopped":
					spotifyPositionHandler.updatePos(0);
					spotifyPositionHandler.stop();
					break;
				case "PositionCorrection":
					spotifyPositionHandler.updatePos(
						(data as PlayerEvent<"PositionCorrection">).position_ms,
					);
					break;
				case "Seeked":
					spotifyPositionHandler.updatePos(
						(data as PlayerEvent<"Seeked">).position_ms,
					);
					break;
				case "TrackChanged":
					spotifyPositionHandler.updatePos(0);
					spotifyPositionHandler.stop();
					break;
			}
			callback(data);
		});
		return unlisten;
	},
};

console.log("info logger", window.PreferenceUtils);

/*
 *  common.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import { v4 } from "uuid";

export function arrayDiff<T>(arr1: T[], arr2: T[]) {
	return arr1.filter((x) => !arr2.includes(x));
}

export function convertDuration(n: number) {
	if (n) {
		if (!Number.isFinite(n) || n < 0) {
			return "Live";
		}
		const tmp = new Date(n * 1000).toISOString().substring(11, 19);

		if (tmp[0] === "0" && tmp[1] === "0") {
			return tmp.substring(3);
		}

		return tmp;
	}
	return "00:00";
}

export function getVersion(verS: string) {
	try {
		return parseInt(verS.split(".").join(""));
	} catch (e) {
		console.warn(
			"Failed to parse",
			verS,
			". Please use x.y.z as extension versioning format",
			e,
		);
	}

	return 0;
}

export function isEmpty<T>(val: T | undefined): val is undefined {
	return typeof val === "undefined" || val === null;
}

function sortAsc(first: unknown, second: unknown) {
	if (typeof first === "string" && typeof second === "string")
		return first.localeCompare(second);

	if (typeof first === "number" && typeof second === "number")
		return first - second;

	return 0;
}

function getSortField(song: Song, field: SongSortOptions["type"]) {
	if (field === "album") {
		return song.album?.album_name;
	}

	if (field === "artist") {
		return song.artists?.[0].artist_name;
	}

	if (field === "genre") {
		return song.genre?.[0];
	}

	return song[field as keyof Song];
}

export function sortSongListFn(options: SongSortOptions[]) {
	const fn = (a: Song, b: Song) => {
		for (const o of options) {
			const first: unknown = getSortField(a, o.type);
			const second: unknown = getSortField(b, o.type);

			if (!isEmpty(first) && !isEmpty(second)) {
				if (first !== second) {
					if (!o.asc) {
						return sortAsc(second, first);
					}
					return sortAsc(first, second);
				}
			}
		}

		return 0;
	};

	return fn;
}

export function sortSongList(
	songList: Song[],
	options: SongSortOptions[],
): Song[] {
	let parsedOptions = options;
	if (!Array.isArray(options)) parsedOptions = [options];
	return songList.sort(sortSongListFn(parsedOptions));
}

const iso8601DurationRegex =
	/(-)?P(?:([.,\d]+)Y)?(?:([.,\d]+)M)?(?:([.,\d]+)W)?(?:([.,\d]+)D)?T(?:([.,\d]+)H)?(?:([.,\d]+)M)?(?:([.,\d]+)S)?/;

export function parseISO8601Duration(duration: string): number {
	const matches = duration.match(iso8601DurationRegex);

	// Don't care about anything over days
	if (matches) {
		return (
			parseInt(matches[8] ?? 0) +
			parseInt(matches[7] ?? 0) * 60 +
			parseInt(matches[6] ?? 0) * 60 * 60 +
			parseInt(matches[5] ?? 0) * 60 * 60 * 24
		);
	}
	return 0;
}

export function humanByteSize(size: number, bitrate = false): string {
	const thresh = bitrate ? 1000 : 1024;
	const dp = 2;

	if (Math.abs(size) < thresh) {
		return `${size} B`;
	}

	const units = bitrate
		? ["kbps", "Mbps", "Gbps", "Tbps", "Pbps", "Ebps", "Zbps", "Ybps"]
		: ["KiB", "MiB", "GiB", "TiB", "PiB", "EiB", "ZiB", "YiB"];
	let u = -1;
	const r = 10 ** dp;

	let parsedSize = size;
	do {
		parsedSize /= thresh;
		++u;
	} while (
		Math.round(Math.abs(parsedSize) * r) / r >= thresh &&
		u < units.length - 1
	);

	return `${parsedSize.toFixed(dp)} ${units[u]}`;
}

export function toRemoteSong(
	song: Song | null | undefined,
	connectionID: string,
): RemoteSong | undefined {
	if (song) {
		if ((song as RemoteSong).senderSocket) {
			return song as RemoteSong;
		}

		return {
			...song,
			senderSocket: connectionID,
		};
	}
}

export function stripSong(song?: RemoteSong): RemoteSong {
	const tmp: RemoteSong = JSON.parse(JSON.stringify(song));
	tmp.path = undefined;
	tmp.lyrics = undefined;

	if (tmp.album) {
		// If the image is hosted somewhere then surely the client on the other end can load it... right?
		if (!tmp.album?.album_coverPath_low?.startsWith("http"))
			tmp.album.album_coverPath_low = undefined;

		if (!tmp.album?.album_coverPath_high?.startsWith("http"))
			tmp.album.album_coverPath_high = undefined;
	}

	if (!tmp.song_coverPath_low?.startsWith("http"))
		tmp.song_coverPath_low = undefined;

	if (!tmp.song_coverPath_high?.startsWith("http"))
		tmp.song_coverPath_high = undefined;

	return tmp;
}

export function getErrorMessage(...args: unknown[]) {
	const ret = [];
	for (const data of args) {
		if (data instanceof Error) {
			ret.push(data.name);
			ret.push(data.message);
			ret.push(data.stack);
		} else {
			ret.push(data);
		}
	}

	return ret;
}

export function sanitizeArtistName(name: string) {
	return name
		.trim()
		.replace(/([a-z])([A-Z])/g, "$1 $2")
		.toLowerCase()
		.replaceAll("vevo", "");
}

export function dotIndex(
	obj: Record<string, unknown>,
	is: string | string[],
	value?: unknown,
): unknown {
	if (typeof is === "string") return dotIndex(obj, is.split("."), value);
	else if (is.length === 1 && value !== undefined) {
		obj[is[0]] === value;
		return obj;
	} else if (is.length === 0) return obj;
	else
		return dotIndex(obj[is[0]] as Record<string, unknown>, is.slice(1), value);
}

function isObject(item: unknown): item is Record<string, unknown> {
	return !!(item && typeof item === "object" && !Array.isArray(item));
}

// eslint-disable-next-line @typescript-eslint/no-explicit-any
export function mergeDeep(
	target: Record<string, unknown>,
	...sources: unknown[]
): Record<string, unknown> {
	if (!sources.length) return target;
	const source = sources.shift();

	if (isObject(target) && isObject(source)) {
		for (const key in source) {
			if (isObject(source[key])) {
				if (!target[key]) Object.assign(target, { [key]: {} });
				mergeDeep(target[key] as Record<string, unknown>, source[key]);
			} else {
				Object.assign(target, { [key]: source[key] });
			}
		}
	}

	return mergeDeep(target, ...sources);
}

// https://stackoverflow.com/questions/3446170/escape-string-for-use-in-javascript-regex
export function escapeRegExp(str: string) {
	return str.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

// https://stackoverflow.com/a/19270021
export function getRandomFromArray(arr: unknown[], n: number) {
	const result = new Array(n);
	let len = arr.length;
	const taken = new Array(len);

	let m = Math.min(n, len - 1);
	while (m--) {
		const x = Math.floor(Math.random() * len);
		result[m] = arr[x in taken ? taken[x] : x];
		taken[x] = --len in taken ? taken[len] : len;
	}
	return result;
}

export function sanitizeSong(ext: string, ...songs: Song[]): Song[] {
	return songs.map((val) => ({
		...val,
		artists: sanitizeArtists(ext, ...(val.artists ?? [])),
		album: val.album && sanitizeAlbums(ext, val.album)[0],
		_id: `${ext}:${val._id ?? v4()}`,
		providerExtension: ext,
	}));
}

export function sanitizePlaylist(
	ext: string,
	isLocal: boolean,
	...playlists: Playlist[]
): ExtendedPlaylist[] {
	return playlists.map((val) => ({
		...val,
		playlist_id: `${ext}:${val.playlist_id ?? v4()}`,
		extension: ext,
		isLocal,
	}));
}

export function sanitizeAlbums(ext: string, ...albums: Album[]): Album[] {
	return albums.map((val) => ({
		...val,
		album_id: `${ext}:${val.album_id ?? v4()}`,
		album_extra_info: {
			extensions: {
				[ext]: sanitizeExtraInfo(val.album_extra_info?.extensions?.[ext]),
			},
		},
	}));
}

export function sanitizeExtraInfo(extra_info?: Record<string, unknown>) {
	const ret: Record<string, string | undefined> = {};
	if (extra_info) {
		for (const [key, val] of Object.entries(extra_info)) {
			if (typeof val !== "string") {
				ret[key] = JSON.stringify(val);
			} else {
				ret[key] = val as string;
			}
		}
	}

	return ret;
}

export function sanitizeArtists(ext: string, ...artists: Artists[]): Artists[] {
	return artists.map((val) => ({
		...val,
		artist_id: `${ext}:${val.artist_id ?? v4()}`,
		artist_extra_info: {
			extensions: {
				[ext]: sanitizeExtraInfo(val.artist_extra_info?.extensions?.[ext]),
			},
		},
	}));
}

export const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

export function isAlbum(data: unknown): data is Album {
	return !!(data as Album).album_id;
}

export function isArtist(data: unknown): data is Artists {
	return !!(data as Artists).artist_id;
}

export function isGenre(data: unknown): data is Genre {
	return !!(data as Genre).genre_id;
}

export async function* emptyGen() {
	yield { songs: [] };
}

export function isThemeDetails(data: unknown): data is ThemeDetails {
	const tmpData = data as ThemeDetails;
	return !!(tmpData.id && tmpData.theme);
}
