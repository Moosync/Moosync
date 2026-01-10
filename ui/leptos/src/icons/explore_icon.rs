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

use leptos::{IntoView, component, prelude::*, view};

#[tracing::instrument(level = "debug", skip(active))]
#[component]
pub fn ExploreIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="18"
            height="18"
            viewBox="0 0 18 18"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M11.75 11.7501C9.4028 14.0973 5.5972 14.0973 3.24998 11.7501C0.902752 9.40285 0.902752 5.59725 3.24998 3.25002C5.5972 0.902801 9.4028 0.902801 11.75 3.25002C14.0972 5.59725 14.0972 9.40285 11.75 11.7501ZM11.75 11.7501L16.5 16.5"
                stroke=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
                stroke-width="2"
                stroke-linejoin="round"
            ></path>
        </svg>
    }
}
