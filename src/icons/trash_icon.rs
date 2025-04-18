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
pub fn TrashIcon() -> impl IntoView {
    view! {
        <svg viewBox="0 0 27 27" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path
                d="M20.3552 25.5H6.64515C6.35655 25.4932 6.07211 25.4296 5.8081 25.3128C5.54409 25.196 5.30567 25.0284 5.10648 24.8194C4.90728 24.6105 4.7512 24.3643 4.64717 24.095C4.54313 23.8257 4.49318 23.5386 4.50015 23.25V8.42249H6.00015V23.25C5.99299 23.3416 6.00402 23.4338 6.0326 23.5212C6.06119 23.6085 6.10676 23.6894 6.1667 23.7591C6.22664 23.8288 6.29977 23.886 6.38188 23.9273C6.46399 23.9686 6.55346 23.9933 6.64515 24H20.3552C20.4468 23.9933 20.5363 23.9686 20.6184 23.9273C20.7005 23.886 20.7737 23.8288 20.8336 23.7591C20.8935 23.6894 20.9391 23.6085 20.9677 23.5212C20.9963 23.4338 21.0073 23.3416 21.0002 23.25V8.42249H22.5002V23.25C22.5071 23.5386 22.4572 23.8257 22.3531 24.095C22.2491 24.3643 22.093 24.6105 21.8938 24.8194C21.6946 25.0284 21.4562 25.196 21.1922 25.3128C20.9282 25.4296 20.6438 25.4932 20.3552 25.5V25.5Z"
                fill="#F02121"
            ></path>
            <path
                d="M23.085 6.75H3.75C3.55109 6.75 3.36032 6.67098 3.21967 6.53033C3.07902 6.38968 3 6.19891 3 6C3 5.80109 3.07902 5.61032 3.21967 5.46967C3.36032 5.32902 3.55109 5.25 3.75 5.25H23.085C23.2839 5.25 23.4747 5.32902 23.6153 5.46967C23.756 5.61032 23.835 5.80109 23.835 6C23.835 6.19891 23.756 6.38968 23.6153 6.53033C23.4747 6.67098 23.2839 6.75 23.085 6.75Z"
                fill="#F02121"
            ></path>
            <path d="M15.75 9.75H17.25V21H15.75V9.75Z" fill="#F02121"></path>
            <path d="M9.75 9.75H11.25V21H9.75V9.75Z" fill="#F02121"></path>
            <path
                d="M17.25 4.395H15.825V3H11.175V4.395H9.75V3C9.74952 2.61484 9.89722 2.24424 10.1625 1.965C10.4278 1.68576 10.7903 1.51926 11.175 1.5H15.825C16.2097 1.51926 16.5722 1.68576 16.8375 1.965C17.1028 2.24424 17.2505 2.61484 17.25 3V4.395Z"
                fill="#F02121"
            ></path>
        </svg>
    }
}
