/*
 *  player.ts is a part of Moosync.
 *
 *  Copyright 2021-2022 by Sahil Gupte <sahilsachingupte@gmail.com>. All rights reserved.
 *  Licensed under the GNU General Public License.
 *
 *  See LICENSE in the project root for license information.
 */

function checkInitialized(
	target: unknown,
	propertyKey: string,
	descriptor: PropertyDescriptor,
) {
	const originalMethod = descriptor.value;

	descriptor.value = function (...args: unknown[]) {
		if ((this as Player).isInitialized) {
			return originalMethod.bind(this)(...args);
		}
		throw new Error("Player has not been initialized yet");
	};

	return descriptor;
}

export abstract class Player {
	public isInitialized = false;

	protected abstract _initialize(config?: unknown): Promise<void>;
	public abstract provides(): PlayerTypes[];

	public async initialize(...config: Parameters<typeof this._initialize>) {
		await this._initialize(...config);
		this.isInitialized = true;
	}

	abstract readonly key: string;

	protected abstract _load(
		src?: string,
		volume?: number,
		autoplay?: boolean,
	): void;
	protected abstract _play(): Promise<void>;
	protected abstract _pause(): void;
	protected abstract _stop(): void;

	@checkInitialized
	public load(
		src?: string,
		volume?: number,
		autoplay?: boolean,
	): void | Promise<void> {
		console.debug("Loading", src, "volume", volume, "autoplay", autoplay);
		return this._load(src, volume, autoplay);
	}

	@checkInitialized
	public play() {
		return this._play();
	}

	@checkInitialized
	public pause() {
		return this._pause();
	}

	@checkInitialized
	public stop() {
		return this._stop();
	}

	abstract get currentTime(): number;
	abstract set currentTime(time: number);

	abstract get volume(): number;
	abstract set volume(volume: number);

	// eslint-disable-next-line @typescript-eslint/no-explicit-any
	private listenerMod(
		eventName: string,
		callback: (e?: unknown) => void,
		e?: unknown,
	) {
		console.debug(
			this.key,
			"player, got event",
			`${eventName}${e ? `: ${e}` : ""}`,
		);
		callback(e);
	}

	set onEnded(callback: () => void) {
		const mod = () => this.listenerMod("onEnded", callback);
		this.listenOnEnded(mod);
		console.debug("Set onEnded callback");
	}

	set onTimeUpdate(callback: (time: number) => void) {
		this.listenOnTimeUpdate(callback);
		console.debug("Set onTimeUpdate callback");
	}

	set onLoad(callback: () => void) {
		const mod = () => this.listenerMod("onLoad", callback);
		this.listenOnLoad(mod);
		console.debug("Set onLoad callback");
	}

	set onError(callback: (err: Error) => void) {
		const mod = (err: Error) =>
			this.listenerMod(
				"onError",
				callback as Parameters<typeof this.listenerMod>[1],
				err,
			);
		this.listenOnError(mod);
		console.debug("Set onError callback");
	}

	set onStateChange(callback: (state: PlayerState) => void) {
		const mod = (state: PlayerState) =>
			this.listenerMod(
				"onStateChange",
				callback as Parameters<typeof this.listenerMod>[1],
				state,
			);
		this.listenOnStateChange(mod);
		console.debug("Set onStateChange callback");
	}

	set onBuffer(callback: () => void) {
		const mod = () => this.listenerMod("onBuffer", callback);
		this.listenOnBuffer(mod);
		console.debug("Set onBuffer callback");
	}

	protected abstract listenOnEnded(callback: () => void): void;
	protected abstract listenOnTimeUpdate(callback: (time: number) => void): void;
	protected abstract listenOnLoad(callback: () => void): void;
	protected abstract listenOnError(callback: (err: Error) => void): void;
	protected abstract listenOnStateChange(
		callback: (state: PlayerState) => void,
	): void;
	protected abstract listenOnBuffer(callback: () => void): void;
	abstract removeAllListeners(): void;

	abstract createAudioContext(): AudioContext | undefined;
	abstract connectAudioContextNode(node: AudioNode): void;

	abstract preload(src: string): void;

	public abstract canPlay(src: string): Promise<boolean>;

	public async close() {
		return;
	}
}
