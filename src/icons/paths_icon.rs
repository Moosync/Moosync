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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

use leptos::{component, prelude::*, view, IntoView};

#[tracing::instrument(level = "trace", skip(active))]
#[component]
pub fn PathsIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="19"
            height="17"
            viewBox="0 0 19 17"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M19 14.0436C18.9999 14.7632 18.7404 15.4581 18.27 15.998C17.7997 16.5379 17.1508 16.8858 16.4451 16.9764C15.7393 17.0671 15.025 16.8943 14.4362 16.4905C13.8473 16.0866 13.4243 15.4795 13.2464 14.7827H4.38462C3.22174 14.7827 2.1065 14.3155 1.28422 13.4838C0.461949 12.6521 0 11.5241 0 10.3479C0 9.17173 0.461949 8.04371 1.28422 7.21202C2.1065 6.38033 3.22174 5.91309 4.38462 5.91309H13.1538C13.7353 5.91309 14.2929 5.67947 14.704 5.26363C15.1152 4.84778 15.3462 4.28378 15.3462 3.69568C15.3462 3.10759 15.1152 2.54358 14.704 2.12774C14.2929 1.71189 13.7353 1.47827 13.1538 1.47827H4.38462C4.1908 1.47827 4.00493 1.4004 3.86788 1.26179C3.73084 1.12317 3.65385 0.935168 3.65385 0.739137C3.65385 0.543105 3.73084 0.355103 3.86788 0.216488C4.00493 0.0778732 4.1908 0 4.38462 0H13.1538C14.1229 0 15.0523 0.389365 15.7375 1.08244C16.4227 1.77552 16.8077 2.71553 16.8077 3.69568C16.8077 4.67584 16.4227 5.61585 15.7375 6.30893C15.0523 7.002 14.1229 7.39137 13.1538 7.39137H4.38462C3.60937 7.39137 2.86587 7.70286 2.31769 8.25732C1.7695 8.81178 1.46154 9.56379 1.46154 10.3479C1.46154 11.132 1.7695 11.884 2.31769 12.4385C2.86587 12.993 3.60937 13.3045 4.38462 13.3045H13.2464C13.4243 12.6077 13.8473 12.0006 14.4362 11.5967C15.025 11.1929 15.7393 11.0201 16.4451 11.1108C17.1508 11.2014 17.7997 11.5493 18.27 12.0892C18.7404 12.6291 18.9999 13.324 19 14.0436Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            />
        </svg>
    }
}
