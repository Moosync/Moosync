use std::rc::Rc;

use leptos::{html::Div, NodeRef};
use tokio::sync::oneshot::Sender as OneShotSender;
use types::{
    errors::errors::Result,
    songs::{Song, SongType},
    ui::player_details::PlayerEvents,
};

pub trait GenericPlayer: std::fmt::Debug {
    fn initialize(&self, element: NodeRef<Div>);
    fn key(&self) -> String;
    fn load(&self, src: String, resolver: OneShotSender<()>);
    fn stop(&mut self) -> Result<()>;
    fn play(&self) -> Result<()>;
    fn pause(&self) -> Result<()>;
    fn seek(&self, pos: f64) -> Result<()>;
    fn provides(&self) -> &[SongType];
    fn can_play(&self, song: &Song) -> bool;
    fn set_volume(&self, volume: f64) -> Result<()>;
    fn get_volume(&self) -> Result<f64>;
    fn add_listeners(&mut self, state_setter: Rc<Box<dyn Fn(PlayerEvents)>>);
}
