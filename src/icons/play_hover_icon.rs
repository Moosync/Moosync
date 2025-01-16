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

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn PlayHoverIcon() -> impl IntoView {
    view! {
        <svg
            width="23"
            height="26"
            viewBox="0 0 23 26"
            fill="none"
            style="cursor: pointer;"
            class="align-self-center"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M21.789 10.9007L3.71707 0.333515C2.24872 -0.524653 0 0.308125 0 2.4307V23.5599C0 25.4641 2.08957 26.6117 3.71707 25.6571L21.789 15.095C23.4011 14.1556 23.4062 11.8401 21.789 10.9007V10.9007Z"
                fill="white"
            ></path>
        </svg>
    }
}
