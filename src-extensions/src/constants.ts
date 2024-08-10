/*
 *  constants.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

export const extensionRequestsKeys = [
  "getSongs",
  "getEntity",
  "updateSong",
  "update-song",
  "addPlaylist",
  "addSongToPlaylist",
  "removeSong",
  "getPreferences",
  "getSecurePreferences",
  "setPreferences",
  "setSecurePreferences",
  "registerOauth",
  "openExternal",
  "registerAccount",
  "setArtistEditableInfo",
  "setAlbumEditableInfo",
] as const;
export const extensionUIRequestsKeys = [
  "getCurrentSong",
  "getVolume",
  "getTime",
  "getQueue",
  "getPlayerState",
  "openLoginModal",
  "closeLoginModal",
  "showToast",
  "updatePreferences",
  "extensionUpdated",
] as const;
export const playerControlRequests = [
  "play",
  "pause",
  "stop",
  "next",
  "prev",
] as const;
export const mainRequestsKeys = [
  "getInstalledExtensions",
  "findNewExtensions",
  "toggleExtensionStatus",
  "removeExtension",
  "stopProcess",
  "getExtensionIcon",
  "extraExtensionEvents",
  "getExtensionContextMenu",
  "onClickedContextMenu",
  "set-log-level",
  "getAccounts",
  "performAccountLogin",
  "getDisplayName",
] as const;

export const providerExtensionKeys = ["getExtensionProviderScopes"] as const;

export type extensionUIRequests =
  | (typeof extensionUIRequestsKeys)[number]
  | (typeof playerControlRequests)[number];
export type extensionRequests =
  | (typeof extensionRequestsKeys)[number]
  | extensionUIRequests;
export type mainRequests = (typeof mainRequestsKeys)[number];
export type providerFetchRequests = (typeof providerExtensionKeys)[number];
