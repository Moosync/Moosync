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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use leptos::{component, prelude::*, view, IntoView};

use crate::{
    modals::{
        discover_extensions::DiscoverExtensionsModal, login_modal::LoginModal,
        new_playlist_modal::NewPlaylistModal, new_theme_modal::NewThemeModal,
        signout_modal::SignoutModal, song_from_url_modal::SongFromUrlModal,
    },
    store::modal_store::{ModalStore, Modals},
};

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn ModalManager() -> impl IntoView {
    let modal_store = expect_context::<RwSignal<ModalStore>>();

    view! {
        <div>
            {move || {
                let active_modal = modal_store.get().active_modal;
                tracing::debug!("Got active modal {:?}", active_modal);
                if active_modal.is_none() {
                    return ().into_any();
                }
                match active_modal.unwrap() {
                    Modals::LoginModal(key, name, account_id) => {
                        view! { <LoginModal key=key name=name account_id=account_id /> }.into_any()
                    }
                    Modals::DiscoverExtensions => view! { <DiscoverExtensionsModal /> }.into_any(),
                    Modals::NewPlaylistModal(initial_state, songs) => {
                        view! { <NewPlaylistModal initial_state=initial_state songs=songs /> }
                            .into_any()
                    }
                    Modals::SongFromUrlModal => view! { <SongFromUrlModal /> }.into_any(),
                    Modals::SignoutModal(key, name, account_id) => {
                        view! { <SignoutModal key=key name=name account_id=account_id /> }
                            .into_any()
                    }
                    Modals::ThemeModal(initial_state) => {
                        view! { <NewThemeModal initial_state=initial_state /> }.into_any()
                    }
                }
            }}

        </div>
    }
}
