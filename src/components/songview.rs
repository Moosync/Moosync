use std::{ops::Deref, rc::Rc};

use leptos::{
    component, create_effect, create_node_ref, create_read_slice, create_rw_signal, event_target,
    expect_context, html::Div, leptos_dom::Element, view, HtmlElement, IntoView, RwSignal, Signal,
    SignalGet, SignalSet, SignalUpdate,
};
use leptos_context_menu::{ContextMenu, ContextMenuData, ContextMenuItemInner};
use leptos_use::{on_click_outside, on_click_outside_with_options, OnClickOutsideOptions};
use types::{songs::Song, ui::song_details::SongDetailIcons};
use wasm_bindgen::JsCast;
use web_sys::{Event, Node};

use crate::{
    components::{songdetails::SongDetails, songlist::SongList},
    console_log,
    store::{
        modal_store::{ModalStore, Modals},
        ui_store::{SongSortBy, SongSortByColumns, UiStore},
    },
    utils::songs::{
        get_sort_cx_items, sort_by_album, sort_by_artist, sort_by_date, sort_by_genre,
        sort_by_playcount, sort_by_title,
    },
};

struct SongsContextMenu {
    ui_store: RwSignal<UiStore>,
    current_sort: Signal<SongSortBy>,
    song_update_request: Option<Rc<Box<dyn Fn()>>>,
}

impl SongsContextMenu {
    pub fn new(ui_store: RwSignal<UiStore>, song_update_request: Option<Box<dyn Fn()>>) -> Self {
        Self {
            ui_store,
            current_sort: create_read_slice(ui_store, |ui_store| ui_store.get_song_sort_by()),
            song_update_request: song_update_request.map(|c| Rc::new(c)),
        }
    }

    pub fn add_song_from_url(&self) {
        let modal_store: RwSignal<ModalStore> = expect_context();
        modal_store.update(|modal_store| {
            modal_store.set_active_modal(Modals::SongFromUrlModal);
            let cb = self.song_update_request.clone();
            modal_store.on_modal_close(move || {
                if cb.is_some() {
                    let cb = cb.clone().unwrap();
                    cb();
                }
            });
        });
    }
}

impl ContextMenuData<Self> for SongsContextMenu {
    fn get_menu_items(&self) -> leptos_context_menu::ContextMenuItems<Self> {
        vec![
            ContextMenuItemInner::new("Sort by".into(), Some(get_sort_cx_items())),
            ContextMenuItemInner::new_with_handler(
                "Add from Url".into(),
                |_, cx| cx.add_song_from_url(),
                None,
            ),
        ]
    }
}

#[component()]
pub fn SongView(
    #[prop()] songs: RwSignal<Vec<Song>>,
    #[prop()] icons: RwSignal<SongDetailIcons>,
    #[prop()] selected_songs: RwSignal<Vec<usize>>,
    #[prop(optional)] song_update_request: Option<Box<dyn Fn()>>,
) -> impl IntoView {
    let last_selected_song = create_rw_signal(None::<Song>);

    let filtered_selected = create_rw_signal(vec![]);

    create_effect(move |_| {
        let selected_song = selected_songs.get().last().cloned();
        if let Some(selected_song) = selected_song {
            let all_songs = songs.get();
            console_log!("selected {:?}", all_songs.get(selected_song).unwrap());
            last_selected_song.set(all_songs.get(selected_song).cloned());
        } else {
            last_selected_song.set(None);
        }
    });

    let song_details_container = create_node_ref();
    let song_list_container = create_node_ref();

    let ignore__class_list = &[
        "context-menu-root",
        "context-menu-outer",
        "context-menu-item",
        "context-menu-item-text",
        "context-menu-item-icon",
        "context-menu-right-arrow",
    ];
    let ignore_class = move |e: &Event| {
        for item in e.composed_path().iter() {
            let item: web_sys::HtmlElement = item.into();
            let class_list = item.class_list();
            if class_list.is_undefined() || class_list.is_null() {
                continue;
            }

            for ele in class_list.values().into_iter().flatten() {
                if ignore__class_list.contains(&ele.as_string().unwrap_or_default().as_str()) {
                    return true;
                }
            }
        }
        false
    };

    let _ = on_click_outside(song_details_container, move |e| {
        if ignore_class(&e) {
            return;
        }

        let target = event_target::<Node>(&e);
        let song_details_elem: HtmlElement<Div> = song_list_container.get_untracked().unwrap();

        if !song_details_elem.contains(Some(&target)) {
            selected_songs.update(|s| s.clear());
            filtered_selected.update(|s| s.clear());
        }
    });

    let _ = on_click_outside(song_list_container, move |e| {
        if ignore_class(&e) {
            return;
        }

        let target = event_target::<Node>(&e);
        let song_details_elem: HtmlElement<Div> = song_details_container.get_untracked().unwrap();

        if !song_details_elem.contains(Some(&target)) {
            selected_songs.update(|s| s.clear());
            filtered_selected.update(|s| s.clear());
        }
    });

    let ui_store: RwSignal<UiStore> = expect_context();
    let song_context_menu = ContextMenu::new(SongsContextMenu::new(ui_store, song_update_request));

    view! {
        <div
            class="w-100 h-100"
            on:contextmenu=move |ev| {
                song_context_menu.show(ev);
            }
        >
            <div class="container-fluid song-container h-100">
                <div class="row no-gutters h-100 compact-container">
                    <div
                        node_ref=song_details_container
                        style="max-height: 100%; height: fit-content;"
                        class="col-xl-3 col-4"
                    >
                        <SongDetails selected_song=last_selected_song.read_only() icons=icons />
                    </div>
                    <div
                        node_ref=song_list_container
                        class="col-xl-9 col-8 h-100 song-list-compact"
                    >
                        <SongList
                            song_list=songs.read_only()
                            selected_songs_sig=selected_songs
                            filtered_selected=filtered_selected
                        />
                    </div>
                </div>
            </div>

        </div>
    }
}
