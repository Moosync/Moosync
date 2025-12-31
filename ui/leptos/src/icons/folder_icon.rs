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
pub fn FolderIcon() -> impl IntoView {
    view! {
        <svg
            width="26"
            height="20"
            viewBox="0 0 26 20"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M24.96 3.4878H13.2925L9.48025 0.0670731C9.43174 0.0244748 9.36794 0.000534441 9.3015 0H1.04C0.46475 0 0 0.435976 0 0.97561V19.0244C0 19.564 0.46475 20 1.04 20H24.96C25.5352 20 26 19.564 26 19.0244V4.46341C26 3.92378 25.5352 3.4878 24.96 3.4878Z"
                fill="var(--textPrimary)"
            />
        </svg>
    }
}
