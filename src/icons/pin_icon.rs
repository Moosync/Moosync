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

#[tracing::instrument(level = "debug", skip(filled))]
#[component]
pub fn PinIcon<T>(#[prop()] filled: T) -> impl IntoView
where
    T: Get<Value = bool> + 'static + Send,
{
    view! {
        <svg
            width="58"
            height="58"
            viewBox="0 0 58 58"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            class="pin-icon button-grow"
        >
            <path
                d="M34.9254 2.93313L34.9286 2.93313C34.9327 2.93312 34.9367 2.93392 34.9405 2.93549C34.9443 2.93702 34.9477 2.93926 34.9505 2.94207C34.9506 2.94213 34.9506 2.94219 34.9507 2.94225L55.0581 23.0496C55.0639 23.0555 55.0672 23.0634 55.0672 23.0717C55.0672 23.08 55.0639 23.088 55.0581 23.0938C53.6172 24.5346 51.8125 24.8966 50.3666 24.8966C49.7807 24.8966 49.2518 24.8368 48.8292 24.7658L47.8125 24.595L47.0836 25.3239L34.3517 38.0558L33.5526 38.8549L33.8248 39.9518C34.1318 41.1885 34.3311 42.4495 34.4205 43.7207C34.5966 46.4181 34.2382 49.6579 32.0769 51.8193C32.0711 51.8252 32.0631 51.8285 32.0548 51.8285C32.0466 51.8285 32.0386 51.8252 32.0327 51.8193L32.0327 51.8193L20.5399 40.3305L19.1257 38.9168L17.7117 40.3308L4.78485 53.2577C4.73211 53.3104 4.62958 53.3981 4.48777 53.5104C4.59904 53.3701 4.68568 53.2691 4.73745 53.2175L4.74109 53.2139L17.668 40.287L19.0819 38.8731L17.6682 37.4589L6.17947 25.966L6.17943 25.966C6.17357 25.9601 6.17028 25.9522 6.17028 25.9439C6.17028 25.9356 6.17357 25.9277 6.17943 25.9218C8.34196 23.7595 11.5839 23.3983 14.2771 23.5782C15.5485 23.6675 16.8098 23.8668 18.0468 24.1739L19.1436 24.4462L19.9427 23.6473L32.6746 10.9194L33.4071 10.1872L33.2318 9.16642C33.1445 8.65851 33.0998 8.14421 33.0981 7.62886C33.0987 6.18941 33.4603 4.38638 34.9036 2.94202C34.9094 2.93633 34.9173 2.93313 34.9254 2.93313Z"
                stroke="white"
                fill=move || if filled.get() { "white" } else { "none" }
                stroke-width="4"
            ></path>
        </svg>
    }
}
