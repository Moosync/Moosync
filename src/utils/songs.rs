use leptos::{expect_context, RwSignal, SignalGet, SignalUpdate};
use leptos_context_menu::ContextMenuItemInner;
use types::songs::Song;

use crate::store::ui_store::{SongSortBy, SongSortByColumns, UiStore};

#[tracing::instrument(level = "trace", skip(song_list, song_indices))]
pub fn get_songs_from_indices<T, Y>(song_list: T, song_indices: Y) -> Vec<Song>
where
    T: SignalGet<Value = Vec<Song>>,
    Y: SignalGet<Value = Vec<usize>>,
{
    let song_list = song_list.get();
    let song_indices = song_indices.get();
    song_indices
        .iter()
        .map(|index| song_list.get(*index).unwrap().clone())
        .collect()
}

#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_album() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Album,
            asc: true,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_artist() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Artist,
            asc: true,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_date() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Date,
            asc: true,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_genre() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Genre,
            asc: true,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_playcount() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::PlayCount,
            asc: true,
        })
    });
}
#[tracing::instrument(level = "trace", skip())]
pub fn sort_by_title() {
    let ui_store: RwSignal<UiStore> = expect_context();
    ui_store.update(|ui_store| {
        ui_store.set_song_sort_by(SongSortBy {
            sort_by: SongSortByColumns::Title,
            asc: true,
        })
    });
}

#[tracing::instrument(level = "trace", skip())]
pub fn get_sort_cx_items<T>() -> Vec<ContextMenuItemInner<T>> {
    vec![
        ContextMenuItemInner::new_with_handler(
            "Album (Track No.)".into(),
            |_, _| sort_by_album(),
            None,
        ),
        ContextMenuItemInner::new_with_handler("Artist".into(), |_, _| sort_by_artist(), None),
        ContextMenuItemInner::new_with_handler("Date Added".into(), |_, _| sort_by_date(), None),
        ContextMenuItemInner::new_with_handler("Genre".into(), |_, _| sort_by_genre(), None),
        ContextMenuItemInner::new_with_handler(
            "Play count".into(),
            |_, _| sort_by_playcount(),
            None,
        ),
        ContextMenuItemInner::new_with_handler("Title".into(), |_, _| sort_by_title(), None),
    ]
}
