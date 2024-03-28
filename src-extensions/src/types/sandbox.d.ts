/* eslint-disable @typescript-eslint/no-explicit-any */
/*
 *  sandbox.d.ts is a part of Moosync.
 *
 *  Copyright 2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

type GithubRepoResponse = {
	sha: string;
	url: string;
	tree: {
		path: string;
		mode: string;
		type: "blob" | "tree";
		sha: string;
		url: string;
	}[];
};

type FetchedExtensionManifest = {
	name: string;
	packageName: string;
	logo: string;
	description: string;
	url: string;
	release: {
		type: "github-release" | "url";
		url: string;
		version: string;
	};
};

type ExtInstallStatus = {
	packageName: string;
	status: string;
	error?: unknown;
	progress: number;
};

type extensionEventMessage = {
	type: keyof MoosyncExtensionTemplate;
	data: unknown;
	packageName?: string;
};

type extensionRequestMessage = {
	type: import("../constants").extensionRequests;
	channel: string;
	data: unknown;
	extensionName: string;
};

type extensionUIRequestMessage = {
	type: import("../constants").extensionUIRequests;
	channel: string;
	data: unknown;
	extensionName: string;
};

type extensionReplyMessage = extensionRequestMessage;

type extensionHostMessage =
	| extensionEventMessage
	| mainRequestMessage
	| providerFetchRequestMessage;

type mainRequestMessage = {
	type: import("../constants").mainRequests;
	channel: string;
	data: {
		packageName: string;
	} & Record<string, unknown>;
};

type providerFetchRequestMessage = {
	type: import("../constants").providerFetchRequests;
	channel: string;
	data: {
		packageName: string;
	} & Record<string, unknown>;
};

type mainReplyMessage = mainRequestMessage;

type mainHostMessage =
	| mainReplyMessage
	| extensionRequestMessage
	| {
			type: "get-all-songs";
			data: undefined;
	  }
	| {
			type: "get-installed-extensions";
			data: ExtensionDetails[];
	  };

interface installMessage {
	success: boolean;
	message?: string;
}

interface ExtensionDetails {
	name: string;
	packageName: string;
	desc: string;
	author: string;
	version: string;
	hasStarted: boolean;
	entry: string;
	preferences: ExtensionPreferenceGroup[];
	extensionPath: string;
	extensionIcon: string | undefined;
}

type ExtraExtensionEventCombinedReturnType<T extends ExtraExtensionEventTypes> =
	{
		[key: string]: ExtraExtensionEventReturnType<T>;
	};

interface ExtraExtensionEvents<T extends ExtraExtensionEventTypes> {
	type: T;
	data: ExtraExtensionEventData<T>;
	packageName?: string;
}

interface ExtendedExtensionAPI extends extensionAPI {
	_emit: <T extends ExtraExtensionEventTypes>(
		event: ExtraExtensionEvents<T>,
	) => Promise<ExtraExtensionEventReturnType<T> | undefined>;
	_getContextMenuItems: () => ExtendedExtensionContextMenuItems<ContextMenuTypes>[];
	_getAccountDetails: () => AccountDetails[];
	_isEventCallbackRegistered: (key: ExtraExtensionEventTypes) => boolean;
}

interface ExtensionItem extends ExtensionDetails {
	instance: MoosyncExtensionTemplate;
	preferences: ExtensionPreferenceGroup[];
	vm: import("vm2").NodeVM;
	global: {
		api: ExtendedExtensionAPI;
	};
}

interface UnInitializedExtensionItem {
	name: string;
	packageName: string;
	desc: string;
	author: string;
	version: string;
	entry: string;
	extensionPath: string;
	extensionIcon: string | undefined;
}

interface getExtensionOptions {
	started?: boolean;
	packageName?: string;
}

interface ExtendedExtensionContextMenuItems<T extends ContextMenuTypes>
	extends Omit<ExtensionContextMenuItem<T>, "children"> {
	id: string;
	packageName: string;
	children?: ExtendedExtensionContextMenuItems<T>[];
}

interface ExtensionCommunicator {
	extensionRetriever: (
		options?: getExtensionOptions,
	) => Iterable<ExtensionItem>;
	addPreference: (
		packageName: string,
		preference: ExtensionPreferenceGroup,
	) => void;
	removePreference: (packageName: string, key: string) => void;
}

interface NodeRequire {
	(
		dependencies: string[],
		callback: (...args: unknown[]) => unknown,
		errorback?: (err: unknown) => void,
	): unknown;
	config(data: unknown): unknown;
	onError: (err: Error) => void;
	__$__nodeRequire<T>(moduleName: string): T;
	getStats(): ReadonlyArray<LoaderEvent>;
	hasDependencyCycle(): boolean;
	define(
		amdModuleId: string,
		dependencies: string[],
		callback: (...args: unknown[]) => unknown,
	): unknown;
}
