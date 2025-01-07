use std::rc::Rc;

use leptos::{
    component, create_effect, create_node_ref, create_rw_signal, event_target, expect_context,
    html::Div, view, HtmlElement, IntoView, RwSignal, SignalGet, SignalSet, SignalUpdate,
};
use leptos_context_menu::{ContextMenu, ContextMenuData, ContextMenuItemInner, Menu};
use leptos_use::on_click_outside;
use types::{songs::Song, ui::song_details::DefaultDetails, ui::song_details::SongDetailIcons};
use web_sys::{Event, Node};

use crate::{
    components::{
        songdetails::SongDetails,
        songlist::{ShowProvidersArgs, SongList},
    },
    store::modal_store::{ModalStore, Modals},
    utils::{context_menu::create_context_menu, songs::get_sort_cx_items},
};

struct SongsContextMenu {
    song_update_request: Option<Rc<Box<dyn Fn()>>>,
}

impl SongsContextMenu {
    #[tracing::instrument(level = "trace", skip(song_update_request))]
    pub fn new(song_update_request: Option<Box<dyn Fn()>>) -> Self {
        Self {
            song_update_request: song_update_request.map(Rc::new),
        }
    }

    #[tracing::instrument(level = "trace", skip(self))]
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
    #[tracing::instrument(level = "trace", skip(self))]
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

#[tracing::instrument(
    level = "trace",
    skip(
        songs,
        icons,
        selected_songs,
        song_update_request,
        default_details,
        refresh_cb,
        fetch_next_page
    )
)]
#[component()]
pub fn SongView(
    #[prop()] songs: impl SignalGet<Value = Vec<Song>> + Copy + 'static,
    #[prop()] icons: RwSignal<SongDetailIcons>,
    #[prop()] selected_songs: RwSignal<Vec<usize>>,
    #[prop()] refresh_cb: impl Fn() + 'static,
    #[prop()] fetch_next_page: impl Fn() + 'static,
    #[prop(optional)] song_update_request: Option<Box<dyn Fn()>>,
    #[prop(optional)] default_details: RwSignal<DefaultDetails>,
    #[prop(optional, default=ShowProvidersArgs::default())] providers: ShowProvidersArgs,
) -> impl IntoView {
    let last_selected_song = create_rw_signal(None::<Song>);

    let filtered_selected = create_rw_signal(vec![]);

    create_effect(move |_| {
        let selected_song = selected_songs.get().last().cloned();
        if let Some(selected_song) = selected_song {
            let all_songs = songs.get();
            tracing::debug!("selected {:?}", all_songs.get(selected_song).unwrap());
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

    let song_context_menu = create_context_menu(SongsContextMenu::new(song_update_request));

    view! {
        <div
            class="w-100 h-100"
            on:contextmenu=move |ev| {
                ev.prevent_default();
                song_context_menu.show(ev);
            }
        >
            <div class="container-fluid song-container h-100">
                <div class="row no-gutters h-100 compact-container">
                    <div
                        style="max-height: 100%; height: fit-content;"
                        class="song-details-container col-xl-3 col-4"
                    >
                        <SongDetails
                            buttons_ref=song_details_container
                            default_details=default_details
                            selected_song=last_selected_song.read_only()
                            icons=icons
                        />
                    </div>
                    <div
                        node_ref=song_list_container
                        class="col-xl-9 col-md-8 col h-100 song-list-compact"
                    >
                        <SongList
                            song_list=songs
                            selected_songs_sig=selected_songs
                            filtered_selected=filtered_selected
                            providers=providers
                            refresh_cb=refresh_cb
                            fetch_next_page=fetch_next_page
                        />
                    </div>
                </div>
            </div>

        </div>
    }
}
