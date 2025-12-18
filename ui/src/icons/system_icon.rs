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
pub fn SystemIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg
            width="18"
            height="17"
            viewBox="0 0 18 17"
            fill="none"
            xmlns="http://www.w3.org/2000/svg"
        >
            <path
                d="M13.5 15.9013H12.5212C12.2892 15.7413 12.0943 15.5357 11.9489 15.2977C11.8036 15.0596 11.7111 14.7945 11.6775 14.5195V13.7793H6.32247V14.5305C6.28733 14.8036 6.19415 15.0665 6.04887 15.3026C5.90359 15.5386 5.7094 15.7425 5.47872 15.9013H4.46622C4.31703 15.9013 4.17396 15.959 4.06847 16.0619C3.96298 16.1647 3.90372 16.3042 3.90372 16.4496C3.90372 16.595 3.96298 16.7345 4.06847 16.8373C4.17396 16.9401 4.31703 16.9979 4.46622 16.9979H13.5337C13.6093 17.0046 13.6855 16.9951 13.7569 16.9699C13.8283 16.9447 13.8931 16.9046 13.9468 16.8522C14.0005 16.7999 14.0417 16.7367 14.0675 16.6671C14.0933 16.5975 14.1031 16.5233 14.0962 16.4496C14.0964 16.3747 14.0808 16.3006 14.0504 16.2319C14.0201 16.1631 13.9756 16.1011 13.9197 16.0497C13.8639 15.9984 13.7978 15.9587 13.7256 15.9332C13.6534 15.9076 13.5766 15.8968 13.5 15.9013Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            />
            <path
                d="M17.1562 0H0.84375C0.619974 0 0.405362 0.0866542 0.247129 0.240899C0.0888949 0.395145 0 0.604346 0 0.822482V12.3372C0 12.5554 0.0888949 12.7646 0.247129 12.9188C0.405362 13.0731 0.619974 13.1597 0.84375 13.1597H17.1562C17.38 13.1597 17.5946 13.0731 17.7529 12.9188C17.9111 12.7646 18 12.5554 18 12.3372V0.822482C18 0.604346 17.9111 0.395145 17.7529 0.240899C17.5946 0.0866542 17.38 0 17.1562 0ZM16.3125 10.3249H1.6875V2.19329H16.3125V10.3249Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            />
        </svg>
    }
}
