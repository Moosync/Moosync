use itertools::Itertools;
use leptos::{expect_context, RwSignal, SignalUpdate};
use leptos_context_menu::ContextMenuItemInner;
use types::entities::QueryableArtist;

use crate::store::ui_store::{PlaylistSortBy, PlaylistSortByColumns, UiStore};

#[tracing::instrument(level = "trace", skip())]
fn sort_by_name() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_playlist_sort_by(PlaylistSortBy {
            sort_by: PlaylistSortByColumns::Title,
            asc: true,
        })
    });
}

#[tracing::instrument(level = "trace", skip())]
fn sort_by_provider() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_playlist_sort_by(PlaylistSortBy {
            sort_by: PlaylistSortByColumns::Provider,
            asc: true,
        })
    });
}

#[tracing::instrument(level = "trace", skip())]
pub fn get_playlist_sort_cx_items<T>() -> Vec<ContextMenuItemInner<T>> {
    vec![
        ContextMenuItemInner::new_with_handler("Title".into(), |_, _| sort_by_name(), None),
        ContextMenuItemInner::new_with_handler("Provider".into(), |_, _| sort_by_provider(), None),
    ]
}

#[tracing::instrument(level = "trace", skip(artists))]
pub fn get_artist_string(artists: Option<Vec<QueryableArtist>>) -> String {
    artists
        .map(|a| a.iter().filter_map(|a| a.artist_name.clone()).join(", "))
        .unwrap_or_default()
}
