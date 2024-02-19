use std::sync::Arc;

use futures_util::StreamExt;

use librespot;
use librespot::core::cache::Cache;
use librespot::core::SpotifyId;
use librespot::core::{authentication::Credentials, config::SessionConfig, session::Session};
use librespot::discovery::DeviceType;


use librespot::playback::config::{PlayerConfig, VolumeCtrl};
use librespot::playback::mixer::{Mixer, MixerConfig};
use librespot::playback::player::Player;
use librespot::playback::{audio_backend, mixer};
use protobuf::Message;
use reqwest::header::{CONTENT_LENGTH, CONTENT_TYPE};
use tokio;

use types::errors::errors::{Result};

use crate::canvaz::entity_canvaz_request::Entity;
use crate::canvaz::{EntityCanvazRequest, EntityCanvazResponse};

pub fn new_player(
    backend_str: String,
    session: Session,
    player_config: PlayerConfig,
    volume_ctrl: String,
) -> (Arc<Player>, Arc<dyn Mixer>) {
    let backend = if backend_str.is_empty() {
        audio_backend::find(Some("rodio".to_string())).unwrap()
    } else {
        audio_backend::find(Some(backend_str)).unwrap()
    };

    let mut mixer_config = MixerConfig::default();
    mixer_config.volume_ctrl =
        VolumeCtrl::from_str_with_range(volume_ctrl.as_str(), VolumeCtrl::DEFAULT_DB_RANGE)
            .unwrap_or_else(|_| VolumeCtrl::Log(VolumeCtrl::DEFAULT_DB_RANGE));

    let mixer = mixer::find(None).unwrap()(mixer_config);

    let p = Player::new(
        player_config,
        session.clone(),
        mixer.get_soft_volume(),
        move || (backend)(None, librespot::playback::config::AudioFormat::F32),
    );

    (p, mixer)
}

pub fn create_session(cache_config: Cache) -> Session {
    let session_config = SessionConfig::default();
    let session = Session::new(session_config, Some(cache_config));

    session
}

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

pub fn get_lyrics(track_uri: String, session: Session) -> Result<String> {
    let session_clone = session.clone();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    runtime.block_on(async {
        let track_id_res = SpotifyId::from_uri(track_uri.as_str())?;

        let resp = session_clone.spclient().get_lyrics(&track_id_res).await?;

        let str = String::from_utf8(resp.to_vec())?;
        Ok(str)
    })
}

pub fn get_canvas(track_uri: String, session: Session) -> Result<EntityCanvazResponse> {
    let session_clone = session.clone();
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_io()
        .enable_time()
        .build()
        .unwrap();

    runtime.block_on(async {
        let spclient = session_clone.spclient();

        let mut req = EntityCanvazRequest::new();
        let mut entity = Entity::new();
        entity.entity_uri = track_uri.clone();
        req.entities.push(entity.clone());

        println!("{}", protobuf::text_format::print_to_string(&req));

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

        Ok(data)
    })
}
