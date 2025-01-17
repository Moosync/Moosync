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
use types::ui::extensions::FetchedExtensionManifest;

use crate::{
    modals::common::GenericModal, store::modal_store::ModalStore,
    utils::invoke::get_extension_manifest,
};

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn DiscoverExtensionsModal() -> impl IntoView {
    let extensions = RwSignal::<Vec<FetchedExtensionManifest>>::new(vec![]);
    spawn_local(async move {
        let res = get_extension_manifest().await.unwrap();

        extensions.set(res);
    });

    let modal_store: RwSignal<ModalStore> = expect_context();

    view! {
        <GenericModal size=move || "modal-xl".into()>
            <div class="w-100 h-100">
                <div class="container overflow-container">
                    <div class="row no-gutters d-flex">
                        <div class="col heading">Discover Extensions</div>
                    </div>

                    <div class="container-fluid">
                        <For
                            each=move || extensions.get()
                            key=|e| e.package_name.clone()
                            children=move |extension| {
                                let extension_clone = extension.clone();
                                view! {
                                    <div class="row no-gutters align-items-center d-flex mt-4">
                                        <div class="col-auto">
                                            <div class="img-container">
                                                <img
                                                    src=extension.logo
                                                    style="width: 50px; border-radius: 0 !important; object-fit: contain;"
                                                />
                                            </div>
                                        </div>
                                        <div class="col ml-3 text-truncate">
                                            <div class="row">
                                                <div
                                                    class="col text-truncate title"
                                                    title=extension.name.clone()
                                                >
                                                    {extension.name.clone()}
                                                </div>
                                            </div>
                                            <div class="row">
                                                <div
                                                    class="col subtitle text-truncate"
                                                    title=extension.description.clone()
                                                >
                                                    {extension.description.clone()}
                                                </div>
                                            </div>
                                        </div>
                                        <div
                                            class="col-2 ml-3 text-center download-button"
                                            // let owner = owner.clone();
                                            on:click=move |_| {
                                                let extension = extension_clone.clone();
                                                spawn_local(async move {
                                                    crate::utils::invoke::download_extension(extension)
                                                        .await
                                                        .unwrap();
                                                    let close_modal = create_write_slice(
                                                        modal_store,
                                                        |m, _: ()| m.clear_active_modal(),
                                                    );
                                                    close_modal.set(());
                                                });
                                            }
                                        >
                                            Download
                                        </div>
                                    </div>
                                }
                            }
                        />
                    </div>

                </div>
            </div>
        </GenericModal>
    }
}
