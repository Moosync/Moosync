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
    fs,
    path::PathBuf,
    str::FromStr,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use hls_client::{config::ConfigBuilder, stream::HLSStream};
use rodio::Sink;
use stream_download::{storage::temp::TempStorageProvider, Settings, StreamDownload};
use tracing::{debug, error, info, trace};
use types::{
    errors::Result,
    ui::player_details::PlayerEvents,
};
use types::errors::error_helpers;



pub struct RodioPlayer {
    tx: Sender<RodioCommand>,
    events_rx: Arc<Mutex<Receiver<PlayerEvents>>>,
}

enum RodioCommand {
    SetSrc(String),
    Play,
    Pause,
    Stop,
    SetVolume(f32),
    Seek(u64),
}

impl RodioPlayer {
    #[tracing::instrument(level = "debug", skip())]
    pub fn new(cache_dir: PathBuf) -> Self {
        let (events_tx, events_rx) = channel::<PlayerEvents>();
        let cache_dir = cache_dir.join("rodio");
        if !cache_dir.exists() {
            fs::create_dir(cache_dir.clone()).unwrap();
        }
        let tx = Self::initialize(events_tx, cache_dir);
        Self {
            tx,
            events_rx: Arc::new(Mutex::new(events_rx)),
        }
    }

    async fn set_src(cache_dir: PathBuf, src: String, sink: &Arc<Sink>) -> Result<()> {
        if src.ends_with(".m3u8") || src.contains(".m3u8") {
            Self::handle_hls_stream(cache_dir.clone(), &src, sink).await?;
        } else if src.starts_with("http") {
            Self::handle_http_stream(cache_dir.clone(), &src, sink).await?;
        } else {
            Self::handle_local_file(&src, sink).await?;
        }

        Ok(())
    }

    async fn handle_hls_stream(cache_dir: PathBuf, src: &str, sink: &Arc<Sink>) -> Result<()> {
        let reader = StreamDownload::new::<HLSStream>(
            ConfigBuilder::new().url(src).map_err(error_helpers::to_playback_error)?.build().map_err(error_helpers::to_playback_error)?,
            TempStorageProvider::new_in(cache_dir.clone()),
            Settings::default(),
        )
        .await
        .map_err(error_helpers::to_playback_error)?;

        info!("HLS Stream content length {:?}", reader.content_length());
        trace!("Stream created");

        let decoder = rodio::Decoder::new(reader).map_err(error_helpers::to_playback_error)?;
        trace!("Decoder created");
        sink.append(decoder);
        trace!("Decoder appended");

        Ok(())
    }

    async fn handle_http_stream(cache_dir: PathBuf, src: &str, sink: &Arc<Sink>) -> Result<()> {
        trace!("Creating HTTP stream");

        match StreamDownload::new_http(
            src.parse().unwrap(),
            TempStorageProvider::new_in(cache_dir.clone()),
            Settings::default()
                .on_progress(move |_cl, state, _c| {
                    tracing::debug!("Progress: {}", state.current_position)
                })
                .prefetch_bytes(512),
        )
        .await
        {
            Ok(reader) => {
                trace!("Stream created");

                let decoder = rodio::Decoder::new(reader).map_err(error_helpers::to_playback_error)?;
                trace!("Decoder created");
                sink.append(decoder);
                trace!("Decoder appended");

                Ok(())
            }
            Err(e) => Err(e.to_string().into()),
        }
    }

    async fn handle_local_file(src: &str, sink: &Arc<Sink>) -> Result<()> {
        let path = PathBuf::from_str(src).unwrap();
        if path.exists() {
            let file = fs::File::open(path)?;
            let decoder = rodio::Decoder::try_from(file).map_err(error_helpers::to_playback_error)?;
            sink.append(decoder);

            trace!("Local file {} appended", src);

            return Ok(());
        }

        Err("Failed to read local file".into())
    }

    pub fn get_events_rx(&self) -> Arc<Mutex<Receiver<PlayerEvents>>> {
        self.events_rx.clone()
    }

    fn send_event(events_tx: Sender<PlayerEvents>, event: PlayerEvents) {
        events_tx.send(event).unwrap();
    }

