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
pub fn FetchAllIcon() -> impl IntoView {
    view! {
        <svg
            width="25"
            height="25"
            viewBox="0 0 37 37"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
            class="button-grow"
        >
            <title>Fetch all</title>
            <g clip-path="url(#clip0_3204_3743)">
                <path
                    d="M31.3414 30.893C33.2443 29.0071 34.6993 26.7179 35.5989 24.1943C36.4985 21.6708 36.8197 18.9774 36.5389 16.313C35.7064 8.05553 29.0014 1.26053 20.7439 0.338028C9.83141 -0.899472 0.62891 7.53803 0.628909 18.1805C0.628905 21.5783 1.59299 24.9063 3.40916 27.7779C5.22533 30.6495 7.81902 32.9468 10.8889 34.403C12.3964 35.123 14.1289 34.043 14.1289 32.378C14.1289 31.5455 13.6789 30.758 12.9364 30.398C10.1261 29.0917 7.85145 26.8573 6.49508 24.0708C5.13871 21.2843 4.7834 18.1157 5.48891 15.098C6.59141 10.103 10.6639 6.07553 15.6589 5.01803C17.6338 4.5735 19.6834 4.57865 21.6561 5.03309C23.6287 5.48753 25.474 6.37965 27.0553 7.64344C28.6367 8.90724 29.9137 10.5104 30.7919 12.3343C31.6701 14.1582 32.1271 16.1562 32.1289 18.1805C32.1289 21.9155 30.5764 25.2455 28.1239 27.6755L24.7264 24.278C23.3089 22.8605 20.8789 23.8505 20.8789 25.853L20.8789 33.9305C20.8789 35.168 21.8914 36.1805 23.1289 36.1805L31.2064 36.1805C33.2089 36.1805 34.2214 33.7505 32.8039 32.333L31.3414 30.893Z"
                    fill="var(--accent)"
                ></path>
                <path
                    d="M31.3414 30.893C33.2443 29.0071 34.6993 26.7179 35.5989 24.1943C36.4985 21.6708 36.8197 18.9774 36.5389 16.313C35.7064 8.05553 29.0014 1.26053 20.7439 0.338028C9.83141 -0.899472 0.62891 7.53803 0.628909 18.1805C0.628905 21.5783 1.59299 24.9063 3.40916 27.7779C5.22533 30.6495 7.81902 32.9468 10.8889 34.403C12.3964 35.123 14.1289 34.043 14.1289 32.378C14.1289 31.5455 13.6789 30.758 12.9364 30.398C10.1261 29.0917 7.85145 26.8573 6.49508 24.0708C5.13871 21.2843 4.7834 18.1157 5.48891 15.098C6.59141 10.103 10.6639 6.07553 15.6589 5.01803C17.6338 4.5735 19.6834 4.57865 21.6561 5.03309C23.6287 5.48753 25.474 6.37965 27.0553 7.64344C28.6367 8.90724 29.9137 10.5104 30.7919 12.3343C31.6701 14.1582 32.1271 16.1562 32.1289 18.1805C32.1289 21.9155 30.5764 25.2455 28.1239 27.6755L24.7264 24.278C23.3089 22.8605 20.8789 23.8505 20.8789 25.853L20.8789 33.9305C20.8789 35.168 21.8914 36.1805 23.1289 36.1805L31.2064 36.1805C33.2089 36.1805 34.2214 33.7505 32.8039 32.333L31.3414 30.893Z"
                    stroke="var(--accent)"
                ></path>
            </g>
            <defs>
                <clipPath id="clip0_3204_3743">
                    <rect
                        width="37"
                        height="37"
                        fill="var(--accent)"
                        transform="translate(37) rotate(90)"
                    ></rect>
                </clipPath>
            </defs>
        </svg>
    }
}
