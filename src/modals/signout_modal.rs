use leptos::{component, prelude::*, task::spawn_local, view, IntoView};

use crate::{
    modals::common::GenericModal, store::modal_store::ModalStore, utils::invoke::provider_signout,
};

#[tracing::instrument(level = "trace", skip(key, account_id, name))]
#[component]
pub fn SignoutModal(#[prop()] key: String, account_id: String, name: String) -> impl IntoView {
    let modal_store: RwSignal<ModalStore> = expect_context();
    let close_modal = move |_| modal_store.update(|m| m.clear_active_modal());

    let signout = move |_| {
        let key = key.clone();
        let account_id = account_id.clone();
        spawn_local(async move {
            provider_signout(key, account_id).await.unwrap();

            modal_store.update(|m| m.clear_active_modal());
        });
    };
    view! {
        <GenericModal size=move || "modal-lg".into()>
            <div class="container-fluid p-0 mt-4">
                <div class="row no-gutters d-flex">
                    <div class="col">
                        <h4>
                            Are you sure you want to <span class="keyword">log out from</span>
                            <span class="logout-item">{name.clone()}</span>?
                        </h4>
                        <h6 class="mt-3">Press Confirm if you are sure</h6>
                    </div>
                </div>
                <div class="row row-cols-auto mt-3 mr-4">
                    <button
                        on:click=close_modal
                        class="btn btn-secondary cancel-button ml-auto"
                        type="button"
                    >
                        Cancel
                    </button>
                    <button
                        on:click=signout
                        class="btn btn-secondary confirm-button ml-3"
                        type="button"
                    >
                        Confirm
                    </button>
                </div>
            </div>
        </GenericModal>
    }
}
