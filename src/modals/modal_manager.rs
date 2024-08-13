use leptos::{
    component, expect_context, view, IntoView, RwSignal, SignalGet,
};

use crate::{
    console_log,
    modals::{discover_extensions::DiscoverExtensionsModal, login_modal::LoginModal},
    store::modal_store::{ModalStore, Modals},
};

#[component]
pub fn ModalManager() -> impl IntoView {
    let modal_store = expect_context::<RwSignal<ModalStore>>();

    view! {
        <div>
            {move || {
                let active_modal = modal_store.get().active_modal;
                console_log!("Got active modal {:?}", active_modal);
                if active_modal.is_none() {
                    return view! {}.into_view();
                }
                
                match active_modal.unwrap() {
                    Modals::LoginModal(key) => {
                        view! { <LoginModal key=key /> }
                    }
                    Modals::DiscoverExtensions => {
                        view! { <DiscoverExtensionsModal /> }
                    }
                }
                    .into_view()
            }}

        </div>
    }
}
