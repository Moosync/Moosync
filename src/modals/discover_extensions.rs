use leptos::{
    component, create_rw_signal, create_write_slice, expect_context, spawn_local, view, For,
    IntoView, RwSignal, SignalGet, SignalSet,
};

use crate::{
    modals::common::GenericModal,
    store::modal_store::ModalStore,
    utils::invoke::get_extension_manifest,
};

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn DiscoverExtensionsModal() -> impl IntoView {
    let extensions = create_rw_signal(vec![]);
    spawn_local(async move {
        let res = get_extension_manifest().await.unwrap();

        extensions.set(res);
    });

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
                            children=|extension| {
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
                                            cols="2"
                                            on:click=move |_| {
                                                let extension = extension_clone.clone();
                                                spawn_local(async move {
                                                    crate::utils::invoke::download_extension(extension)
                                                        .await
                                                        .unwrap();
                                                    let modal_store: RwSignal<ModalStore> = expect_context();
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
