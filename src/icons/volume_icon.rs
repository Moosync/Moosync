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

#[tracing::instrument(level = "trace", skip(cut))]
#[component]
pub fn VolumeIcon(#[prop()] cut: impl Get<Value = bool> + 'static + Send + Sync) -> impl IntoView {
    view! {
        <svg
            class="vol-icon button-grow volume-icon align-self-center"
            viewBox="0 0 8 8"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M7.29195 1.5182L6.90933 1.84081C7.30241 2.30644 7.51195 2.89958 7.4986 3.50881C7.48525 4.11804 7.24991 4.70143 6.8368 5.14939L7.20442 5.49951C7.70049 4.96195 7.98315 4.26171 7.99927 3.53041C8.01539 2.79911 7.76386 2.0871 7.29195 1.5282V1.5182Z"
                fill="var(--accent)"
            ></path>
            <path
                d="M5.89652 2.49852C6.13231 2.77812 6.25785 3.13421 6.24957 3.49986C6.24128 3.86552 6.09972 4.21555 5.85151 4.48418L6.21912 4.82429C6.54995 4.46576 6.73838 3.99871 6.74897 3.51099C6.75956 3.02326 6.59159 2.54847 6.27664 2.17592L5.89652 2.49852Z"
                fill="var(--accent)"
            ></path>
            <path
                d="M5.00126 7C4.96818 6.99986 4.93545 6.99316 4.90497 6.98028C4.87449 6.9674 4.84687 6.94861 4.82371 6.92498L2.91811 4.99934H1.25008C1.18375 4.99934 1.12015 4.97299 1.07325 4.9261C1.02635 4.8792 1 4.81559 1 4.74926V2.24844C1 2.18211 1.02635 2.1185 1.07325 2.0716C1.12015 2.0247 1.18375 1.99836 1.25008 1.99836H2.91811L4.82371 0.0727221C4.87056 0.026144 4.93395 0 5.00001 0C5.06608 0 5.12946 0.026144 5.17632 0.0727221C5.22344 0.118895 5.2504 0.181816 5.25134 0.24778V6.74992C5.25134 6.81624 5.225 6.87985 5.1781 6.92675C5.1312 6.97365 5.06759 7 5.00126 7ZM1.50016 4.49918H3.02064C3.05373 4.49932 3.08646 4.50602 3.11693 4.5189C3.14741 4.53177 3.17503 4.55057 3.19819 4.5742L4.75119 6.14222V0.855479L3.19819 2.4235C3.17503 2.44712 3.14741 2.46592 3.11693 2.4788C3.08646 2.49168 3.05373 2.49838 3.02064 2.49852H1.50016V4.49918Z"
                fill="var(--accent)"
            ></path>

            {move || {
                if cut.get() {
                    view! {
                        <rect
                            y="0.37616"
                            width="0.531963"
                            height="10.7817"
                            rx="0.265982"
                            transform="rotate(-45 0 0.37616)"
                            fill="var(--accent)"
                        ></rect>
                    }
                        .into_any()
                } else {
                    ().into_any()
                }
            }}

        </svg>
    }
}
