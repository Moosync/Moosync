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

#[tracing::instrument(level = "debug", skip(playing))]
#[component]
pub fn AnimatedEqualizerIcon(#[prop()] playing: bool) -> impl IntoView {
    view! {
        <svg
            id="eF20KXoiB5d1"
            xmlns="http://www.w3.org/2000/svg"
            xmlns:xlink="http://www.w3.org/1999/xlink"
            viewBox="0 0 12 14"
            shape-rendering="geometricPrecision"
            text-rendering="geometricPrecision"
            class="animated-playing"
        >
            <g
                id="eF20KXoiB5d2_ts"
                class=("animation-play", playing)
                class=("animation-pause", !playing)
                transform="translate(1.989899,14) scale(1,1)"
            >
                <path
                    id="eF20KXoiB5d2"
                    d="M3.337220,0.381592L0.642578,0.381592L0.642578,13.991600L3.337220,13.991600L3.337220,0.381592Z"
                    transform="translate(-1.989899,-13.991600)"
                    fill="#ffffff"
                    stroke="none"
                    stroke-width="1"
                    stroke-miterlimit="1"
                />
            </g>
            <g
                id="eF20KXoiB5d3_ts"
                class=("animation-play", playing)
                class=("animation-pause", !playing)
                transform="translate(6.030915,14.000671) scale(1,1)"
            >
                <path
                    id="eF20KXoiB5d3"
                    d="M7.378240,5.199710L4.683590,5.199710L4.683590,13.991700L7.378240,13.991700L7.378240,5.199710Z"
                    transform="translate(-6.030915,-13.991600)"
                    fill="#ffffff"
                    stroke="none"
                    stroke-width="1"
                    stroke-miterlimit="1"
                />
            </g>
            <g
                id="eF20KXoiB5d4_ts"
                class=("animation-play", playing)
                class=("animation-pause", !playing)
                transform="translate(9.851647,14.039325) scale(1,1)"
            >
                <path
                    id="eF20KXoiB5d4"
                    d="M11.175100,8.803340L8.480470,8.803340L8.480470,13.991600L11.175100,13.991600L11.175100,8.803340Z"
                    transform="translate(-9.827785,-13.991600)"
                    fill="#ffffff"
                    stroke="none"
                    stroke-width="1"
                    stroke-miterlimit="1"
                />
            </g>
        </svg>
    }
}
