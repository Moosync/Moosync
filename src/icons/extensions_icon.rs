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
pub fn ExtensionsIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="18"
            height="19"
            viewBox="0 0 18 19"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M10 1C10 0.447715 10.4477 0 11 0H17C17.5523 0 18 0.447715 18 1V7C18 7.55228 17.5523 8 17 8H11C10.4477 8 10 7.55228 10 7V1ZM11.5 2.5C11.5 1.94772 11.9477 1.5 12.5 1.5H15.5C16.0523 1.5 16.5 1.94772 16.5 2.5V5.5C16.5 6.05228 16.0523 6.5 15.5 6.5H12.5C11.9477 6.5 11.5 6.05228 11.5 5.5V2.5Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            />
            <path
                fill-rule="evenodd"
                clip-rule="evenodd"
                d="M13 18.5C13.5523 18.5 14 18.0523 14 17.5V11.5C14 10.9477 13.5523 10.5 13 10.5H9C8.44772 10.5 8 10.0523 8 9.5V5.5C8 4.94772 7.55228 4.5 7 4.5H1C0.447715 4.5 0 4.94772 0 5.5V17.5C0 18.0523 0.447715 18.5 1 18.5H13ZM6.5 7C6.5 6.44772 6.05228 6 5.5 6H2.5C1.94772 6 1.5 6.44772 1.5 7V10C1.5 10.5523 1.94772 11 2.5 11H5.5C6.05228 11 6.5 10.5523 6.5 10V7ZM2.5 17C1.94772 17 1.5 16.5523 1.5 16V13C1.5 12.4477 1.94772 12 2.5 12H5.5C6.05228 12 6.5 12.4477 6.5 13V16C6.5 16.5523 6.05228 17 5.5 17H2.5ZM8.5 17C7.94772 17 7.5 16.5523 7.5 16V13C7.5 12.4477 7.94772 12 8.5 12H11.5C12.0523 12 12.5 12.4477 12.5 13V16C12.5 16.5523 12.0523 17 11.5 17H8.5Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            />
        </svg>
    }
}
