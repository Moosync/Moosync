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

use leptos::{component, prelude::*, task::spawn_local, view, IntoView};
use serde::Serialize;

use crate::{
    modals::common::GenericModal,
    utils::invoke::{provider_authorize, provider_login},
};

#[tracing::instrument(level = "trace", skip(key, name, account_id))]
#[component]
pub fn LoginModal(
    #[prop()] key: String,
    #[prop()] name: String,
    account_id: String,
) -> impl IntoView {
    let having_trouble = RwSignal::new(false);
    let code = RwSignal::new(String::new());
    let url = RwSignal::new(String::new());

    let key_cloned = key.clone();

    let authorize = Action::new_local(move |code: &String| {
        let code = code.clone();
        let key = key.clone();

        async move { provider_authorize(key, code).await }
    });

    spawn_local(async move {
        let ret = provider_login(key_cloned, account_id).await.unwrap();
        url.set(ret);
    });

    let open_external = move |_| {
        let url = url.get();

        #[derive(Serialize)]
        struct OpenExternalArgs {
            url: String,
        }

        spawn_local(async move {
            let res = crate::utils::invoke::open_external(url).await;
            if let Err(e) = res {
                tracing::error!("Failed to open external: {:?}", e);
            }
        });
    };

    view! {
        <GenericModal size=move || "modal-sm".into()>
            <div class="w-100 h-100">
                <div class="container response-container">
                    <div class="row no-gutters d-flex">
                        <div class="col-auto title">Logging in to</div>
                        <div class="col-auto title ml-1" style="color: var(--accent)">
                            {name}
                        </div>
                    </div>

                    {move || {
                        if !having_trouble.get() {
                            view! {
                                <div>
                                    <div class="row">
                                        <div class="col mt-4 waiting">
                                            Waiting for response from your browser
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col d-flex justify-content-center">
                                            <div
                                                on:click=open_external
                                                class="start-button button-grow mt-4 d-flex justify-content-center align-items-center"
                                            >
                                                Open browser
                                            </div>
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div
                                            class="col not-working-text mt-3"
                                            on:click=move |_| having_trouble.set(true)
                                        >
                                            Having trouble?
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div>
                                    <div class="row">
                                        <div class="col mt-4 waiting">
                                            Paste this link in your browser...
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col">
                                            <input
                                                class="ext-input mt-3"
                                                readonly
                                                prop:value=move || url.get()
                                            />
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col mt-4 waiting">
                                            Then enter the code shown after the login process has completed
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col">
                                            <input
                                                class="login-input mt-3"
                                                placeholder="Code"
                                                on:input=move |ev| code.set(event_target_value(&ev))
                                            />
                                        </div>
                                    </div>
                                    <div class="row">
                                        <div class="col d-flex justify-content-center">
                                            <div
                                                class="start-button button-grow mt-4 d-flex justify-content-center align-items-center"
                                                on:click=move |_| {
                                                    authorize.dispatch_local(code.get_untracked());
                                                }
                                            >
                                                Submit
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    }}

                </div>
            </div>
        </GenericModal>
    }
}
