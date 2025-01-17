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

#[tracing::instrument(level = "trace", skip(play))]
#[component]
pub fn PlayIcon<T>(#[prop()] play: T) -> impl IntoView
where
    T: Get<Value = bool> + 'static + Send,
{
    view! {
        <svg class="button-grow" viewBox="0 0 41 42" fill="none" xmlns="http://www.w3.org/2000/svg">
            // <transition
            // name="custom-fade"
            // enter-active-class="animate__animated animate__fadeIn"
            // leave-active-class="animate__animated animate__fadeOut animate__faster"
            // >

            {move || {
                if play.get() {
                    view! {
                        <g>
                            <path d="M16 13L16 28" stroke="var(--accent)" stroke-width="3"></path>
                            <path d="M25 13L25 28" stroke="var(--accent)" stroke-width="3"></path>
                        </g>
                    }
                        .into_any()
                } else {
                    view! {
                        <path
                            d="M16.2775 14.3421C16.0689 14.2378 15.8212 14.249 15.6228 14.3716C15.4244 14.4942 15.3037 14.7108 15.3037 14.944V27.0561C15.3037 27.2893 15.4244 27.5059 15.6228 27.6285C15.8212 27.7511 16.0689 27.7623 16.2775 27.658L28.3897 21.6017C28.6176 21.4877 28.7616 21.2547 28.7616 20.9998C28.7616 20.745 28.6176 20.512 28.3896 20.398L16.2775 14.3421Z"
                            fill="var(--accent)"
                            stroke="var(--accent)"
                            stroke-width="1.3458"
                            stroke-linecap="round"
                            stroke-linejoin="round"
                        ></path>
                    }
                        .into_any()
                }
            }}

            // </transition>
            <circle
                cx="20.6869"
                cy="21"
                r="19.514"
                stroke="var(--accent)"
                stroke-width="1.3458"
            ></circle>
        </svg>
    }
}
