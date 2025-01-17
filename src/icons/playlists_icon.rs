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

use leptos::{component, prelude::*, view, IntoView};

#[tracing::instrument(level = "trace", skip(active))]
#[component]
pub fn PlaylistsIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="19"
            height="12"
            viewBox="0 0 19 12"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <title>Playlists</title>
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M1 0H18C18.5523 0 19 0.447715 19 1V11C19 11.5523 18.5523 12 18 12H1C0.447715 12 0 11.5523 0 11V1C0 0.447715 0.447715 0 1 0ZM5 8C6.10457 8 7 7.10457 7 6C7 4.89543 6.10457 4 5 4C3.89543 4 3 4.89543 3 6C3 7.10457 3.89543 8 5 8ZM14 8C15.1046 8 16 7.10457 16 6C16 4.89543 15.1046 4 14 4C12.8954 4 12 4.89543 12 6C12 7.10457 12.8954 8 14 8Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            ></path>
        </svg>
    }
}
