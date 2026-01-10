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
pub fn QueueIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="18"
            height="18"
            viewBox="0 0 24 21"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >

            <title>Queue</title>
            <path
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
                d="M6 0.666504H24V2.6665H6V0.666504ZM6 6.6665H24V8.6665H6V6.6665ZM11 12.6665H24V14.6665H11V12.6665ZM6 18.6665H24V20.6665H6V18.6665ZM0 8.6665L7 13.6665L0 18.6665V8.6665Z"
            ></path>
        </svg>
    }
}
