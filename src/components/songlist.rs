use std::rc::Rc;

use leptos::{
    component, create_effect, create_memo, create_node_ref, create_read_slice, create_rw_signal,
    create_write_slice,
    ev::{keydown, keyup},
    event_target_value, expect_context,
    html::{Div, Input},
    use_context, view, window_event_listener, HtmlElement, IntoView, ReadSignal, RwSignal, Show,
    SignalGet, SignalSet, SignalUpdate,
};
use leptos_context_menu::{ContextMenu, ContextMenuData, ContextMenuItemInner, ContextMenuItems};
use leptos_router::{use_navigate, NavigateOptions};
use leptos_use::on_click_outside;
use leptos_virtual_scroller::VirtualScroller;
use types::{entities::QueryablePlaylist, songs::Song};

use crate::{
    components::{low_img::LowImg, provider_icon::ProviderIcon},
    console_log,
    icons::{
        add_to_queue_icon::AddToQueueIcon, ellipsis_icon::EllipsisIcon, search_icon::SearchIcon,
        sort_icon::SortIcon,
    },
    store::{
        player_store::PlayerStore,
        ui_store::{SongSortByColumns, UiStore},
    },
    utils::{
        common::{format_duration, get_low_img},
        db_utils::{
            add_songs_to_library, add_to_playlist, get_playlists_by_option, get_playlists_local,
            remove_songs_from_library,
        },
        songs::{get_songs_from_indices, get_sort_cx_items},
    },
};

#[derive(Clone)]
struct SongItemContextMenu {
    current_song: Option<Song>,
    song_list: ReadSignal<Vec<Song>>,
    selected_songs: RwSignal<Vec<usize>>,
    playlists: RwSignal<Vec<QueryablePlaylist>>,
}

impl SongItemContextMenu {
    fn current_or_list(&self) -> Vec<Song> {
        let selected_songs = self.selected_songs.get();
        let ret = if selected_songs.is_empty() {
            if let Some(song) = self.current_song.as_ref() {
                vec![song.clone()]
            } else {
                vec![]
            }
        } else {
            get_songs_from_indices(self.song_list, self.selected_songs)
        };

        console_log!("Got songs {:?}", ret);
        ret
    }

    pub fn play_now(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.play_now_multiple(self.current_or_list()));
    }

    pub fn add_to_queue(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.add_to_queue(self.current_or_list()));
    }

    pub fn play_next(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| store.play_next_multiple(self.current_or_list()));
    }

    pub fn clear_queue_and_play(&self) {
        let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
        player_store.update(|store| {
            store.clear_queue();
            store.play_now_multiple(self.current_or_list())
        });
    }

    pub fn add_to_library(&self) {
        add_songs_to_library(self.current_or_list());
    }

    pub fn remove_from_library(&self) {
        remove_songs_from_library(self.current_or_list());
    }

    pub fn add_to_playlist(&self, id: String) {
        add_to_playlist(id, self.current_or_list());
    }

    pub fn goto_album(&self) {
        let navigate = use_navigate();
        if let Some(song) = &self.current_song {
            if let Some(album) = &song.album {
                if let Some(id) = album.album_id.clone() {
                    navigate(
                        format!("/main/albums/single?id={}", id).as_str(),
                        NavigateOptions::default(),
                    );
                }
            }
        }
    }

    pub fn goto_artist(&self, id: String) {
        let navigate = use_navigate();
        navigate(
            format!("/main/artists/single?id={}", id).as_str(),
            NavigateOptions::default(),
        );
    }
}

impl ContextMenuData<Self> for SongItemContextMenu {
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
        let mut artist_items = vec![];
        if let Some(song) = &self.current_song {
            if let Some(artists) = &song.artists {
                for artist in artists.clone() {
                    let artist_name = artist.artist_name.clone().unwrap_or_default();
                    let artist_id = artist.artist_id.clone().unwrap_or_default();
                    artist_items.push(ContextMenuItemInner::<Self>::new_with_handler(
                        artist_name,
                        move |_, cx| cx.goto_artist(artist_id.clone()),
                        None,
                    ))
                }
            }
        }

