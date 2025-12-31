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

use std::{
    sync::{
        Arc, Mutex,
        mpsc::{self, Receiver, Sender},
    },
    thread::{self, JoinHandle},
    time::{SystemTime, UNIX_EPOCH},
};

use futures::executor::block_on;
use librespot::{
    connect::{ConnectConfig, LoadRequest, LoadRequestOptions, Spirc},
    core::{Session, SpotifyUri, cache::Cache, token::Token},
    discovery::Credentials,
    playback::{
        config::PlayerConfig,
        player::{PlayerEvent, PlayerEventChannel},
    },
};
use serde::{Deserialize, Serialize};
use tokio::{runtime::Builder, sync::Mutex as AsyncMutex};

use crate::player::{create_session, get_canvas, get_lyrics, new_player};
use types::errors::error_helpers;
use types::{
    canvaz::CanvazResponse,
    errors::{MoosyncError, Result},
};

#[derive(Debug, Serialize, Deserialize)]
pub struct ParsedToken {
    pub access_token: String,
    pub scopes: Vec<String>,
    pub token_type: String,
    pub expires_in: u128,
    pub expiry_from_epoch: u128,
}

#[derive(Debug)]
pub struct SpircWrapper {
    tx: mpsc::Sender<(Message, Sender<Result<MessageReply>>)>,
    pub events_channel: Arc<Mutex<mpsc::Receiver<PlayerEvent>>>,
    channel_close_rx: Arc<Mutex<mpsc::Receiver<()>>>,
    device_id: Arc<AsyncMutex<Option<String>>>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Message {
    Play,
    Pause,
    Close,
    GetToken(String),
    Seek(u32),
    Load(String, bool),
    Volume(u16),
    GetLyrics(String),
    GetCanvaz(String),
}

pub enum MessageReply {
    None,
    GetToken(Token),
    GetLyrics(String),
    GetCanvaz(CanvazResponse),
}

impl SpircWrapper {
    #[tracing::instrument(
        level = "trace",
        skip(
            credentials,
            player_config,
            connect_config,
            cache_config,
            backend,
            volume_ctrl
        )
    )]
    pub fn new(
        credentials: Credentials,
        player_config: PlayerConfig,
        connect_config: ConnectConfig,
        cache_config: Cache,
        backend: String,
        volume_ctrl: String,
    ) -> Result<Self> {
        let device_id_mutex = Arc::new(AsyncMutex::new(None));
        let (tx, rx) = mpsc::channel::<(Message, Sender<Result<MessageReply>>)>();

        let (player_events_tx, player_events_rx) = mpsc::channel::<PlayerEvent>();
        let (channel_close_tx, channel_close_rx) = mpsc::channel::<()>();

        let binding = device_id_mutex.clone();
        thread::spawn(move || {
            let runtime = Builder::new_multi_thread()
                .enable_io()
                .enable_time()
                .build()
                .unwrap();

            let device_id_mutex = binding.clone();
            runtime.block_on(async move {
                let mut device_id_mutex = device_id_mutex.lock().await;
                let session = create_session(cache_config);

                let device_id = session.device_id().to_string();

                let (player, mixer) =
                    new_player(backend, session.clone(), player_config.clone(), volume_ctrl);

                let events_channel = player.get_player_event_channel();

                tracing::info!("Creating spirc");
                let res = Spirc::new(
                    connect_config.clone(),
                    session.clone(),
                    credentials.clone(),
                    player,
                    mixer,
                )
                .await;

                match res {
                    Ok((spirc, spirc_task)) => {
                        tracing::info!("Spirc created");

                        spirc.activate().unwrap();
                        let commands_thread =
                            SpircWrapper::listen_commands(rx, channel_close_tx, spirc, session);
                        let events_thread =
                            SpircWrapper::listen_events(player_events_tx, events_channel);

                        *device_id_mutex = Some(device_id);
                        drop(device_id_mutex);

                        spirc_task.await;

                        commands_thread.join().unwrap();
                        events_thread.join().unwrap();
                    }
                    Err(e) => {
                        tracing::error!("Error creating spirc: {:?}", e);
                    }
                }
            });
        });

        {
            let _ = block_on(device_id_mutex.lock());
        }

        let spirc = Self {
            tx,
            device_id: device_id_mutex,
            events_channel: Arc::new(Mutex::new(player_events_rx)),
            channel_close_rx: Arc::new(Mutex::new(channel_close_rx)),
        };
        spirc.listen_channel_close();
        Ok(spirc)
    }

    fn listen_channel_close(&self) {
        let channel_close_rx = self.channel_close_rx.clone();
        let device_id = self.device_id.clone();
        thread::spawn(move || {
            let channel_close_rx = channel_close_rx.lock().unwrap();
            while channel_close_rx.recv().is_ok() {
                let mut device_id = block_on(device_id.lock());
                device_id.take();
            }
        });
    }

    #[tracing::instrument(level = "debug", skip(tx, events_channel))]
    fn listen_events(
        tx: Sender<PlayerEvent>,
        mut events_channel: PlayerEventChannel,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            loop {
                tracing::trace!("Listening for librespot events");
                let message = events_channel.blocking_recv();
                if let Some(m) = message {
                    tx.send(m.clone()).unwrap();
                    if let PlayerEvent::SessionDisconnected {
                        connection_id: _,
                        user_name: _,
                    } = m
                    {
                        return;
                    }
                } else {
                    tracing::info!("Closing spirc event listener");
                    return;
                }
            }
        })
    }

    #[tracing::instrument(level = "debug", skip(message, tx, spirc, session))]
    fn handle_command(
        message: Message,
        tx: Sender<Result<MessageReply>>,
        spirc: &mut Spirc,
        session: &mut Session,
    ) {
        match message {
            Message::Play => {
                let res = (spirc)
                    .play()
                    .map(|_| MessageReply::None)
                    .map_err(error_helpers::to_media_error);

                tx.send(res).unwrap();
            }

            Message::Pause => {
                let res = (spirc)
                    .pause()
                    .map(|_| MessageReply::None)
                    .map_err(error_helpers::to_media_error);

                tx.send(res).unwrap();
            }

            Message::GetToken(scopes) => {
                let rt = Builder::new_current_thread().build().unwrap();
                let data = rt.block_on(async move {
                    session
                        .token_provider()
                        .get_token(scopes.as_str())
                        .await
                        .map(MessageReply::GetToken)
                        .map_err(error_helpers::to_media_error)
                });

                tx.send(data).unwrap();
            }
            Message::Seek(pos) => {
                let res = spirc
                    .set_position_ms(pos)
                    .map(|_| MessageReply::None)
                    .map_err(error_helpers::to_media_error);
                tx.send(res).unwrap();
            }
            Message::Load(uri, autoplay) => {
                let track_id =
                    SpotifyUri::from_uri(uri.as_str()).map_err(error_helpers::to_media_error);
                match track_id {
                    Err(e) => {
                        tx.send(Err(e)).unwrap();
                    }
                    Ok(_track_id) => {
                        // track_ref.set_gid(Vec::from(track_id.to_raw()));
                        let command = LoadRequest::from_context_uri(
                            uri,
                            LoadRequestOptions {
                                start_playing: autoplay,
                                seek_to: 0,
                                context_options: None,
                                playing_track: None,
                            },
                        );

                        let res = spirc
                            .load(command)
                            .map(|_| MessageReply::None)
                            .map_err(error_helpers::to_media_error);

                        tx.send(res).unwrap();
                    }
                }
            }
            Message::Volume(vol) => {
                let res = spirc
                    .set_volume(vol)
                    .map(|_| MessageReply::None)
                    .map_err(error_helpers::to_media_error);
                tx.send(res).unwrap();
            }
            Message::Close => {
                tx.send(Ok(MessageReply::None)).unwrap();
            }
            Message::GetLyrics(uri) => {
                let lyrics = get_lyrics(uri, session.clone()).map(MessageReply::GetLyrics);
                tx.send(lyrics).unwrap();
            }
            Message::GetCanvaz(uri) => {
                let canvaz = get_canvas(uri, session.clone()).map(MessageReply::GetCanvaz);
                tx.send(canvaz).unwrap();
            }
        };
    }

    #[tracing::instrument(level = "debug", skip(rx, spirc, session))]
    pub fn listen_commands(
        rx: Receiver<(Message, Sender<Result<MessageReply>>)>,
        channel_close_tx: Sender<()>,
        mut spirc: Spirc,
        mut session: Session,
    ) -> JoinHandle<()> {
        thread::spawn(move || {
            loop {
                tracing::trace!("Receiving librespot commands");
                if let Ok((message, tx)) = rx.recv() {
                    if message == Message::Close {
                        spirc.shutdown().unwrap();
                        session.shutdown();
                        tx.send(Ok(MessageReply::None)).unwrap();
                        channel_close_tx.send(()).unwrap();
                        return;
                    }

                    tracing::info!("handling: {:?}", message);
                    Self::handle_command(message, tx, &mut spirc, &mut session);
                } else {
                    channel_close_tx.send(()).unwrap();
                    return;
                }
            }
        })
    }

    #[tracing::instrument(level = "debug", skip(self, command))]
    pub fn send(&self, command: Message) -> Result<MessageReply> {
        let (tx, rx) = mpsc::channel::<Result<MessageReply>>();
        self.tx
            .send((command, tx))
            .map_err(|e| MoosyncError::String(e.to_string()))?;

        rx.recv().unwrap()
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn librespot_close(&self) -> Result<()> {
        self.send(Message::Close)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn librespot_play(&self) -> Result<()> {
        self.send(Message::Play)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn librespot_pause(&self) -> Result<()> {
        self.send(Message::Pause)?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, pos))]
    pub fn librespot_seek(&self, pos: u32) -> Result<()> {
        self.send(Message::Seek(pos))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, volume))]
    pub fn librespot_volume(&self, volume: u16) -> Result<()> {
        self.send(Message::Volume(volume))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, uri, autoplay))]
    pub fn librespot_load(&self, uri: String, autoplay: bool) -> Result<()> {
        self.send(Message::Load(uri, autoplay))?;
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self, scopes))]
    pub fn librespot_get_token(&self, scopes: String) -> Result<ParsedToken> {
        let res = self.send(Message::GetToken(scopes))?;
        match res {
            MessageReply::GetToken(token) => Ok(ParsedToken {
                access_token: token.access_token,
                scopes: token.scopes,
                token_type: token.token_type,
                expires_in: token.expires_in.as_millis(),
                expiry_from_epoch: (SystemTime::now().duration_since(UNIX_EPOCH).unwrap()
                    + token.expires_in)
                    .as_millis(),
            }),
            _ => Err(MoosyncError::String("Invalid command reply".to_string())),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, uri))]
    pub fn get_lyrics(&self, uri: String) -> Result<String> {
        let res = self.send(Message::GetLyrics(uri))?;
        match res {
            MessageReply::GetLyrics(lyrics) => Ok(lyrics),
            _ => Err(MoosyncError::String("Invalid command reply".to_string())),
        }
    }

    #[tracing::instrument(level = "debug", skip(self, uri))]
    pub fn get_canvaz(&self, uri: String) -> Result<CanvazResponse> {
        let res = self.send(Message::GetCanvaz(uri))?;
        match res {
            MessageReply::GetCanvaz(canvaz) => Ok(canvaz),
            _ => Err(MoosyncError::String("Invalid command reply".to_string())),
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_device_id(&self) -> Arc<AsyncMutex<Option<String>>> {
        self.device_id.clone()
    }
}
