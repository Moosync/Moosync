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

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn PrevIcon() -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            width="10"
            height="16"
            viewBox="0 0 10 16"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M9.24257 2.34317L7.82836 0.928955L0.757324 8.00001L7.82839 15.0711L9.24261 13.6569L3.58574 8L9.24257 2.34317Z"
                fill="var(--textPrimary)"
            ></path>
        </svg>
    }
}