        let mut playlist_items = vec![];
        for playlist in self.playlists.get().iter() {
            let playlist_name = playlist.playlist_name.clone();
            let playlist_id = playlist.playlist_id.clone().unwrap_or_default();
            playlist_items.push(ContextMenuItemInner::<Self>::new_with_handler(
                playlist_name,
                move |_, cx| cx.add_to_playlist(playlist_id.clone()),
                None,
            ))
        }

        let library_menu_item = if self
            .current_song
            .clone()
            .map(|s| s.song.library_item.unwrap_or_default())
            .unwrap_or_default()
        {
            ContextMenuItemInner::<Self>::new_with_handler(
                "Remove from library".into(),
                |_, cx| cx.remove_from_library(),
                None,
            )
        } else {
            ContextMenuItemInner::<Self>::new_with_handler(
                "Add to library".into(),
                |_, cx| cx.add_to_library(),
                None,
            )
        };

        vec![
            ContextMenuItemInner::new_with_handler("Play now".into(), |_, cx| cx.play_now(), None),
            ContextMenuItemInner::new_with_handler(
                "Play next".into(),
                |_, cx| cx.play_next(),
                None,
            ),
            ContextMenuItemInner::new_with_handler(
                "Clear queue and play".into(),
                |_, cx| cx.clear_queue_and_play(),
                None,
            ),
            ContextMenuItemInner::new_with_handler(
                "Add to queue".into(),
                |_, cx| cx.add_to_queue(),
                None,
            ),
            ContextMenuItemInner::new("Add to playlist".into(), Some(playlist_items)),
            library_menu_item,
            ContextMenuItemInner::new_with_handler(
                "Goto album".into(),
                |_, cx| cx.goto_album(),
                None,
            ),
            ContextMenuItemInner::new("Goto artists".into(), Some(artist_items)),
        ]
    }
}

struct SortContextMenu {}

impl ContextMenuData<Self> for SortContextMenu {
    fn get_menu_items(&self) -> ContextMenuItems<Self> {
        get_sort_cx_items()
    }
}

#[component()]
pub fn SongListItem(
    #[prop()] song: Song,
    #[prop()] is_selected: Box<dyn Fn() -> bool>,
) -> impl IntoView {
    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let play_now = create_write_slice(player_store, |store, value| store.play_now(value));
    let add_to_queue =
        create_write_slice(player_store, |store, song| store.add_to_queue(vec![song]));
    let song_cloned = song.clone();
    let song_cloned1 = song.clone();
    view! {
        <div class="container-fluid wrapper w-100 mb-3" class:selectedItem=is_selected>
            <div class="row no-gutters align-content-center w-100">
                <LowImg
                    show_eq=|| false
                    eq_playing=|| false
                    cover_img=get_low_img(&song)
                    play_now=move || play_now.set(song_cloned.clone())
                />

                <div class="col-5 align-self-center ml-2">
                    <div class="row no-gutters align-items-center">
                        <div class="col-auto d-flex">
                            <div class="title text-truncate mr-2">
                                {song.song.title.clone().unwrap_or_default()}
                            </div>
                            {move || {
                                let extension = song.song.provider_extension.clone();
                                if let Some(extension) = extension {
                                    view! { <ProviderIcon extension=extension /> }
                                } else {
                                    view! {}.into_view()
                                }
                            }}
                        </div>
                    </div>
                    <div class="row no-gutters flex-nowrap">
                        <div class="subtitle text-truncate">
                            <span>
                                {song
                                    .artists
                                    .unwrap_or_default()
                                    .into_iter()
                                    .map(|a| a.artist_name.unwrap_or_default())
                                    .collect::<Vec<String>>()
                                    .join(", ")}
                            </span>
                        </div>
                    </div>
                </div>

                <div class="col-auto offset-1 align-self-center ml-auto timestamp">
                    {format_duration(song.song.duration.unwrap_or(-1f64))}
                </div>

                <div class="col-auto align-self-center button-icon ml-5">
                    <AddToQueueIcon
                        title="Add to queue".into()
                        on:click=move |_| {
                            add_to_queue.set(song_cloned1.clone());
                        }
                    />
                </div>

                <div class="col-auto align-self-center ml-5 mr-3 py-2 ellipsis-icon">
                    <EllipsisIcon />
                </div>
            </div>
        </div>
    }
}

