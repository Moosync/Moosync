interface NormalizationConfig {
	normalization: boolean;
	normalizationPregain: number;
	normalizationType: "auto" | "album" | "track";
	normalizationMethod: "dynamic" | "basic";
	normalizationAttackCF: number;
	normalizationKneeDB: number;
	normalizationReleaseCF: number;
	normalizationThreshold: number;
}

interface ConnectConfig {
	name: string;
	deviceType:
		| "computer"
		| "tablet"
		| "smartphone"
		| "speaker"
		| "tv"
		| "avr"
		| "stb"
		| "audiodongle"
		| "gameconsole"
		| "castaudio"
		| "castvideo"
		| "automobile"
		| "smartwatch"
		| "chromebook"
		| "carthing"
		| "homething";
	initialVolume: number;
	hasVolumeControl: boolean;
}

interface CacheConfig {
	credentials_location?: string;
	volume_location?: string;
	audio_location?: string;
	size_limiter?: number;
}

/**
 * Config to pass to librespot instance
 */
interface ConstructorConfig {
	/**
	 * Authentication config
	 */
	auth: Partial<AuthDetails>;

	/**
	 * Cache config
	 */
	cache?: CacheConfig;

	/**
	 * Interval at which player position is updated. (milliseconds)
	 */
	pos_update_interval?: number;

	/**
	 * Librespot backend to use (Default rodio)
	 * Possible values rodio, pipe, subprocess
	 */
	backend?: string;

	/**
	 * Enable gapless playback (Default false)
	 */
	gapless?: boolean;

	/**
	 * Bitrate to use.
	 * Possible values 96, 160, 320
	 */
	bitrate?: "96" | "160" | "320";

	passThrough?: boolean;

	/**
	 * Volume normalization config
	 */
	normalizationConfig?: Partial<NormalizationConfig>;

	/**
	 * Spotify connect config (Works only wit SPIRC player)
	 */
	connectConfig?: Partial<ConnectConfig>;

	/**
	 * Mixer volume control type
	 */
	volumeCtrl?: "cubic" | "fixed" | "linear" | "log";

	logLevel?: "debug" | "info" | "trace" | "warn" | "error";
}

interface AuthDetails {
	username: string;
	password: string;
	authType?:
		| "AUTHENTICATION_USER_PASS"
		| "AUTHENTICATION_USER_PASS"
		| "AUTHENTICATION_STORED_FACEBOOK_CREDENTIALS"
		| "AUTHENTICATION_SPOTIFY_TOKEN"
		| "AUTHENTICATION_FACEBOOK_TOKEN";
}

/**
 * Events emitted by player
 */
type PlayerEventTypes =
	| "Stopped"
	| "Loading"
	| "Preloading"
	| "Playing"
	| "Paused"
	| "TimeToPreloadNextTrack"
	| "EndOfTrack"
	| "Unavailable"
	| "VolumeChanged"
	| "PositionCorrection"
	| "Seeked"
	| "FilterExplicitContentChanged"
	| "TrackChanged"
	| "SessionConnected"
	| "SessionDisconnected"
	| "SessionClientChanged"
	| "ShuffleChanged"
	| "RepeatChanged"
	| "AutoPlayChanged"
	| "PlayerInitialized"
	| "TimeUpdated"
	| "InitializationError";

type PlayerEvent<T extends PlayerEventTypes = "InitializationError"> = {
	event: T;
} & (T extends "Stopped"
	? {
			play_request_id: bigint;
			track_id: string;
	  }
	: T extends "Loading"
	  ? {
				play_request_id: bigint;
				track_id: string;
				position_ms: number;
		  }
	  : T extends "Preloading"
		  ? {
					track_id: string;
			  }
		  : T extends "Playing"
			  ? {
						play_request_id: bigint;
						track_id: string;
						position_ms: number;
				  }
			  : T extends "Paused"
				  ? {
							play_request_id: bigint;
							track_id: string;
							position_ms: number;
					  }
				  : T extends "TimeToPreloadNextTrack"
					  ? {
								play_request_id: bigint;
								track_id: string;
						  }
					  : T extends "EndOfTrack"
						  ? {
									play_request_id: bigint;
									track_id: string;
							  }
						  : T extends "Unavailable"
							  ? {
										play_request_id: bigint;
										track_id: string;
								  }
							  : T extends "VolumeChanged"
								  ? {
											volume: number;
									  }
								  : T extends "PositionCorrection"
									  ? {
												play_request_id: bigint;
												track_id: string;
												position_ms: number;
										  }
									  : T extends "Seeked"
										  ? {
													play_request_id: bigint;
													track_id: string;
													position_ms: number;
											  }
										  : T extends "TrackChanged"
											  ? {
														audio_item: string;
												  }
											  : T extends "SessionConnected"
												  ? {
															connection_id: string;
															user_name: string;
													  }
												  : T extends "SessionDisconnected"
													  ? {
																connection_id: string;
																user_name: string;
														  }
													  : T extends "SessionClientChanged"
														  ? {
																	client_id: string;
																	client_name: string;
																	client_brand_name: string;
																	client_model_name: string;
															  }
														  : T extends "ShuffleChanged"
															  ? {
																		shuffle: boolean;
																  }
															  : T extends "RepeatChanged"
																  ? {
																			repeat: boolean;
																	  }
																  : T extends "AutoPlayChanged"
																	  ? {
																				auto_play: boolean;
																		  }
																	  : T extends "FilterExplicitContentChanged"
																		  ? {
																					filter: boolean;
																			  }
																		  : T extends "TimeUpdated"
																			  ? {
																						position_ms: number;
																				  }
																			  : T extends "PlayerInitialized"
																				  ? undefined
																				  : T extends "InitializationError"
																					  ? { error: Error }
																					  : unknown);

type TokenScope =
	| "ugc-image-upload"
	| "user-read-playback-state"
	| "user-modify-playback-state"
	| "user-read-currently-playing"
	| "app-remote-control"
	| "streaming"
	| "playlist-read-private"
	| "playlist-read-collaborative"
	| "playlist-modify-private"
	| "playlist-modify-public"
	| "user-follow-modify"
	| "user-follow-read"
	| "user-read-playback-position"
	| "user-top-read"
	| "user-read-recently-played"
	| "user-library-modify"
	| "user-library-read"
	| "user-read-email"
	| "user-read-private";

type Token = {
	access_token: string;
	token_type: "Bearer";
	expires_in: number;
	expiry_from_epoch: number;
	scopes: TokenScope[];
};

interface CanvazResponse {
	canvases: Canvaz[];
	ttl_in_seconds: number;
}

interface Canvaz {
	id: string;
	url: string;
	file_id: string;
	entity_uri: string;
	explicit: boolean;
	uploaded_by: string;
	etag: string;
	canvas_uri: string;
	storylines_id: string;
	type_: number;
	artist: Artist;
}

interface Artist {
	uri: string;
	name: string;
	avatar: string;
}

interface LyricsResponse {
	lyrics: Lyrics;
	colors: Colors;
	hasVocalRemoval: boolean;
}

interface Lyrics {
	syncType: string;
	lines: Line[];
	provider: string;
	providerLyricsId: string;
	providerDisplayName: string;
	syncLyricsUri: string;
	isDenseTypeface: boolean;
	alternatives: any[];
	language: string;
	isRtlLanguage: boolean;
	fullscreenAction: string;
}

interface Line {
	startTimeMs: string;
	words: string;
	syllables: any[];
	endTimeMs: string;
}

interface Colors {
	background: number;
	text: number;
	highlightText: number;
}
