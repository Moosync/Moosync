use tokio::sync::{mpsc::Sender, oneshot::Sender as OneShotSender};
use types::{errors::errors::Result, songs::SongType, ui::player_details::PlayerEvents};

pub trait GenericPlayer: std::fmt::Debug {
    fn initialize(&self);
    fn load(&self, src: String, resolver: OneShotSender<()>);
    fn play(&self) -> Result<()>;
    fn pause(&self) -> Result<()>;
    fn seek(&self, pos: f64) -> Result<()>;
    fn provides(&self) -> &[SongType];
    fn set_volume(&self, volume: f64) -> Result<()>;
    fn get_volume(&self) -> Result<f64>;
    fn add_listeners(&mut self, tx: Sender<PlayerEvents>);
}