#[component()]
pub fn SongList(
    #[prop()] song_list: ReadSignal<Vec<Song>>,
    #[prop()] selected_songs_sig: RwSignal<Vec<usize>>,
    #[prop()] filtered_selected: RwSignal<Vec<usize>>,
    #[prop(default = false)] hide_search_bar: bool,
) -> impl IntoView {
    let is_ctrl_pressed = create_rw_signal(false);
    let is_shift_pressed = create_rw_signal(false);

    let show_searchbar = create_rw_signal(false);
    let searchbar_ref = create_node_ref();

    let filter = create_rw_signal(None::<String>);

    let playlists = create_rw_signal(vec![]);
    get_playlists_local(playlists);

    let ui_store: RwSignal<UiStore> = expect_context();
    let songs_sort = create_read_slice(ui_store, |u| u.get_song_sort_by());

    let sorted_songs = create_memo(move |_| {
        let sort = songs_sort.get();
        let mut songs = song_list.get();
        match sort.sort_by {
            SongSortByColumns::Album => songs.sort_by(|a, b| a.album.cmp(&b.album)),
            SongSortByColumns::Artist => songs.sort_by(|a, b| a.artists.cmp(&b.artists)),
            SongSortByColumns::Date => songs.sort_by(|a, b| a.song.date.cmp(&b.song.date)),
            SongSortByColumns::Genre => songs.sort_by(|a, b| a.genre.cmp(&b.genre)),
            SongSortByColumns::PlayCount => {}
            SongSortByColumns::Title => songs.sort_by(|a, b| a.song.title.cmp(&b.song.title)),
        }

        songs
    });

    let filtered_songs = create_memo(move |_| {
        let filter = filter.get();
        if filter.is_none() {
            return sorted_songs.get();
        }
        let binding = filter.unwrap();
        let binding = binding.to_lowercase();
        let filter = binding.as_str();

        sorted_songs
            .get()
            .into_iter()
            .filter(|s| {
                if let Some(title) = &s.song.title {
                    title.to_lowercase().contains(filter)
                } else {
                    false
                }
            })
            .collect()
    });

    create_effect(move |_| {
        let _ = filtered_songs.get();
        filtered_selected.update(|s| s.clear());
        selected_songs_sig.update(|s| s.clear());
    });

    create_effect(move |_| {
        let show_searchbar = show_searchbar.get();
        if show_searchbar {
            if let Some(searchbar) = searchbar_ref.get() {
                (searchbar as HtmlElement<Input>)
                    .focus()
                    .expect("Could not focus on searchbar");
            }
        }
    });

    window_event_listener(keydown, move |ev| {
        if ev.shift_key() {
            is_shift_pressed.set(true);
        }

        if ev.ctrl_key() {
            is_ctrl_pressed.set(true);
        }
    });

    window_event_listener(keyup, move |ev| {
        if ev.key() == "Shift" {
            is_shift_pressed.set(false);
        }

        if ev.key() == "Control" {
            is_ctrl_pressed.set(false);
        }
    });

    let get_actual_position = move |filtered_index: usize| {
        let filtered_songs = filtered_songs.get();
        console_log!("Filtered index {}", filtered_index);
        let filtered_song = filtered_songs.get(filtered_index).unwrap();
        song_list
            .get()
            .iter()
            .position(|s| s == filtered_song)
            .unwrap()
    };

    let add_to_selected = move |index: usize| {
        let is_ctrl_pressed = is_ctrl_pressed.get();
        let is_shift_pressed = is_shift_pressed.get();

        if is_shift_pressed {
            let selected_songs = filtered_selected.get();
            let first_selected = selected_songs.first();

            if let Some(&first_selected) = first_selected {
                let (i, j) = if first_selected < index {
                    (first_selected, index)
                } else {
                    (index, first_selected)
                };

                console_log!("First selected {}, index {}", i, j);

                let mut ret = vec![];
                for k in i..=j {
                    ret.push(get_actual_position(k));
                }

                filtered_selected.set((i..=j).collect());
                selected_songs_sig.set(ret);
            }
            return;
        }

        if is_ctrl_pressed {
            selected_songs_sig.update(move |s| {
                s.push(get_actual_position(index));
            });
            filtered_selected.update(|s| {
                s.push(index);
            });
            return;
        }

        selected_songs_sig.set(vec![get_actual_position(index)]);
        filtered_selected.set(vec![index]);
        console_log!("{:?}", selected_songs_sig.get());
    };

    let context_menu_data = SongItemContextMenu {
        current_song: None,
        song_list,
        selected_songs: selected_songs_sig,
        playlists,
    };
    let song_context_menu = Rc::new(ContextMenu::new(context_menu_data));

    let sort_context_menu = Rc::new(ContextMenu::new(SortContextMenu {}));
    view! {
        <div class="d-flex h-100 w-100">
            <div class="container-fluid">

                <Show
                    when=move || hide_search_bar
                    fallback=move || {
                        let sort_context_menu = sort_context_menu.clone();
                        view! {
                            <div class="container-fluid tab-carousel">
                                <div class="row no-gutters">
                                    <div class="col song-header-options w-100">
                                        <div class="row no-gutters align-items-center h-100">
                                            // Sort icons here
                                            <div class="col-auto ml-auto d-flex">

                                                {move || {
                                                    if show_searchbar.get() {
                                                        view! {
                                                            <div class="songlist-searchbar-container mr-3">
                                                                <input
                                                                    ref=searchbar_ref
                                                                    on:input=move |ev| {
                                                                        let text = event_target_value(&ev);
                                                                        if text.is_empty() {
                                                                            filter.set(None);
                                                                        } else {
                                                                            filter.set(Some(text));
                                                                        }
                                                                    }

                                                                    type="text"
                                                                    class="songlist-searchbar"
                                                                    placeholder="search"
                                                                />
                                                            </div>
                                                        }
                                                            .into_view()
                                                    } else {
                                                        view! {}.into_view()
                                                    }
                                                }}
                                                <div
                                                    class="mr-3 align-self-center"
                                                    on:click=move |_| show_searchbar.set(!show_searchbar.get())
                                                >
                                                    <SearchIcon accent=false />
                                                </div>
                                                <div
                                                    class="align-self-center"
                                                    on:click=move |e| { sort_context_menu.show(e) }
                                                >
                                                    <SortIcon />
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        }
                    }
                >
                    <div></div>
                </Show>

                <div class="row no-gutters h-100">
                    <div
                        class="scroller w-100 full-height"
                        style="height: calc(100% - 53px) !important;"
                    >

                        <VirtualScroller
                            each=filtered_songs
                            item_height=95usize
                            inner_el_style="width: calc(100% - 15px);"
                            children=move |(index, song)| {
                                let song = song.clone();
                                let song_context_menu = song_context_menu.clone();
                                view! {
                                    <SongListItem
                                        on:contextmenu=move |ev| {
                                            ev.stop_propagation();
                                            let mut data = song_context_menu.get_data();
                                            data.current_song = Some(song.clone());
                                            drop(data);
                                            song_context_menu.show(ev);
                                        }
                                        on:click=move |_| add_to_selected(index)
                                        is_selected=Box::new(move || {
                                            filtered_selected.get().contains(&index)
                                        })

                                        song=song.clone()
                                    />
                                }
                            }
                        />

                    </div>
                </div>
            </div>
        </div>
    }
}
