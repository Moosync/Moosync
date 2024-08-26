/*
 *  extensionHandler.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

import type { AbstractExtensionManager } from "./extensionManager";
import { ExtensionManager } from "./extensionManager";
import { ExtensionFinder } from "./extensionFinder";
import type { AbstractExtensionFinder } from "./extensionFinder";

import {
  getVersion,
  sanitizeAlbums,
  sanitizeArtists,
  sanitizePlaylist,
  sanitizeSong,
} from "./common";
import { ProviderScopes } from "./commonConstants";
import type { providerFetchRequests } from "../constants";
import type EventEmitter from "node:events";

type CombinedSongsType =
  | SongsReturnType
  | PlaylistAndSongsReturnType
  | RecommendationsReturnType;

export class ExtensionHandler {
  private extensionManager: AbstractExtensionManager;
  private extensionFinder: AbstractExtensionFinder;
  private _initialized_resolver: () => void;
  initialized = new Promise<void>((r) => (this._initialized_resolver = r));
  // eslint-disable-next-line @typescript-eslint/ban-types
  // biome-ignore lint/complexity/noBannedTypes: <explanation>
  private preInitializedCalls: { func: Function; args?: unknown[] }[];

  constructor(
    searchPaths: string[],
    logsPath: string,
    installPath: string,
    bus: EventEmitter,
  ) {
    this.preInitializedCalls = [];
    this.extensionManager = new ExtensionManager(logsPath, installPath, bus);
    this.extensionFinder = new ExtensionFinder(searchPaths);

    this.registerPlugins().then(() => {
      this._initialized_resolver();
      for (const [index, f] of this.preInitializedCalls.entries()) {
        if (f.args) {
          f.func.bind(this)(...f.args);
        } else f.func.bind(this)();

        this.preInitializedCalls.splice(index);
      }
    });
  }

  private isDuplicateExtension(ext: UnInitializedExtensionItem) {
    const matches = this.extensionManager.getExtensions({
      packageName: ext.packageName,
    });
    for (const oldExt of matches) {
      const oldVer = getVersion(oldExt.version);
      const newVer = getVersion(ext.version);
      if (newVer > oldVer) {
        this.extensionManager.deregister(oldExt.packageName);
        return false;
      }
      return true;
    }
    return false;
  }

  public async registerPlugins(): Promise<void> {
    for await (const ext of this.extensionFinder.findExtensions()) {
      if (!this.isDuplicateExtension(ext)) {
        await this.extensionManager.instantiateAndRegister(ext);
      }
    }
  }

  public async startAll() {
    await this.initialized;
    await this.toggleExtStatus(undefined, true);
  }

  public async toggleExtStatus(
    packageName: string | undefined,
    enabled: boolean,
  ) {
    const ext = this.extensionManager.getExtensions(
      packageName ? { packageName } : undefined,
    );
    for (const e of ext) {
      if (enabled) {
        this.sendToExtensions(e.packageName, "onStarted");
      } else {
        this.sendToExtensions(e.packageName, "onStopped");
      }
      this.extensionManager.setStarted(e.packageName, enabled);
    }
  }

  public async removeExt(packageName: string) {
    // Shut down extension before removing
    await this.toggleExtStatus(packageName, false);

    this.extensionManager.deregister(packageName);
  }

  public getExtensionIcon(packageName: string) {
    const ext = this.extensionManager.getExtensions({ packageName });
    for (const e of ext) {
      return { [packageName]: e.extensionIcon };
    }
    return undefined;
  }

  public getExtensionAccounts(packageName?: string) {
    const ext = this.extensionManager.getExtensions({ packageName });
    const accountMap: { [key: string]: AccountDetails[] } = {};
    for (const e of ext) {
      accountMap[e.packageName] = e.global.api._getAccountDetails();
    }

    return accountMap;
  }

  public getDisplayName(packageName: string) {
    const ext = this.extensionManager.getExtensions({ packageName });
    for (const e of ext) {
      return e.name;
    }
    return undefined;
  }

  public extensionProvides(
    packageName?: string,
  ): Record<string, ProviderScopes[]> {
    const providesMap: Record<string, ProviderScopes[]> = {};

    const exts = this.extensionManager.getExtensions({ packageName });
    for (const e of exts) {
      if (!providesMap[e.packageName]) {
        providesMap[e.packageName] = [];
      }

      if (e.global.api._isEventCallbackRegistered("requestedSearchResult")) {
        providesMap[e.packageName].push(ProviderScopes.SEARCH);
      }

      if (e.global.api._isEventCallbackRegistered("requestedAlbumSongs")) {
        providesMap[e.packageName].push(ProviderScopes.ALBUM_SONGS);
      }

      if (e.global.api._isEventCallbackRegistered("requestedArtistSongs")) {
        providesMap[e.packageName].push(ProviderScopes.ARTIST_SONGS);
      }

      if (e.global.api._isEventCallbackRegistered("requestedPlaylists")) {
        providesMap[e.packageName].push(ProviderScopes.PLAYLISTS);
      }

      if (e.global.api._isEventCallbackRegistered("requestedPlaylistSongs")) {
        providesMap[e.packageName].push(ProviderScopes.PLAYLIST_SONGS);
      }

      if (e.global.api._isEventCallbackRegistered("requestedPlaylistFromURL")) {
        providesMap[e.packageName].push(ProviderScopes.PLAYLIST_FROM_URL);
      }

      if (e.global.api._isEventCallbackRegistered("requestedSongFromURL")) {
        providesMap[e.packageName].push(ProviderScopes.SONG_FROM_URL);
      }

      if (e.global.api._isEventCallbackRegistered("requestedRecommendations")) {
        providesMap[e.packageName].push(ProviderScopes.RECOMMENDATIONS);
      }

      if (e.global.api._isEventCallbackRegistered("playbackDetailsRequested")) {
        providesMap[e.packageName].push(ProviderScopes.PLAYBACK_DETAILS);
      }
    }

    return providesMap;
  }

  public handleProviderRequests(
    type: providerFetchRequests,
    packageName: string,
  ) {
    if (type === "getExtensionProviderScopes") {
      return this.extensionProvides(packageName);
    }
    return undefined;
  }

  public async performExtensionAccountLogin(
    packageName: string,
    accountId: string,
    loginStatus: boolean,
  ) {
    const ext = this.extensionManager.getExtensions({ packageName });
    for (const e of ext) {
      const account = e.global.api
        ._getAccountDetails()
        .find((val) => val.id === accountId);
      if (account) {
        loginStatus
          ? await account.signinCallback()
          : await account.signoutCallback();
      }
    }
  }

  public sendEvent(event: extensionEventMessage) {
    const method: keyof MoosyncExtensionTemplate = event.type;
    if (this.initialized) {
      this.sendToExtensions(event.packageName, method, event.data);
    } else {
      this.preInitializedCalls.push({ func: this.sendEvent, args: [event] });
    }
  }

  private toExtensionDetails(item: ExtensionItem): ExtensionDetails {
    return {
      name: item.name,
      desc: item.desc,
      author: item.author,
      packageName: item.packageName,
      version: item.version,
      hasStarted: item.hasStarted,
      entry: item.entry,
      preferences: item.preferences,
      extensionPath: item.extensionPath,
      extensionIcon: item.extensionIcon,
    };
  }

  public getInstalledExtensions() {
    const extensions = this.extensionManager.getExtensions();
    const parsed: ExtensionDetails[] = [];
    for (const e of extensions) {
      parsed.push(this.toExtensionDetails(e));
    }
    return parsed;
  }

  public async sendToExtensions(
    packageName: string | undefined,
    method: keyof MoosyncExtensionTemplate,
    args?: unknown,
  ) {
    for (const ext of this.extensionManager.getExtensions({
      started: method === "onStarted" ? false : true,
      packageName,
    })) {
      try {
        console.debug("Trying to send event:", method, "to", ext.packageName);
        if (ext.instance[method]) {
          console.debug("Extension can handle event, sending");
          (ext.instance[method] as (args: unknown) => Promise<void>)(args);
        }
      } catch (e) {
        console.error(e);
      }
    }
  }

  public async stopAllExtensions() {
    console.debug("Stopping all extensions");
    this.toggleExtStatus(undefined, false);
  }

  private isForwardRequest(data: unknown | undefined): boolean {
    return !!(data as { forwardTo: string })?.forwardTo;
  }

  public async sendExtraEventToExtensions<T extends ExtraExtensionEventTypes>(
    event: ExtraExtensionEvents<T>,
  ) {
    const allData: {
      [key: string]: ExtraExtensionEventReturnType<T> | undefined;
    } = {};
    const EventType: T = event.type;
    for (const ext of this.extensionManager.getExtensions({
      started: true,
      packageName: event.packageName,
    })) {
      if (event.type === "requestedPlaylistSongs") {
        event.data[0] = (
          event.data as ExtraExtensionEventData<"requestedPlaylistSongs">
        )[0]?.replace(`${ext.packageName}:`, "");
      }

      const resp = await ext.global.api._emit<T>({
        type: event.type,
        data: event.data,
      });

      const packageName = ext.packageName;

      if (resp && !this.isForwardRequest(resp)) {
        if (event.type === "requestedPlaylists") {
          (resp as PlaylistReturnType).playlists = sanitizePlaylist(
            packageName,
            false,
            ...(resp as PlaylistReturnType).playlists,
          );
        }

        if (EventType === "requestedPlaylistFromURL") {
          (resp as PlaylistAndSongsReturnType).playlist = sanitizePlaylist(
            packageName,
            false,
            (resp as PlaylistAndSongsReturnType).playlist,
          )[0];
          (resp as PlaylistAndSongsReturnType).songs = sanitizeSong(
            packageName,
            ...(resp as PlaylistAndSongsReturnType).songs,
          );
        }

        if (
          EventType === "requestedPlaylistSongs" ||
          EventType === "requestedRecommendations"
        ) {
          (resp as CombinedSongsType).songs = sanitizeSong(
            packageName,
            ...(resp as CombinedSongsType).songs,
          );
        }

        if (
          EventType === "requestedSongFromURL" ||
          EventType === "requestedSongFromId"
        ) {
          (resp as SongReturnType).song = sanitizeSong(
            packageName,
            (resp as SongReturnType).song,
          )[0];
        }

        if (EventType === "requestedSearchResult") {
          (resp as SearchReturnType).songs = sanitizeSong(
            packageName,
            ...(resp as SearchReturnType).songs,
          );
          (resp as SearchReturnType).playlists = sanitizePlaylist(
            packageName,
            false,
            ...(resp as SearchReturnType).playlists,
          );
          (resp as SearchReturnType).artists = sanitizeArtists(
            packageName,
            ...(resp as SearchReturnType).artists,
          );
          (resp as SearchReturnType).albums = sanitizeAlbums(
            packageName,
            ...(resp as SearchReturnType).albums,
          );
        }

        if (EventType === "requestedArtistSongs") {
          (resp as SongsReturnType).songs = sanitizeSong(
            packageName,
            ...(resp as SongsReturnType).songs,
          );
        }

        if (EventType === "requestedAlbumSongs") {
          (resp as SongsReturnType).songs = sanitizeSong(
            packageName,
            ...(resp as SongsReturnType).songs,
          );
        }
      }

      allData[ext.packageName] = resp;
    }

    return allData;
  }

  public getExtensionContextMenu<T extends ContextMenuTypes>(
    type: T,
  ): ExtensionContextMenuItem<T>[] {
    const items: ExtensionContextMenuItem<ContextMenuTypes>[] = [];
    for (const ext of this.extensionManager.getExtensions({ started: true })) {
      items.push(
        ...ext.global.api
          ._getContextMenuItems()
          .filter((val) => val.type === type),
      );
    }

    return items as ExtensionContextMenuItem<T>[];
  }

  public fireContextMenuCallback(
    id: string,
    packageName: string,
    arg: ExtensionContextMenuHandlerArgs<ContextMenuTypes>,
  ) {
    for (const ext of this.extensionManager.getExtensions({
      started: true,
      packageName,
    })) {
      const item = (
        ext.global.api.getContextMenuItems() as ExtendedExtensionContextMenuItems<ContextMenuTypes>[]
      ).find((val) => val.id === id);

      item?.handler?.(arg);
    }
  }
}
