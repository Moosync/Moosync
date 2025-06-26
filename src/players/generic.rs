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

use std::rc::Rc;

use dyn_clone::DynClone;
use leptos::{html::Div, prelude::NodeRef};
use tokio::sync::oneshot::Sender as OneShotSender;
use types::{
    errors::Result,
    songs::{Song, SongType},
    ui::player_details::PlayerEvents,
};

pub type PlayerEventsSender = Rc<Box<dyn Fn(String, PlayerEvents)>>;

pub trait GenericPlayer: std::fmt::Debug + DynClone {
    fn initialize(&self, element: NodeRef<Div>);
    fn key(&self) -> String;
    fn load(&self, src: String, autoplay: bool, resolver: OneShotSender<()>);
    fn stop(&mut self) -> Result<()>;
    fn play(&self) -> Result<()>;
    fn pause(&self) -> Result<()>;
    fn seek(&self, pos: f64) -> Result<()>;
    fn provides(&self) -> &[SongType];
    fn can_play(&self, song: &Song) -> bool;
    fn set_volume(&self, volume: f64) -> Result<()>;
    fn get_volume(&self) -> Result<f64>;
    fn add_listeners(&mut self, state_setter: PlayerEventsSender);
}

dyn_clone::clone_trait_object!(GenericPlayer);
