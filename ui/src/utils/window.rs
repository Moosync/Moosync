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

use leptos::{
    prelude::{expect_context, RwSignal, Update},
    task::spawn_local,
};

use crate::store::modal_store::{ModalStore, Modals};

use super::invoke::fetch_update;

pub fn check_for_updates() {
    tracing::info!("Checking for updates...");
    let modal_store = expect_context::<RwSignal<ModalStore>>();
    spawn_local(async move {
        let update = fetch_update().await;
        match update {
            Ok(Some(update)) => {
                modal_store.update(|m| m.set_active_modal(Modals::UpdateModal(update)));
            }
            Err(e) => tracing::error!("Failed to fetch update: {:?}", e),
            _ => {
                tracing::info!("No update found")
            }
        }
    });
}
