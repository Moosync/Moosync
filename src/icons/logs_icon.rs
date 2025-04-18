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
pub fn LogsIcon(#[prop()] active: ReadSignal<bool>) -> impl IntoView {
    view! {
        <svg width="20" viewBox="0 0 27 25" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
                d="M15 15H27V17H15V15ZM15 19H27V21H15V19ZM15 23H23V25H15V23Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            />
            <path
                d="M20.9998 0C20.1155 0.00263111 19.2569 0.298577 18.5588 0.841467C17.8606 1.38436 17.3623 2.14352 17.1418 3H8.99984V0H0.999841V8H8.99984V5H17.1418C17.235 5.35793 17.378 5.70096 17.5668 6.019L7.01884 16.567C6.40918 16.1996 5.71162 16.0037 4.99984 16C4.25108 15.9972 3.51656 16.2046 2.87986 16.5987C2.24316 16.9927 1.72984 17.5575 1.39831 18.2289C1.06678 18.9003 0.930356 19.6512 1.00455 20.3963C1.07875 21.1414 1.36058 21.8507 1.818 22.4435C2.27541 23.0363 2.89004 23.4889 3.59194 23.7496C4.29384 24.0104 5.05484 24.0688 5.78835 23.9184C6.52186 23.768 7.19842 23.4148 7.74107 22.8988C8.28371 22.3829 8.67064 21.725 8.85784 21H12.9998V19H8.85784C8.7647 18.6421 8.62163 18.299 8.43284 17.981L18.9808 7.433C19.5905 7.80037 20.2881 7.99627 20.9998 8C22.0607 8 23.0781 7.57857 23.8283 6.82843C24.5784 6.07828 24.9998 5.06087 24.9998 4C24.9998 2.93913 24.5784 1.92172 23.8283 1.17157C23.0781 0.421427 22.0607 0 20.9998 0ZM6.99984 6H2.99984V2H6.99984V6ZM4.99984 22C4.60428 22 4.2176 21.8827 3.8887 21.6629C3.5598 21.4432 3.30346 21.1308 3.15208 20.7654C3.00071 20.3999 2.9611 19.9978 3.03827 19.6098C3.11544 19.2219 3.30592 18.8655 3.58563 18.5858C3.86533 18.3061 4.2217 18.1156 4.60966 18.0384C4.99762 17.9613 5.39976 18.0009 5.76521 18.1522C6.13066 18.3036 6.44302 18.56 6.66278 18.8889C6.88254 19.2178 6.99984 19.6044 6.99984 20C6.99931 20.5303 6.78843 21.0387 6.41347 21.4136C6.03851 21.7886 5.53011 21.9995 4.99984 22V22ZM20.9998 6C20.6043 6 20.2176 5.8827 19.8887 5.66294C19.5598 5.44318 19.3035 5.13082 19.1521 4.76537C19.0007 4.39991 18.9611 3.99778 19.0383 3.60982C19.1154 3.22186 19.3059 2.86549 19.5856 2.58579C19.8653 2.30608 20.2217 2.1156 20.6097 2.03843C20.9976 1.96126 21.3998 2.00087 21.7652 2.15224C22.1307 2.30362 22.443 2.55996 22.6628 2.88886C22.8825 3.21776 22.9998 3.60444 22.9998 4C22.9993 4.53027 22.7884 5.03867 22.4135 5.41363C22.0385 5.78859 21.5301 5.99947 20.9998 6V6Z"
                fill=move || if active.get() { "var(--accent)" } else { "var(--textPrimary)" }
            />
        </svg>
    }
}
