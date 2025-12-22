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

#[tracing::instrument(level = "debug", skip(color))]
#[component]
pub fn CrossIcon(#[prop(default = "var(--textPrimary)".into())] color: String) -> impl IntoView {
    view! {
        <svg viewBox="0 0 8 8" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
                d="M7.25 0.75L0.75 7.25M0.75 0.75L7.25 7.25"
                stroke=color
                stroke-width="1.5"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
        </svg>
    }
}
