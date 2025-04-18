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

#[tracing::instrument(level = "debug", skip(active))]
#[component]
pub fn AlbumsIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="20"
            height="19"
            viewBox="0 0 20 19"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <title>Albums</title>
            <path
                d="M16.5 15.6924C14.8931 17.2993 12.748 18.2541 10.4785 18.3723C8.20909 18.4906 5.97628 17.7641 4.21102 16.333C2.44576 14.9018 1.27318 12.8675 0.919594 10.6227C0.566008 8.37784 1.05648 6.0816 2.29634 4.17711C3.53621 2.27263 5.43757 0.894916 7.63349 0.309865C9.8294 -0.275186 12.1642 -0.0261005 14.1873 1.00905C16.2103 2.0442 17.7782 3.79203 18.5884 5.91522C19.3986 8.03841 19.3936 10.3864 18.5744 12.5062L12.6495 10.2163C12.9026 9.56136 12.9042 8.83583 12.6538 8.17977C12.4035 7.52371 11.919 6.98364 11.2939 6.66378C10.6688 6.34392 9.94731 6.26696 9.26878 6.44773C8.59025 6.62851 8.00274 7.05422 7.61962 7.6427C7.23651 8.23118 7.08496 8.94071 7.19421 9.63436C7.30347 10.328 7.66579 10.9566 8.21125 11.3988C8.75671 11.841 9.44664 12.0655 10.1479 12.029C10.8491 11.9924 11.512 11.6974 12.0085 11.2009L16.5 15.6924Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            ></path>
        </svg>
    }
}
