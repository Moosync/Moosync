/*
 *  api.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import * as crypto from "node:crypto";
import type { extensionRequests } from "../constants";
import type EventEmitter from "node:events";

export class ExtensionRequestGenerator implements ExtendedExtensionAPI {
	private packageName: string;
	player: PlayerControls;
	utils: Utils;

	#communicator: ExtensionCommunicator;
	#bus: EventEmitter;

	private eventCallbackMap: { [key: string]: unknown } = {};

	private contextMenuMap: ExtendedExtensionContextMenuItems<ContextMenuTypes>[] =
		[];

	private accountsMap: AccountDetails[] = [];

	constructor(
		packageName: string,
		communicator: ExtensionCommunicator,
		bus: EventEmitter,
	) {
		this.packageName = packageName;
		this.player = new PlayerControls(this.packageName, bus);
		this.utils = new Utils(packageName);
		this.#communicator = communicator;
		this.#bus = bus;
	}

	public async getSongs(options: SongAPIOptions) {
		return sendAsync<Song[]>(this.#bus, this.packageName, "get-songs", options);
	}

	public async getEntity<T extends Artists | Album | Genre | Playlist>(
		options: EntityApiOptions<T>,
	) {
		return sendAsync<T[]>(this.#bus, this.packageName, "get-entity", options);
	}

	public async getCurrentSong() {
		return sendAsync<Song>(this.#bus, this.packageName, "get-current-song");
	}

	public async getVolume() {
		return sendAsync<number>(this.#bus, this.packageName, "get-volume");
	}

	public async getTime() {
		return sendAsync<number>(this.#bus, this.packageName, "get-time");
	}

	public async getQueue() {
		return sendAsync<SongQueue>(this.#bus, this.packageName, "get-queue");
	}

	public async getPlayerState() {
		return sendAsync<PlayerState>(
			this.#bus,
			this.packageName,
			"get-player-state",
		);
	}

	public async getPreferences<T>(
		key?: string,
		defaultValue?: unknown,
	): Promise<T | undefined> {
		return sendAsync<T>(this.#bus, this.packageName, "get-preferences", {
			packageName: this.packageName,
			key,
			defaultValue,
		});
	}

	public async getSecure<T>(
		key?: string,
		defaultValue?: unknown,
	): Promise<T | undefined> {
		return sendAsync<T>(this.#bus, this.packageName, "get-secure-preferences", {
			packageName: this.packageName,
			key,
			defaultValue,
		});
	}

	public async setPreferences(key: string, value: unknown): Promise<void> {
		return sendAsync<void>(this.#bus, this.packageName, "set-preferences", {
			packageName: this.packageName,
			key,
			value,
		});
	}

	public async setSecure(key: string, value: unknown): Promise<void> {
		return sendAsync<void>(
			this.#bus,
			this.packageName,
			"set-secure-preferences",
			{
				packageName: this.packageName,
				key,
				value,
			},
		);
	}

	public async addSongs(...songs: Song[]) {
		return sendAsync<(Song | undefined)[]>(
			this.#bus,
			this.packageName,
			"add-songs",
			songs,
		);
	}

	public async updateSong(song: Song) {
		return sendAsync<Song>(this.#bus, this.packageName, "update-song", song);
	}

	public async removeSong(song: Song | string) {
		let parsedSong: Song | undefined;
		if (typeof song === "string") {
			const searchRes = await this.getSongs({
				song: {
					_id: song,
				},
			});

			parsedSong = searchRes?.at(0);
		} else {
			parsedSong = song;
		}

		if (parsedSong) {
			return sendAsync<void>(this.#bus, this.packageName, "remove-song", song);
		}
	}

	public async addPlaylist(playlist: Omit<Playlist, "playlist_id">) {
		return (
			(await sendAsync<string>(
				this.#bus,
				this.packageName,
				"add-playlist",
				playlist,
			)) ?? ""
		);
	}

	public async addSongsToPlaylist(playlistID: string, ...songs: Song[]) {
		return sendAsync<void>(
			this.#bus,
			this.packageName,
			"add-song-to-playlist",
			{
				playlistID,
				songs,
			},
		);
	}

	public async registerOAuth(path: string) {
		return sendAsync<void>(this.#bus, this.packageName, "register-oauth", path);
	}

	public async openExternalURL(url: string): Promise<void> {
		return sendAsync<void>(this.#bus, this.packageName, "open-external", url);
	}

	public async showToast(
		message: string,
		duration?: number,
		type?: "success" | "info" | "error" | "default",
	) {
		return sendAsync<void>(this.#bus, this.packageName, "show-toast", {
			type,
			duration,
			message,
		});
	}

	public async registerAccount(
		name: string,
		bgColor: string,
		icon: string,
		signinCallback: AccountDetails["signinCallback"],
		signoutCallback: AccountDetails["signoutCallback"],
	): Promise<string> {
		const id = `account:${this.packageName}:${this.accountsMap.length}`;
		const final: AccountDetails = {
			id,
			packageName: this.packageName,
			name,
			bgColor,
			icon,
			signinCallback,
			signoutCallback,
			loggedIn: false,
		};
		this.accountsMap.push(final);
		await sendAsync<void>(
			this.#bus,
			this.packageName,
			"register-account",
			final,
		);
		return id;
	}

	public async changeAccountAuthStatus(
		id: string,
		loggedIn: boolean,
		username?: string,
	) {
		const item = this.accountsMap.find((val) => val.id === id);
		if (item) {
			item.username = username;
			item.loggedIn = loggedIn;
			await sendAsync<void>(
				this.#bus,
				this.packageName,
				"register-account",
				item,
			);
		}
	}

	public async openLoginModal(data: LoginModalOptions) {
		return (
			(await sendAsync<boolean>(
				this.#bus,
				this.packageName,
				"open-login-modal",
				{
					packageName: this.packageName,
					...data,
				},
			)) ?? false
		);
	}

	public async closeLoginModal() {
		return await sendAsync<void>(
			this.#bus,
			this.packageName,
			"close-login-modal",
		);
	}

	private async extensionUpdated() {
		return sendAsync<void>(this.#bus, this.packageName, "extension-updated");
	}

	public on(eventName: ExtraExtensionEventTypes, callback: unknown) {
		console.debug(
			"Registering listener for",
			eventName,
			"in package",
			this.packageName,
		);
		this.eventCallbackMap[eventName] = callback;

		this.extensionUpdated();

		if (eventName === "requestedPlaylists") {
			this.registerPlaylistProvider();
		}
		return callback;
	}

	public off<T extends ExtraExtensionEventTypes>(eventName: T) {
		console.debug(
			"Removing listener for",
			eventName,
			"in package",
			this.packageName,
		);
		delete this.eventCallbackMap[eventName];
	}

	public async _emit<T extends ExtraExtensionEventTypes>(
		event: ExtraExtensionEvents<T>,
	) {
		const callback = this.eventCallbackMap[event.type] as (
			...data: ExtraExtensionEventData<T>
		) => Promise<ExtraExtensionEventReturnType<T>>;
		if (callback && typeof callback === "function") {
			console.debug("emitting", event.type, "in package", this.packageName);
			return (await callback(
				...event.data,
			)) as ExtraExtensionEventReturnType<T>;
		}
	}

	private generateExtendedContextMenuItems<T extends ContextMenuTypes>(
		...items: ExtensionContextMenuItem<T>[]
	): ExtendedExtensionContextMenuItems<T>[] {
		const ret: ExtendedExtensionContextMenuItems<T>[] = [];
		for (const m of items) {
			let children: ExtendedExtensionContextMenuItems<T>[] | undefined =
				undefined;
			if (m.children) {
				children = this.generateExtendedContextMenuItems(...m.children);
			}
			ret.push({
				...m,
				id: crypto.randomUUID(),
				children,
				packageName: this.packageName,
			});
		}

		console.debug("Generated extended type for context menu items");

		return ret;
	}

	public setContextMenuItem<T extends ContextMenuTypes>(
		...items: ExtensionContextMenuItem<T>[]
	): number {
		console.debug(
			"Adding context menu items for types",
			items.map((val) => val.type).join(", "),
		);
		return this.contextMenuMap.push(
			...this.generateExtendedContextMenuItems(...items),
		);
	}

	public removeContextMenuItem(index: number) {
		console.debug("Removing context menu items at", index);
		this.contextMenuMap.splice(index, 1);
	}

	public getContextMenuItems() {
		return this.contextMenuMap;
	}

	public getInstalledExtensions() {
		return Array.from(this.#communicator.extensionRetriever()).map(
			(val) => val.packageName,
		);
	}

	public _getContextMenuItems(): ExtendedExtensionContextMenuItems<ContextMenuTypes>[] {
		return JSON.parse(JSON.stringify(this.getContextMenuItems()));
	}

	public _getAccountDetails(): AccountDetails[] {
		return this.accountsMap;
	}

	public registerSearchProvider() {
		console.warn("Deprecated API, please update the extension");
	}

	public registerArtistSongProvider() {
		console.warn("Deprecated API, please update the extension");
	}

	public registerAlbumSongProvider() {
		console.warn("Deprecated API, please update the extension");
	}

	private registerPlaylistProvider() {
		console.warn("Deprecated API, please update the extension");
	}

	public setArtistEditableInfo(
		artist_id: string,
		object: Record<string, string>,
	) {
		return sendAsync<void>(
			this.#bus,
			this.packageName,
			"set-artist-editable-info",
			{
				artist_id,
				object,
			},
		);
	}

	public setAlbumEditableInfo(
		album_id: string,
		object: Record<string, string>,
	) {
		return sendAsync<void>(
			this.#bus,
			this.packageName,
			"set-album-editable-info",
			{
				album_id,
				object,
			},
		);
	}

	public _isEventCallbackRegistered(key: string) {
		return !!this.eventCallbackMap[key];
	}

	addUserPreference(pref: ExtensionPreferenceGroup): void {
		this.#communicator.addPreference(this.packageName, pref);
	}
	removeUserPreference(key: string): void {
		this.#communicator.removePreference(this.packageName, key);
	}
}

class Utils implements utils {
	private _packageName: string;

	constructor(packageName: string) {
		this._packageName = packageName;
	}

	public getArtistExtraInfo(artist: Artists) {
		if (artist?.artist_extra_info?.extensions)
			return artist.artist_extra_info.extensions[this.packageName] as Record<
				string,
				string
			>;
		return undefined;
	}

	public getAlbumExtraInfo(album: Album) {
		if (album?.album_extra_info?.extensions)
			return album?.album_extra_info?.extensions[this.packageName] as Record<
				string,
				string
			>;
		return undefined;
	}

	public get packageName(): string {
		return this._packageName;
	}

	public get customRequestBaseUrl() {
		return `extension://${this.packageName}`;
	}
}

class PlayerControls implements playerControls {
	private packageName: string;
	#bus: EventEmitter;

	constructor(packageName: string, bus: EventEmitter) {
		this.packageName = packageName;
		this.#bus = bus;
	}

	play(): Promise<void> {
		return sendAsync<void>(this.#bus, this.packageName, "play");
	}

	pause(): Promise<void> {
		return sendAsync<void>(this.#bus, this.packageName, "pause");
	}

	stop(): Promise<void> {
		return sendAsync<void>(this.#bus, this.packageName, "stop");
	}

	nextSong(): Promise<void> {
		return sendAsync<void>(this.#bus, this.packageName, "next");
	}

	prevSong(): Promise<void> {
		return sendAsync<void>(this.#bus, this.packageName, "prev");
	}
}

function sendAsync<T>(
	bus: EventEmitter,
	packageName: string,
	type: extensionRequests,
	data?: unknown,
): Promise<T | undefined> {
	const channel = crypto.randomUUID();
	return new Promise((resolve) => {
		const listener: (data: extensionReplyMessage) => void = (
			data: extensionReplyMessage,
		) => {
			if (data.channel === channel) {
				process.off(channel, listener);
				resolve(data.data as T);
			}
		};
		bus.on(channel, listener);
		bus.emit("request", {
			type,
			channel,
			data,
			extensionName: packageName,
		} as extensionRequestMessage);
	});
}
