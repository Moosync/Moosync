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

#[tracing::instrument(level = "debug", skip(disabled))]
#[component]
pub fn PrevTrackIcon<T>(#[prop()] disabled: T) -> impl IntoView
where
    T: Get<Value = bool> + 'static + Send,
{
    view! {
        <svg
            class="button-grow"
            width="24"
            height="16"
            viewBox="0 0 24 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M14.0462 14.0561V10.5709L22.0541 14.6555C22.2627 14.7619 22.5116 14.7522 22.7113 14.6299C22.911 14.5076 23.0327 14.2903 23.0327 14.0561V1.94394C23.0327 1.70977 22.911 1.49244 22.7113 1.37013C22.5116 1.24782 22.2627 1.23811 22.0541 1.34451L14.0462 5.42889V1.94394C14.0462 1.70977 13.9244 1.49244 13.7247 1.37013C13.5251 1.24782 13.2761 1.23811 13.0675 1.34451L1.19426 7.40038C0.968947 7.5153 0.827106 7.74687 0.827102 7.9998C0.827099 8.25273 0.968934 8.48431 1.19425 8.59923L13.0675 14.6555C13.2761 14.7619 13.525 14.7522 13.7247 14.6299C13.9244 14.5076 14.0462 14.2903 14.0462 14.0561Z"
                stroke=move || if disabled.get() { "var(--textSecondary)" } else { "var(--accent)" }
                stroke-width="1.3458"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
        </svg>
    }
}
