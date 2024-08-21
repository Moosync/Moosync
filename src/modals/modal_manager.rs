use leptos::{component, expect_context, view, IntoView, RwSignal, SignalGet};

use crate::{
    console_log,
    modals::{
        discover_extensions::DiscoverExtensionsModal, login_modal::LoginModal,
        new_playlist_modal::NewPlaylistModal, song_from_url_modal::SongFromUrlModal,
    },
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
                    Modals::LoginModal(key, name) => {
                        view! { <LoginModal key=key name=name /> }
                    }
                    Modals::DiscoverExtensions => {
                        view! { <DiscoverExtensionsModal /> }
                    }
                    Modals::NewPlaylistModal => {
                        view! { <NewPlaylistModal /> }
                    }
                    Modals::SongFromUrlModal => {
                        view! { <SongFromUrlModal /> }
                    }
                }
                    .into_view()
            }}

        </div>
    }
}
