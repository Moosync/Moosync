use leptos::{component, expect_context, view, IntoView, RwSignal, SignalGet};

use crate::{
    modals::{
        discover_extensions::DiscoverExtensionsModal, login_modal::LoginModal,
        new_playlist_modal::NewPlaylistModal, new_theme_modal::NewThemeModal,
        signout_modal::SignoutModal, song_from_url_modal::SongFromUrlModal,
    },
    store::modal_store::{ModalStore, Modals},
};

#[tracing::instrument(level = "trace", skip())]
#[component]
pub fn ModalManager() -> impl IntoView {
    let modal_store = expect_context::<RwSignal<ModalStore>>();

    view! {
        <div>
            {move || {
                let active_modal = modal_store.get().active_modal;
                tracing::debug!("Got active modal {:?}", active_modal);
                if active_modal.is_none() {
                    return view! {}.into_view();
                }
                match active_modal.unwrap() {
                    Modals::LoginModal(key, name, account_id) => {
                        view! { <LoginModal key=key name=name account_id=account_id /> }
                    }
                    Modals::DiscoverExtensions => {
                        view! { <DiscoverExtensionsModal /> }
                    }
                    Modals::NewPlaylistModal(initial_state, songs) => {
                        view! { <NewPlaylistModal initial_state=initial_state songs=songs /> }
                    }
                    Modals::SongFromUrlModal => {
                        view! { <SongFromUrlModal /> }
                    }
                    Modals::SignoutModal(key, name, account_id) => {
                        view! { <SignoutModal key=key name=name account_id=account_id /> }
                    }
                    Modals::ThemeModal => {
                        view! { <NewThemeModal /> }
                    }
                }
                    .into_view()
            }}

        </div>
    }
}
