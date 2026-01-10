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

use leptos::{IntoView, component, prelude::*, task::spawn_local, view};
use types::ui::updater::UpdateMetadata;

use crate::{
    modals::common::GenericModal, store::modal_store::ModalStore, utils::invoke::install_update,
};

#[tracing::instrument(level = "debug", skip(metadata))]
#[component]
pub fn UpdateModal(#[prop()] metadata: UpdateMetadata) -> impl IntoView {
    let modal_store: RwSignal<ModalStore> = expect_context();
    let close_modal = move |_| modal_store.update(|m| m.clear_active_modal());

    let trigger_update = move |ev| {
        spawn_local(async move {
            close_modal(ev);
            if let Err(e) = install_update().await {
                tracing::error!("Failed to install update: {:?}", e);
            }
        });
    };

    tracing::info!("Got update {:?}", metadata);

    view! {
        <GenericModal size=move || "modal-lg".into()>
            <div class="container-fluid p-0 mt-4">
                <div class="row no-gutters d-flex">
                    <div class="col">
                        <h4>An update is available. Do you want to update?</h4>
                    </div>
                </div>
                <div class="row row-cols-auto mt-3 mr-4">
                    <button
                        on:click=close_modal
                        class="btn btn-secondary cancel-button ml-auto"
                        type="button"
                    >
                        Cancel
                    </button>
                    <button
                        on:click=trigger_update
                        class="btn btn-secondary confirm-button ml-3"
                        type="button"
                    >
                        Confirm
                    </button>
                </div>
            </div>
        </GenericModal>
    }
}
