use std::collections::HashMap;

use librespot::playback::player::PlayerEvent;

#[tracing::instrument(level = "trace", skip(event))]
pub fn event_to_map(event: PlayerEvent) -> HashMap<String, String> {
    let mut map = HashMap::new();

    match event {
        PlayerEvent::PlayRequestIdChanged { play_request_id } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("event".to_string(), "PlayRequestIdChanged".to_string());
        }
        PlayerEvent::Stopped {
            play_request_id,
            track_id,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("event".to_string(), "Stopped".to_string());
        }
        PlayerEvent::Loading {
            play_request_id,
            track_id,
            position_ms,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("position_ms".to_string(), position_ms.to_string());
            map.insert("event".to_string(), "Loading".to_string());
        }
        PlayerEvent::Playing {
            play_request_id,
            track_id,
            position_ms,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("position_ms".to_string(), position_ms.to_string());
            map.insert("event".to_string(), "Playing".to_string());
        }
        PlayerEvent::Paused {
            play_request_id,
            track_id,
            position_ms,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("position_ms".to_string(), position_ms.to_string());
            map.insert("event".to_string(), "Paused".to_string());
        }
        PlayerEvent::TimeToPreloadNextTrack {
            play_request_id,
            track_id,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("event".to_string(), "TimeToPreloadNextTrack".to_string());
        }
        PlayerEvent::EndOfTrack {
            play_request_id,
            track_id,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("event".to_string(), "EndOfTrack".to_string());
        }
        PlayerEvent::Unavailable {
            play_request_id,
            track_id,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("event".to_string(), "Unavailable".to_string());
        }
        PlayerEvent::PositionCorrection {
            play_request_id,
            track_id,
            position_ms,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("position_ms".to_string(), position_ms.to_string());
            map.insert("event".to_string(), "PositionCorrection".to_string());
        }
        PlayerEvent::Seeked {
            play_request_id,
            track_id,
            position_ms,
        } => {
            map.insert("play_request_id".to_string(), play_request_id.to_string());
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("position_ms".to_string(), position_ms.to_string());
            map.insert("event".to_string(), "Seeked".to_string());
        }
        PlayerEvent::Preloading { track_id } => {
            map.insert("track_id".to_string(), track_id.to_string());
            map.insert("event".to_string(), "Preloading".to_string());
        }
        PlayerEvent::TrackChanged { audio_item } => {
            map.insert("audio_item".to_string(), audio_item.track_id.to_string());
            map.insert("event".to_string(), "TrackChanged".to_string());
        }
        PlayerEvent::VolumeChanged { volume } => {
            map.insert("volume".to_string(), volume.to_string());
            map.insert("event".to_string(), "VolumeChanged".to_string());
        }
        PlayerEvent::SessionConnected {
            connection_id,
            user_name,
        } => {
            map.insert("connection_id".to_string(), connection_id);
            map.insert("user_name".to_string(), user_name);
            map.insert("event".to_string(), "SessionConnected".to_string());
        }
        PlayerEvent::SessionDisconnected {
            connection_id,
            user_name,
        } => {
            map.insert("connection_id".to_string(), connection_id);
            map.insert("user_name".to_string(), user_name);
            map.insert("event".to_string(), "SessionDisconnected".to_string());
        }
        PlayerEvent::ShuffleChanged { shuffle } => {
            map.insert("shuffle".to_string(), shuffle.to_string());
            map.insert("event".to_string(), "ShuffleChanged".to_string());
        }
        PlayerEvent::RepeatChanged { context, track } => {
            map.insert("context".to_string(), context.to_string());
            map.insert("track".to_string(), track.to_string());
            map.insert("event".to_string(), "RepeatChanged".to_string());
        }
        PlayerEvent::AutoPlayChanged { auto_play } => {
            map.insert("auto_play".to_string(), auto_play.to_string());
            map.insert("event".to_string(), "AutoPlayChanged".to_string());
        }
        PlayerEvent::FilterExplicitContentChanged { filter } => {
            map.insert("filter".to_string(), filter.to_string());
            map.insert(
                "event".to_string(),
                "FilterExplicitContentChanged".to_string(),
            );
        }
        PlayerEvent::SessionClientChanged {
            client_id,
            client_name,
            client_brand_name,
            client_model_name,
        } => {
            map.insert("client_id".to_string(), client_id.to_string());
            map.insert("client_name".to_string(), client_name.to_string());
            map.insert(
                "client_brand_name".to_string(),
                client_brand_name.to_string(),
            );
            map.insert(
                "client_model_name".to_string(),
                client_model_name.to_string(),
            );
            map.insert("event".to_string(), "SessionClientChanged".to_string());
        }
    }

    map
}
