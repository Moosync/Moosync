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

#[tracing::instrument(level = "debug", skip(active))]
#[component]
pub fn ArtistsIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="20"
            height="23"
            viewBox="0 0 20 23"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <title>Artists</title>
            <path
                d="M13.4205 8.18772L14.8471 4.41899C15.0251 3.9487 14.781 3.42577 14.3018 3.25106C13.8226 3.07634 13.2898 3.31592 13.1118 3.78625L11.9685 6.80639L9.08773 6.15274L11.2606 6.21768C10.9993 5.65166 10.3649 5.31893 9.71979 5.45223L7.31145 5.94977C6.5656 6.10382 6.08822 6.82217 6.24523 7.55418L7.64291 14.0717L7.4535 17.2378L5.15447 20.9882C4.8475 21.489 5.01228 22.1392 5.52254 22.4405C6.03287 22.7418 6.69532 22.58 7.00225 22.0792L9.43813 18.1054C9.52803 17.9588 9.58036 17.7929 9.59063 17.622L9.81218 13.9191L10.693 13.7371L11.8809 16.5731L11.9328 21.5446C11.9388 22.1199 12.4146 22.5956 13.0219 22.5919C13.6174 22.5859 14.0951 22.1073 14.0891 21.523L14.035 16.3481C14.0336 16.2137 14.0061 16.0808 13.9541 15.9566L12.7838 13.1624L11.9739 9.38597L8.67097 7.92296L12.3445 8.75652C12.7977 8.85929 13.2584 8.61592 13.4205 8.18772Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            ></path>
            <path
                d="M10.2249 3.48757C10.2969 3.20308 10.3063 2.89865 10.2406 2.59253C10.0082 1.50885 8.92475 0.815302 7.82063 1.04341C6.71646 1.27151 6.0098 2.33486 6.24222 3.41849C6.47464 4.50221 7.55813 5.19576 8.66221 4.96766C9.36092 4.82331 9.90005 4.34432 10.1432 3.73756L10.2249 3.48757Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            ></path>
        </svg>
    }
}
