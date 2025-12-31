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

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn SidebarToggleIcon() -> impl IntoView {
    view! {
        <svg
            width="19"
            height="17"
            viewBox="0 0 19 17"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >

            <path
                d="M18.0666 7.29707H1.83862C1.32313 7.29707 0.905273 7.71493 0.905273 8.23042C0.905273 8.7459 1.32313 9.16376 1.83862 9.16376H18.0666C18.5821 9.16376 18.9999 8.7459 18.9999 8.23042C18.9999 7.71493 18.5821 7.29707 18.0666 7.29707Z"
                fill="var(--textPrimary)"
            />
            <path
                d="M18.0666 0H1.83862C1.32313 0 0.905273 0.417858 0.905273 0.933344C0.905273 1.44883 1.32313 1.86669 1.83862 1.86669H18.0666C18.5821 1.86669 18.9999 1.44883 18.9999 0.933344C18.9999 0.417858 18.5821 0 18.0666 0Z"
                fill="var(--textPrimary)"
            />
            <path
                d="M18.0666 14.5941H1.83862C1.32313 14.5941 0.905273 15.0119 0.905273 15.5274C0.905273 16.0429 1.32313 16.4608 1.83862 16.4608H18.0666C18.5821 16.4608 18.9999 16.0429 18.9999 15.5274C18.9999 15.0119 18.5821 14.5941 18.0666 14.5941Z"
                fill="var(--textPrimary)"
            />
        </svg>
    }
}
