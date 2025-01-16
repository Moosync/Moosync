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
use types::ui::extensions::PackageNameArgs;
use wasm_bindgen_futures::spawn_local;

use crate::{
    icons::{spotify_icon::SpotifyIcon, youtube_icon::YoutubeIcon},
    utils::invoke::get_extension_icon,
};

#[tracing::instrument(level = "trace", skip(extension))]
#[component]
pub fn ProviderIcon(#[prop()] extension: String) -> impl IntoView {
    let provider_icon = RwSignal::new(String::new());
    let extension_clone = extension.clone();
    spawn_local(async move {
        if !extension_clone.is_empty()
            && extension_clone != "youtube"
            && extension_clone != "spotify"
        {
            let res = get_extension_icon(PackageNameArgs {
                package_name: extension_clone,
            })
            .await;

            if let Ok(res) = res {
                provider_icon.set(res);
            } else {
                tracing::error!("Failed to get provider icon {:?}", res);
            }
        }
    });
    view! {
        <div class="d-flex provider-icon">
            {move || {
                let extension = extension.as_str();
                if extension == "youtube" {
                    view! { <YoutubeIcon /> }.into_any()
                } else if extension == "spotify" {
                    view! { <SpotifyIcon /> }.into_any()
                } else {
                    view! {
                        <img
                            style:display=if provider_icon.get().is_empty() {
                                "none"
                            } else {
                                "block"
                            }
                            referrerpolicy="no-referrer"
                            src=move || provider_icon.get()
                        />
                    }
                        .into_any()
                }
            }}
        </div>
    }
}
