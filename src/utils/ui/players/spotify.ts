import { vxm } from "@/mainWindow/store";
// import { PlayerEvent, PlayerEventTypes } from 'librespot-node'
import { Player } from "./player";
import { UnlistenFn } from "@tauri-apps/api/event";

export class SpotifyPlayer extends Player {
	private listenerMap: Partial<
		Record<PlayerEventTypes, { unlisten: UnlistenFn }>
	> = {};
	private lastVolume = 0;

	private ignorePause = false;

	private loadingCallback: (() => void) | undefined;
	private timeUpdateCallback: ((pos: number) => void) | undefined;
	private errorCallback: ((err: Error) => void) | undefined;
	private autoPlayQueued = false;

	public provides(): PlayerTypes[] {
		return ["SPOTIFY"];
	}

	public async canPlay(src: string): Promise<boolean> {
		await vxm.providers.spotifyProvider.getLoggedIn();
		const shouldPlay = await vxm.providers.spotifyProvider.shouldPlayPremium();
		if (
			vxm.providers.spotifyProvider.canPlayPremium &&
			shouldPlay &&
			src.startsWith("spotify:track:")
		) {
			return true;
		}
		return false;
	}

	get key() {
		return "SPOTIFY";
	}

	async _initialize(): Promise<void> {
		await vxm.providers.spotifyProvider.getLoggedIn();
		return;
	}

	async _load(
		src?: string | undefined,
		volume?: number | undefined,
		autoplay?: boolean | undefined,
	): Promise<void> {
		this.loadingCallback?.();
		try {
			await window.SpotifyPlayer.command("LOAD", [src]);
		} catch (e) {
			console.error(e);
			this.errorCallback?.(e as Error);
			return;
		}
		this.volume = volume || this.volume;
		this.autoPlayQueued = autoplay ?? false;

		// Emit time as 0 after loading song
		this.timeUpdateCallback?.(0);
	}

	async _play(): Promise<void> {
		await window.SpotifyPlayer.command("PLAY");
	}

	async _pause(): Promise<void> {
		await window.SpotifyPlayer.command("PAUSE");
	}

	async _stop(): Promise<void> {
		this.ignorePause = true;
		await window.SpotifyPlayer.command("PAUSE");
		await window.SpotifyPlayer.command("VOLUME", [0]);
	}

	get currentTime(): number {
		return 0;
	}

	set currentTime(time: number) {
		window.SpotifyPlayer.command("SEEK", [time * 1000]);
	}

	get volume(): number {
		return 0;
	}

	set volume(volume: number) {
		this.lastVolume = volume;
		window.SpotifyPlayer.command("VOLUME", [volume]);
	}

	private async registerListener<T extends PlayerEventTypes>(
		event: T,
		listener: (e: PlayerEvent<T>) => void,
	) {
		const unlisten = await window.SpotifyPlayer.on(
			event,
			vxm.providers.spotifyProvider.librespotId ?? "",
			listener,
		);
		this.listenerMap[event] = { unlisten };
	}

	protected listenOnEnded(callback: () => void): void {
		this.registerListener("EndOfTrack", () => {
			this.ignorePause = true;
			callback();
		});
	}

	protected listenOnTimeUpdate(callback: (time: number) => void): void {
		this.timeUpdateCallback = callback;
		this.registerListener("TimeUpdated", (e) => callback(e.position_ms / 1000));
	}

	protected listenOnLoad(callback: () => void): void {
		this.registerListener("TrackChanged", async () => {
			callback();
			console.debug(
				"Spotify player emitting play for autoload",
				this.autoPlayQueued,
			);
			if (this.autoPlayQueued) {
				this.autoPlayQueued = false;
				await window.SpotifyPlayer.command("PLAY");
			}
		});
	}

	protected listenOnError(callback: (err: Error) => void): void {
		this.errorCallback = callback;
		this.registerListener("Unavailable", (e) =>
			callback(
				new Error(`Failed to load track ${e.track_id}. Track unavailable`),
			),
		);
	}

	protected listenOnStateChange(callback: (state: PlayerState) => void): void {
		this.registerListener("Playing", () => {
			callback("PLAYING");
			this.volume = this.lastVolume;
		});

		this.registerListener("Paused", () => {
			if (this.ignorePause) {
				this.ignorePause = false;
				return;
			}
			callback("PAUSED");
		});
		this.registerListener("Stopped", () => callback("STOPPED"));
		this.registerListener("Loading", () => callback("LOADING"));
	}

	protected listenOnBuffer(callback: () => void): void {
		this.registerListener("Loading", () => callback());
	}

	removeAllListeners(): void {
		for (const [_, value] of Object.entries(this.listenerMap)) {
			value.unlisten();
		}
	}

	createAudioContext(): AudioContext | undefined {
		return undefined;
	}

	connectAudioContextNode(): void {
		undefined;
	}

	preload(): void {
		// window.SpotifyPlayer.command('ADD_TO_QUEUE', [src])
	}

	async close() {
		await window.SpotifyPlayer.close();
	}
}
