use leptos::{expect_context, RwSignal, SignalUpdate};
use leptos_context_menu::ContextMenuItemInner;

use crate::store::ui_store::{PlaylistSortBy, PlaylistSortByColumns, UiStore};

fn sort_by_name() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_playlist_sort_by(PlaylistSortBy {
            sort_by: PlaylistSortByColumns::Title,
            asc: true,
        })
    });
}

fn sort_by_provider() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_playlist_sort_by(PlaylistSortBy {
            sort_by: PlaylistSortByColumns::Provider,
            asc: true,
        })
    });
}

pub fn get_playlist_sort_cx_items<T>() -> Vec<ContextMenuItemInner<T>> {
    vec![
        ContextMenuItemInner::new_with_handler("Title".into(), |_, _| sort_by_name(), None),
        ContextMenuItemInner::new_with_handler("Provider".into(), |_, _| sort_by_provider(), None),
    ]
}
