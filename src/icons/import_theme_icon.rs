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
pub fn ImportThemeIcon() -> impl IntoView {
    view! {
        <svg data-v-e030a87e="" viewBox="0 0 62 62" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
                d="M11.7219 62C9.99028 62 8.50842 61.3934 7.27633 60.1803C6.04214 58.9651 5.42505 57.505 5.42505 55.8V21.1575C5.42505 20.3308 5.58247 19.5424 5.89731 18.7922C6.21216 18.0441 6.65819 17.3858 7.2354 16.8175L22.5053 1.7825C23.0825 1.21417 23.7511 0.775 24.5109 0.465C25.2728 0.155 26.0736 0 26.9131 0H49.5032C51.2348 0 52.7177 0.606566 53.9519 1.8197C55.184 3.0349 55.8 4.495 55.8 6.2V55.8C55.8 57.505 55.184 58.9651 53.9519 60.1803C52.7177 61.3934 51.2348 62 49.5032 62H11.7219ZM11.7219 55.8H49.5032V6.2H26.9918L11.7219 21.235V55.8ZM30.6125 45.1825C31.0323 45.1825 31.4259 45.1174 31.7932 44.9872C32.1605 44.8591 32.5016 44.64 32.8165 44.33L41.0811 36.1925C41.6583 35.6242 41.9469 34.9267 41.9469 34.1C41.9469 33.2733 41.6321 32.55 41.0024 31.93C40.4252 31.3617 39.7042 31.0775 38.8394 31.0775C37.9725 31.0775 37.2243 31.3617 36.5946 31.93L33.761 34.565V24.8C33.761 23.9217 33.4598 23.1849 32.8574 22.5897C32.2529 21.9966 31.5046 21.7 30.6125 21.7C29.7205 21.7 28.9733 21.9966 28.3709 22.5897C27.7664 23.1849 27.4641 23.9217 27.4641 24.8V34.565L24.6305 31.8525C24.0008 31.2842 23.2662 31 22.4266 31C21.587 31 20.8524 31.31 20.2227 31.93C19.6455 32.4983 19.3569 33.2217 19.3569 34.1C19.3569 34.9783 19.6455 35.7017 20.2227 36.27L28.4086 44.33C28.7235 44.64 29.0646 44.8591 29.4319 44.9872C29.7992 45.1174 30.1928 45.1825 30.6125 45.1825Z"
                fill="var(--textPrimary)"
            ></path>
        </svg>
    }
}
