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

#[tracing::instrument(level = "trace", skip(title))]
#[component]
pub fn AddToQueueIcon(#[prop()] title: String) -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            width="25"
            height="25"
            viewBox="0 0 25 25"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <title>{title}</title>
            <path
                d="M22.1667 0.416672H7.66667C6.33387 0.416672 5.25 1.50055 5.25 2.83334V17.3333C5.25 18.6661 6.33387 19.75 7.66667 19.75H22.1667C23.4995 19.75 24.5833 18.6661 24.5833 17.3333V2.83334C24.5833 1.50055 23.4995 0.416672 22.1667 0.416672ZM7.66667 17.3333V2.83334H22.1667L22.1691 17.3333H7.66667Z"
                fill="var(--accent)"
            ></path>
            <path
                d="M2.83366 7.66667H0.416992V22.1667C0.416992 23.4995 1.50087 24.5833 2.83366 24.5833H17.3337V22.1667H2.83366V7.66667ZM16.1253 5.25H13.7087V8.875H10.0837V11.2917H13.7087V14.9167H16.1253V11.2917H19.7503V8.875H16.1253V5.25Z"
                fill="var(--accent)"
            ></path>
        </svg>
    }
}
