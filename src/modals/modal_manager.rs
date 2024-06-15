use leptos::{
    component, expect_context, view, IntoView, RwSignal, SignalGet, SignalUpdateUntracked,
};

use crate::{
    modals::login_modal::LoginModal,
    store::modal_store::{ModalStore, Modals},
};

#[component]
pub fn ModalManager() -> impl IntoView {
    let modal_store = expect_context::<RwSignal<ModalStore>>();

    let render_active_modal = move || {
        let active_modal = modal_store.get().active_modal;
        if active_modal.is_none() {
            return view! {}.into_view();
        }

        let ret = match active_modal.unwrap() {
            Modals::LoginModal(key) => view! { <LoginModal key=key/> },
        }
        .into_view();

        modal_store.update_untracked(move |m| m.clear_active_modal());

        ret
    };

    view! { render_active_modal }
}
