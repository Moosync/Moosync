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

#[tracing::instrument(level = "debug", skip(title))]
#[component]
pub fn PlainPlayIcon(#[prop()] title: String) -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            style="cursor: pointer;"
            width="22"
            height="22"
            viewBox="0 0 20 20"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <title>{title}</title>
            <path
                d="M1.2236 0.552783C1.06861 0.475289 0.884538 0.483573 0.737132 0.574676C0.589725 0.66578 0.5 0.826712 0.5 1V19C0.5 19.1733 0.589727 19.3342 0.737137 19.4253C0.884547 19.5164 1.06862 19.5247 1.22361 19.4472L19.2236 10.4469C19.393 10.3622 19.5 10.1891 19.5 9.99969C19.5 9.8103 19.393 9.63717 19.2236 9.55247L1.2236 0.552783Z"
                fill="var(--accent)"
                stroke="var(--accent)"
                stroke-linecap="round"
                stroke-linejoin="round"
            ></path>
        </svg>
    }
}
