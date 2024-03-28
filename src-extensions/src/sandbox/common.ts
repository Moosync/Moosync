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

	if ((field as string) === "artist") {
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
			parseInt(matches[8] ?? "0") +
			parseInt(matches[7] ?? "0") * 60 +
			parseInt(matches[6] ?? "0") * 60 * 60 +
			parseInt(matches[5] ?? "0") * 60 * 60 * 24
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

interface ExtendedPlaylist extends Playlist {
	extension?: string;
	isLocal: boolean;
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
