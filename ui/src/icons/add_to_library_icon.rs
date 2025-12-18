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

#[tracing::instrument(level = "debug", skip(title))]
#[component]
pub fn AddToLibraryIcon(#[prop()] title: String) -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            width="20"
            height="24"
            viewBox="0 0 20 24"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <title>{title}</title>
            <path
                d="M18.75 8.25H13.75V0.75H6.25V8.25H1.25L10 18.25L18.75 8.25ZM0 20.75H20V23.25H0V20.75Z"
                fill="var(--accent)"
            ></path>
        </svg>
    }
}