    fn initialize(events_tx: Sender<PlayerEvents>, cache_dir: PathBuf) -> Sender<RodioCommand> {
        let (tx, rx) = channel::<RodioCommand>();
        let ret = tx.clone();

        thread::spawn(move || {
            let stream_handle = rodio::OutputStreamBuilder::open_default_stream().unwrap();
            let sink = Arc::new(rodio::Sink::connect_new(stream_handle.mixer()));

            let runtime = tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap();

            let events_tx = events_tx.clone();
            runtime.block_on(async move {
                let last_src = Arc::new(Mutex::new(None));
                while let Ok(command) = rx.recv() {
                    let sink = sink.clone();

                    match command {
                        RodioCommand::SetSrc(src) => {
                            let last_src = last_src.clone();
                            {
                                let mut last_src = last_src.lock().unwrap();
                                *last_src = Some(src.clone());
                            }

                            sink.clear();
                            Self::send_event(events_tx.clone(), PlayerEvents::TimeUpdate(0f64));
                            Self::send_event(events_tx.clone(), PlayerEvents::Loading);

                            if let Err(err) =
                                Self::set_src(cache_dir.clone(), src.clone(), &sink).await
                            {
                                error!("Failed to set src: {:?}", err);
                                Self::send_event(events_tx.clone(), PlayerEvents::Error(err))
                            } else {
                                debug!("Set src");
                                let src_clone = src.clone();

                                let events_tx = events_tx.clone();
                                let sink = sink.clone();

                                // Send ended event only if song hasn't changed yet
                                thread::spawn(move || {
                                    sink.sleep_until_end();
                                    let last_src = last_src.clone();
                                    let last_src = last_src.lock().unwrap();
                                    if let Some(last_src) = last_src.clone() {
                                        info!("last src={}, current src={}", last_src, src_clone);
                                        if last_src == src_clone {
                                            Self::send_event(
                                                events_tx.clone(),
                                                PlayerEvents::Ended,
                                            );
                                        }
                                    }
                                });
                            }
                        }
                        RodioCommand::Play => {
                            if !sink.empty() {
                                sink.play();
                                Self::send_event(events_tx.clone(), PlayerEvents::Play)
                            }
                        }
                        RodioCommand::Pause => {
                            if !sink.empty() {
                                sink.pause();
                                Self::send_event(events_tx.clone(), PlayerEvents::Pause)
                            }
                        }
                        RodioCommand::Stop => {
                            if !sink.empty() {
                                sink.stop();
                                sink.clear();
                                Self::send_event(events_tx.clone(), PlayerEvents::Pause)
                            }
                        }
                        RodioCommand::SetVolume(volume) => {
                            if !sink.empty() {
                                sink.set_volume(volume);
                            }
                        }
                        RodioCommand::Seek(pos) => {
                            if !sink.empty() {
                                if let Err(err) = sink.try_seek(Duration::from_secs(pos)) {
                                    error!("Failed to seek: {:?}", err)
                                } else {
                                    Self::send_event(
                                        events_tx.clone(),
                                        PlayerEvents::TimeUpdate(pos as f64),
                                    )
                                }
                            } else {
                                let last_src = last_src.clone();
                                let last_src = last_src.lock().unwrap();
                                if let Some(last_src) = last_src.clone() {
                                    tx.send(RodioCommand::SetSrc(last_src.clone())).unwrap();
                                    tx.send(RodioCommand::Seek(pos)).unwrap();
                                    tx.send(RodioCommand::Play).unwrap();
                                }
                            }
                        }
                    }
                }
            });
        });

        ret
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rodio_load(&self, src: String) -> Result<()> {
        info!("Loading src={}", src);
        self.tx.send(RodioCommand::SetSrc(src.clone())).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rodio_play(&self) -> Result<()> {
        self.tx.send(RodioCommand::Play).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rodio_pause(&self) -> Result<()> {
        self.tx.send(RodioCommand::Pause).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rodio_stop(&self) -> Result<()> {
        self.tx.send(RodioCommand::Stop).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rodio_seek(&self, pos: f64) -> Result<()> {
        self.tx
            .send(RodioCommand::Seek(pos.abs().round() as u64))
            .unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rodio_set_volume(&self, volume: f32) -> Result<()> {
        self.tx.send(RodioCommand::SetVolume(volume)).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub async fn rodio_get_volume(&self) -> Result<f32> {
        Ok(0f32)
    }
}
