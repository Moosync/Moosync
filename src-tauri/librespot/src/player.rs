// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use std::sync::Arc;

use futures_util::StreamExt;

use librespot::core::cache::Cache;
use librespot::core::SpotifyId;
use librespot::core::{authentication::Credentials, config::SessionConfig, session::Session};
use librespot::discovery::DeviceType;

use librespot::playback::config::{PlayerConfig, VolumeCtrl};
use librespot::playback::mixer::{Mixer, MixerConfig};
use librespot::playback::player::Player;
use librespot::playback::{audio_backend, mixer};
use protobuf::Message;
use regex::Regex;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};

use types::canvaz::{Canvaz, CanvazArtist, CanvazResponse, Type};
use types::errors::Result;
use url::Url;

use crate::canvaz::entity_canvaz_request::Entity;
use crate::canvaz::{EntityCanvazRequest, EntityCanvazResponse};

#[tracing::instrument(
    level = "trace",
    skip(backend_str, session, player_config, volume_ctrl)
)]
pub fn new_player(
    backend_str: String,
    session: Session,
    player_config: PlayerConfig,
    volume_ctrl: String,
) -> (Arc<Player>, Arc<dyn Mixer>) {
    let backend = if backend_str.is_empty() {
        audio_backend::BACKENDS[0].1
    } else {
        audio_backend::find(Some(backend_str)).unwrap()
    };

    let mixer_config = MixerConfig {
        volume_ctrl: VolumeCtrl::from_str_with_range(
            volume_ctrl.as_str(),
            VolumeCtrl::DEFAULT_DB_RANGE,
        )
        .unwrap_or(VolumeCtrl::Linear),
        ..Default::default()
    };

    let mixer = mixer::find(None).unwrap()(mixer_config);

    let p = Player::new(
        player_config,
        session.clone(),
        mixer.get_soft_volume(),
        move || (backend)(None, librespot::playback::config::AudioFormat::F32),
    );

    (p, mixer)
}

pub(crate) const KEYMASTER_CLIENT_ID: &str = "65b708073fc0480ea92a077233ca87bd";

#[tracing::instrument(level = "debug", skip(cache_config))]
pub fn create_session(cache_config: Cache) -> Session {
    let device_id = uuid::Uuid::new_v4().as_hyphenated().to_string();
    let client_id = KEYMASTER_CLIENT_ID.to_owned();
    let session_config = SessionConfig {
        client_id,
        device_id,
        proxy: None,
        ap_port: None,
        tmp_dir: std::env::temp_dir(),
        autoplay: None,
    };

    Session::new(session_config, Some(cache_config))
}

#[tracing::instrument(level = "debug", skip(client_id))]
#[allow(dead_code)]
#[tokio::main]
pub async fn start_discovery(client_id: String) -> Credentials {
    let device_id = "test";

    let mut discovery = librespot::discovery::Discovery::builder(device_id, client_id.as_str())
        .name("test device")
        .device_type(DeviceType::Computer)
        .port(9001)
        .launch()
        .unwrap();

    discovery.next().await.unwrap()
}

#[tracing::instrument(level = "debug", skip(track_uri, session))]
pub fn get_lyrics(track_uri: String, session: Session) -> Result<String> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    runtime.block_on(async {
        let track_id_res = SpotifyId::from_uri(track_uri.as_str())?;

        let resp = session.spclient().get_lyrics(&track_id_res).await?;

        let str = String::from_utf8(resp.to_vec())?;
        Ok(str)
    })
}

