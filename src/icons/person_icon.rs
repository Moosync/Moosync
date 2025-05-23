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

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn PersonIcon() -> impl IntoView {
    view! {
        <svg
            class="button-grow"
            width="17"
            height="17"
            viewBox="0 0 17 17"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M1.41667 17C1.41667 17 0 17 0 15.5833C0 14.1667 1.41667 9.91667 8.5 9.91667C15.5833 9.91667 17 14.1667 17 15.5833C17 17 15.5833 17 15.5833 17H1.41667ZM8.5 8.5C9.62717 8.5 10.7082 8.05223 11.5052 7.2552C12.3022 6.45817 12.75 5.37717 12.75 4.25C12.75 3.12283 12.3022 2.04183 11.5052 1.2448C10.7082 0.447767 9.62717 0 8.5 0C7.37283 0 6.29183 0.447767 5.4948 1.2448C4.69777 2.04183 4.25 3.12283 4.25 4.25C4.25 5.37717 4.69777 6.45817 5.4948 7.2552C6.29183 8.05223 7.37283 8.5 8.5 8.5V8.5Z"
                fill="var(--textPrimary)"
            ></path>
        </svg>
    }
}
