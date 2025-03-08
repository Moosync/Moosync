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

use types::{
    songs::Song,
    ui::{themes::ThemeModalState, updater::UpdateMetadata},
};

use crate::modals::new_playlist_modal::PlaylistModalState;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Modals {
    LoginModal(String, String, String),
    SignoutModal(String, String, String),
    DiscoverExtensions,
    NewPlaylistModal(PlaylistModalState, Option<Vec<Song>>),
    SongFromUrlModal,
    ThemeModal(ThemeModalState),
    UpdateModal(UpdateMetadata),
}

#[derive(Clone, Default)]
pub struct ModalStore {
    pub active_modal: Option<Modals>,
    pub on_modal_close: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
}

impl ModalStore {
    #[tracing::instrument(level = "debug", skip(self, modal))]
    pub fn set_active_modal(&mut self, modal: Modals) {
        self.clear_active_modal();
        self.active_modal = Some(modal);
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn clear_active_modal(&mut self) {
        self.active_modal = None;
        if let Some(cb) = self.on_modal_close.take() {
            cb();
        }
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub fn get_active_modal(&self) -> Option<Modals> {
        self.active_modal.clone()
    }

    #[tracing::instrument(level = "debug", skip(self, cb))]
    pub fn on_modal_close<T>(&mut self, cb: T)
    where
        T: Fn() + 'static + Send + Sync,
    {
        self.on_modal_close = Some(Arc::new(Box::new(cb)));
    }
}