#[tracing::instrument(level = "debug", skip(canvaz))]
fn parse_canvaz(canvaz: EntityCanvazResponse) -> Result<CanvazResponse> {
    Ok(CanvazResponse {
        canvases: canvaz
            .canvases
            .iter()
            .map(|c| {
                let artist = CanvazArtist {
                    uri: c.artist.uri.clone(),
                    name: c.artist.name.clone(),
                    avatar: c.artist.avatar.clone(),
                };
                let type_ = match c.type_.enum_value_or_default() {
                    crate::canvaz::Type::IMAGE => Type::Image,
                    crate::canvaz::Type::VIDEO => Type::Video,
                    crate::canvaz::Type::VIDEO_LOOPING => Type::VideoLooping,
                    crate::canvaz::Type::VIDEO_LOOPING_RANDOM => Type::VideoLoopingRandom,
                    crate::canvaz::Type::GIF => Type::Gif,
                };
                Canvaz {
                    id: c.id.clone(),
                    url: c.url.clone(),
                    file_id: c.file_id.clone(),
                    type_,
                    entity_uri: c.entity_uri.clone(),
                    artist,
                    explicit: c.explicit,
                    uploaded_by: c.uploaded_by.clone(),
                    etag: c.etag.clone(),
                    canvas_uri: c.canvas_uri.clone(),
                    storylines_id: c.storylines_id.clone(),
                }
            })
            .collect(),
        ttl_in_seconds: canvaz.ttl_in_seconds,
    })
}

#[tracing::instrument(level = "debug", skip(val))]
fn validate_uri(val: &str) -> (Option<String>, Option<String>) {
    let track_regex = Regex::new(
            r"^(?P<urlType>(?:spotify:|(?:https?:\/\/(?:open|play)\.spotify\.com\/)))(?:embed)?\/?(?P<type>album|track|playlist|artist)(?::|\/)((?:[0-9a-zA-Z]){22})"
        ).unwrap();
    if let Some(captures) = track_regex.captures(val) {
        if let Some(url_type) = captures.name("urlType") {
            if let Some(match_type) = captures.name("type") {
                if url_type.as_str().starts_with("https") {
                    if let Ok(parsed_url) = Url::parse(val) {
                        if let Some(last_segment) = parsed_url
                            .path_segments()
                            .and_then(|segments| segments.last())
                        {
                            return (
                                Some(format!("spotify:{}:{}", match_type.as_str(), last_segment)),
                                Some(match_type.as_str().to_string()),
                            );
                        }
                    }
                } else {
                    return (Some(val.to_string()), Some(match_type.as_str().to_string()));
                }
            }
        }
    }

    (None, None)
}

#[tracing::instrument(level = "debug", skip(track_uri, session))]
pub fn get_canvas(track_uri: String, session: Session) -> Result<CanvazResponse> {
    let (uri, type_) = validate_uri(&track_uri);
    if let Some(type_) = type_ {
        if type_ != "track" {
            return Err(format!(
                "Spotify URI is not of a track: {}, {}, {:?}",
                track_uri, type_, uri
            )
            .into());
        }

        if uri.is_none() {
            return Err(format!(
                "Failed to parse spotify URI: {}, {}, {:?}",
                track_uri, type_, uri
            )
            .into());
        }
    }

    let uri = uri.unwrap();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    runtime.block_on(async {
        let spclient = session.spclient();

        let mut req = EntityCanvazRequest::new();
        let mut entity = Entity::new();
        entity.entity_uri.clone_from(&uri);
        req.entities.push(entity.clone());

        tracing::info!("{}", protobuf::text_format::print_to_string(&req));

        let url = format!(
            "{}/canvaz-cache/v0/canvases",
            spclient.base_url().await.unwrap()
        );
        let token = session
            .token_provider()
            .get_token("playlist-read")
            .await?
            .access_token;

        let body = req.write_to_bytes()?;

        let resp = reqwest::Client::builder()
            .build()
            .unwrap()
            .post(url)
            .header(CONTENT_TYPE, "application/x-protobuf")
            .bearer_auth(token)
            .header(CONTENT_LENGTH, body.len())
            .body(body)
            .send()
            .await?;

        let bytes = resp.bytes().await?;

        let data = EntityCanvazResponse::parse_from_tokio_bytes(&bytes.clone())?;

        parse_canvaz(data)
    })
}
