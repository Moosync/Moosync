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

#[tracing::instrument(level = "trace", skip(class))]
#[component]
pub fn FavPlaylistIcon(#[prop()] class: impl Into<String>) -> impl IntoView {
    view! {
        <svg
            class=class.into()
            viewBox="0 0 337 337"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <rect width="337" height="337" fill="#E98181" />
            <path
                d="M213.28 84C202.693 84 192.987 87.2926 184.431 93.7867C176.229 100.013 170.768 107.942 167.553 113.709C164.338 107.942 158.877 100.013 150.674 93.7867C142.118 87.2926 132.412 84 121.825 84C92.2801 84 70 107.718 70 139.169C70 173.148 97.796 196.396 139.875 231.59C147.021 237.567 155.121 244.341 163.539 251.567C164.649 252.52 166.074 253.045 167.553 253.045C169.031 253.045 170.456 252.52 171.566 251.567C179.985 244.341 188.084 237.567 195.234 231.586C237.309 196.396 265.105 173.148 265.105 139.169C265.105 107.718 242.825 84 213.28 84Z"
                fill="white"
            />
        </svg>
    }
}
