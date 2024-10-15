use std::{
    fs,
    io::BufReader,
    path::PathBuf,
    str::FromStr,
    sync::{
        atomic::AtomicBool,
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use rodio::{Decoder, OutputStream, Sink};
use stream_download::{storage::temp::TempStorageProvider, Settings, StreamDownload};
use tracing::{error, info, trace};
use types::{errors::Result, ui::player_details::PlayerEvents};

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
    #[tracing::instrument(level = "trace", skip())]
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
        if src.starts_with("http") {
            trace!("Creating stream");
            let reader = StreamDownload::new_http(
                src.parse().unwrap(),
                TempStorageProvider::new_in(cache_dir),
                Settings::default(),
            )
            .await?;
            trace!("stream created");

            let decoder = rodio::Decoder::new(reader)?;
            trace!("decoder created");
            sink.append(decoder);
            trace!("decoder appended");

            Ok(())
        } else {
            let path = PathBuf::from_str(src.as_str()).unwrap();
            if path.exists() {
                let file = fs::File::open(path)?;
                let reader = BufReader::new(file);
                let decoder = Decoder::new(reader)?;
                sink.append(decoder);
                return Ok(());
            }

            Err("Failed to read src".into())
        }
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
            let (_stream, stream_handle) = OutputStream::try_default().unwrap();
            let sink = Arc::new(rodio::Sink::try_new(&stream_handle).unwrap());

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
                                info!("Set src");
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

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_load(&self, src: String) -> Result<()> {
        info!("Loading src={}", src);
        self.tx.send(RodioCommand::SetSrc(src.clone())).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_play(&self) -> Result<()> {
        self.tx.send(RodioCommand::Play).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_pause(&self) -> Result<()> {
        self.tx.send(RodioCommand::Pause).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_stop(&self) -> Result<()> {
        self.tx.send(RodioCommand::Stop).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_seek(&self, pos: f64) -> Result<()> {
        self.tx
            .send(RodioCommand::Seek(pos.abs().round() as u64))
            .unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_set_volume(&self, volume: f32) -> Result<()> {
        self.tx.send(RodioCommand::SetVolume(volume)).unwrap();
        Ok(())
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub async fn rodio_get_volume(&self) -> Result<f32> {
        Ok(0f32)
    }
}
