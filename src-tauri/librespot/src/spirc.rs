use std::{
    sync::{
        mpsc::{self, Receiver, Sender},
        Arc, Mutex,
    },
    thread::{self, JoinHandle},
    time::{SystemTime, UNIX_EPOCH},
};

use librespot::{
    connect::{
        config::ConnectConfig,
        spirc::{Spirc, SpircLoadCommand},
    },
    core::{cache::Cache, token::Token, Session, SpotifyId},
    discovery::Credentials,
    playback::{
        config::PlayerConfig,
        player::{PlayerEvent, PlayerEventChannel},
    },
    protocol::spirc::TrackRef,
};
use serde::{Deserialize, Serialize};
use tokio::runtime::Builder;

use crate::player::{create_session, get_canvas, get_lyrics, new_player};
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
    device_id: Arc<Mutex<Option<String>>>,
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
        let device_id_mutex = Arc::new(Mutex::new(None));
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
                let session = create_session(cache_config).clone();

                let device_id = session.device_id().to_string();

                let (player, mixer) =
                    new_player(backend, session.clone(), player_config.clone(), volume_ctrl);

                let events_channel = player.get_player_event_channel();

                tracing::info!("Creating spirc {:?}", credentials);
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
                        let commands_thread = SpircWrapper::listen_commands(
                            rx,
                            channel_close_tx,
                            spirc,
                            session.clone(),
                        );
                        let events_thread =
                            SpircWrapper::listen_events(player_events_tx, events_channel);

                        {
                            let mut device_id_mutex = device_id_mutex.lock().unwrap();
                            *device_id_mutex = Some(device_id);
                        }

                        spirc_task.await;

                        commands_thread.join().unwrap();
                        events_thread.join().unwrap();
                    }
                    Err(e) => {
                        tracing::info!("Error creating spirc: {:?}", e);
                    }
                }
            });
        });

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
                let mut device_id = device_id.lock().unwrap();
                device_id.take();
            }
        });
    }

    #[tracing::instrument(level = "trace", skip(tx, events_channel))]
    fn listen_events(
        tx: Sender<PlayerEvent>,
        mut events_channel: PlayerEventChannel,
    ) -> JoinHandle<()> {
        thread::spawn(move || loop {
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
        })
    }

    #[tracing::instrument(level = "trace", skip(message, tx, spirc, session))]
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
                    .map_err(MoosyncError::Librespot);

                tx.send(res).unwrap();
            }

            Message::Pause => {
                let res = (spirc)
                    .pause()
                    .map(|_| MessageReply::None)
                    .map_err(MoosyncError::Librespot);

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
                        .map_err(MoosyncError::Librespot)
                });

                tx.send(data).unwrap();
            }
            Message::Seek(pos) => {
                let res = spirc
                    .set_position_ms(pos)
                    .map(|_| MessageReply::None)
                    .map_err(MoosyncError::Librespot);
                tx.send(res).unwrap();
            }
            Message::Load(uri, autoplay) => {
                let track_id = SpotifyId::from_uri(uri.as_str()).map_err(MoosyncError::Librespot);
                match track_id {
                    Err(e) => {
                        tx.send(Err(e)).unwrap();
                    }
                    Ok(track_id) => {
                        let mut track_ref = TrackRef::new();
                        track_ref.set_gid(Vec::from(track_id.to_raw()));
                        let command = SpircLoadCommand {
                            context_uri: uri,
                            start_playing: autoplay,
                            shuffle: false,
                            repeat: false,
                            playing_track_index: 0,
                            tracks: vec![track_ref],
                        };

                        let res = spirc
                            .load(command)
                            .map(|_| MessageReply::None)
                            .map_err(MoosyncError::Librespot);

                        tx.send(res).unwrap();
                    }
                }
            }
            Message::Volume(vol) => {
                let res = spirc
                    .set_volume(vol)
                    .map(|_| MessageReply::None)
                    .map_err(MoosyncError::Librespot);
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

    #[tracing::instrument(level = "trace", skip(rx, spirc, session))]
    pub fn listen_commands(
        rx: Receiver<(Message, Sender<Result<MessageReply>>)>,
        channel_close_tx: Sender<()>,
        mut spirc: Spirc,
        mut session: Session,
    ) -> JoinHandle<()> {
        thread::spawn(move || loop {
            if let Ok((message, tx)) = rx.recv() {
                if message == Message::Close {
                    spirc.shutdown().unwrap();
                    session.shutdown();
                    tx.send(Ok(MessageReply::None)).unwrap();
                    channel_close_tx.send(()).unwrap();
                    return;
                }

                Self::handle_command(message.clone(), tx, &mut spirc, &mut session);
                tracing::info!("Finished handling: {:?}", message);
            } else {
                channel_close_tx.send(()).unwrap();
                return;
            }
        })
    }

    #[tracing::instrument(level = "trace", skip(self, command))]
    pub fn send(&self, command: Message) -> Result<MessageReply> {
        let (tx, rx) = mpsc::channel::<Result<MessageReply>>();
        self.tx
            .send((command, tx))
            .map_err(|e| MoosyncError::String(e.to_string()))?;

        rx.recv().unwrap()
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn librespot_close(&self) -> Result<()> {
        self.send(Message::Close)?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn librespot_play(&self) -> Result<()> {
        self.send(Message::Play)?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn librespot_pause(&self) -> Result<()> {
        self.send(Message::Pause)?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, pos))]
    pub fn librespot_seek(&self, pos: u32) -> Result<()> {
        self.send(Message::Seek(pos))?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, volume))]
    pub fn librespot_volume(&self, volume: u16) -> Result<()> {
        self.send(Message::Volume(volume))?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, uri, autoplay))]
    pub fn librespot_load(&self, uri: String, autoplay: bool) -> Result<()> {
        self.send(Message::Load(uri, autoplay))?;
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self, scopes))]
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

    #[tracing::instrument(level = "trace", skip(self, uri))]
    pub fn get_lyrics(&self, uri: String) -> Result<String> {
        let res = self.send(Message::GetLyrics(uri))?;
        match res {
            MessageReply::GetLyrics(lyrics) => Ok(lyrics),
            _ => Err(MoosyncError::String("Invalid command reply".to_string())),
        }
    }

    #[tracing::instrument(level = "trace", skip(self, uri))]
    pub fn get_canvaz(&self, uri: String) -> Result<CanvazResponse> {
        let res = self.send(Message::GetCanvaz(uri))?;
        match res {
            MessageReply::GetCanvaz(canvaz) => Ok(canvaz),
            _ => Err(MoosyncError::String("Invalid command reply".to_string())),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_device_id(&self) -> Arc<Mutex<Option<String>>> {
        self.device_id.clone()
    }
}
