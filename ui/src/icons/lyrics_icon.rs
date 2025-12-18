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

#[tracing::instrument(level = "debug")]
#[component]
pub fn LyricsIcon() -> impl IntoView {
    view! {
        <svg
            width="20"
            height="19"
            viewBox="0 0 20 19"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M0 18.077V3.116C0 2.668 0.157333 2.28667 0.472 1.972C0.786667 1.65733 1.16767 1.5 1.615 1.5H11.385C11.8317 1.5 12.2127 1.65733 12.528 1.972C12.8427 2.28667 13 2.668 13 3.116V3.367C12.8047 3.46033 12.627 3.567 12.467 3.687C12.307 3.80633 12.1513 3.93633 12 4.077V3.116C12 2.936 11.9423 2.78833 11.827 2.673C11.7117 2.55767 11.5637 2.5 11.383 2.5H1.615C1.43567 2.5 1.28833 2.55767 1.173 2.673C1.05767 2.78833 1 2.936 1 3.116V15.656L2.156 14.5H11.385C11.5643 14.5 11.7117 14.4423 11.827 14.327C11.9423 14.2117 12 14.0643 12 13.885V10.923C12.1513 11.0643 12.307 11.1943 12.467 11.313C12.627 11.433 12.8047 11.5397 13 11.633V13.885C13 14.3317 12.8427 14.7127 12.528 15.028C12.2133 15.3427 11.8323 15.5 11.385 15.5H2.577L0 18.077ZM3.5 12H6.5V11H3.5V12ZM15.039 10C14.3463 10 13.7563 9.75667 13.269 9.27C12.7817 8.78333 12.5383 8.19333 12.539 7.5C12.5397 6.80667 12.783 6.21667 13.269 5.73C13.755 5.24333 14.345 5 15.039 5C15.363 5 15.6373 5.04567 15.862 5.137C16.0867 5.22833 16.3123 5.382 16.539 5.598V0H19.539V1H17.539V7.5C17.539 8.19267 17.2953 8.78267 16.808 9.27C16.3207 9.75667 15.7317 10 15.039 10ZM3.5 9H9.5V8H3.5V9ZM3.5 6H9.5V5H3.5V6Z"
                fill="var(--textPrimary)"
            />
        </svg>
    }
}
