use leptos::{
    component, create_effect, create_node_ref, create_rw_signal, create_signal, create_write_slice,
    ev::{keydown, keypress, keyup},
    event_target_value,
    html::Input,
    spawn_local, use_context, view, window_event_listener, CollectView, ErrorBoundary, For,
    HtmlElement, IntoView, ReadSignal, RwSignal, SignalGet, SignalSet, SignalUpdate, WriteSignal,
};
use types::songs::{GetSongOptions, Song};

use crate::{
    console_log,
    icons::{
        add_to_library_icon::AddToLibraryIcon, add_to_queue_icon::AddToQueueIcon,
        ellipsis_icon::EllipsisIcon, fetch_all_icon::FetchAllIcon, plain_play_icon::PlainPlayIcon,
        play_hover_icon::PlayHoverIcon, random_icon::RandomIcon, search_icon::SearchIcon,
        song_default_icon::SongDefaultIcon, sort_icon::SortIcon,
    },
    store::player_store::PlayerStore,
    utils::{common::format_duration, db_utils::get_songs_by_option},
};

#[component()]
pub fn SongDetails(#[prop()] selected_songs: ReadSignal<Option<Song>>) -> impl IntoView {
    let selected_title = create_rw_signal(None::<String>);
    let selected_artists = create_rw_signal(None::<String>);
    let selected_duration = create_rw_signal(None::<String>);
    let selected_cover_path = create_rw_signal(None::<String>);

    let show_default_cover_img = create_rw_signal(true);

    create_effect(move |_| {
        let selected_song = selected_songs.get();
        selected_title.set(
            selected_song
                .clone()
                .map(|s| s.song.clone().title.unwrap_or_default()),
        );
        selected_artists.set(Some(
            selected_song
                .as_ref()
                .map(|s| s.clone().artists.unwrap_or_default())
                .unwrap_or_default()
                .iter()
                .map(|a| a.artist_name.clone().unwrap_or_default())
                .collect::<Vec<String>>()
                .join(", "),
        ));

        selected_duration.set(
            selected_song
                .clone()
                .map(|s| format_duration(s.song.duration.unwrap_or(-1f64))),
        );

        let cover_path = selected_song.and_then(|s| s.song.song_cover_path_high.clone());
        selected_cover_path.set(cover_path.clone());

        if cover_path.is_none() {
            show_default_cover_img.set(true)
        } else {
            show_default_cover_img.set(false)
        }
    });

    view! {
        <div class="col-xl-3 col-4 h-100">
            <div class="container-fluid h-100 scrollable">
                <div class="row no-gutters">
                    <div class="col position-relative">

                        <div class="image-container w-100">
                            <div class="embed-responsive embed-responsive-1by1">
                                <div class="embed-responsive-item albumart">

                                    {move || {
                                        if !show_default_cover_img.get() {
                                            view! {
                                                <img
                                                    src=move || selected_cover_path.get()
                                                    on:error=move |_| { show_default_cover_img.set(true) }
                                                />
                                            }
                                                .into_view()
                                        } else {
                                            view! {
                                                <SongDefaultIcon class="fade-in-image".to_string()/>
                                            }
                                                .into_view()
                                        }
                                    }}

                                </div>
                            </div>
                        </div>

                        <div class="song-info-container">
                            <div class="row d-flex">
                                <div class="col song-title-details text-truncate">
                                    {move || selected_title.get()}

                                </div>
                            </div>

                            <div class="song-subtitle-details text-truncate">
                                {move || selected_artists.get()}

                            </div>

                            <div class="song-timestamp-details">

                                {move || selected_duration.get()}

                            </div>
                        </div>
                    </div>
                </div>

                <div class="row no-gutters flex-fill mt-2">
                    <div class="col">
                        <div class="button-group d-flex">

                            {move || {
                                let title = selected_title.get();
                                if let Some(title) = title {
                                    view! {
                                        <PlainPlayIcon title=title.clone()/>
                                        <AddToQueueIcon title=title.clone()/>
                                        <AddToLibraryIcon title=title/>
                                    }
                                        .into_view()
                                } else {
                                    view! {}.into_view()
                                }
                            }}
                            <RandomIcon/> <FetchAllIcon/>
                        </div>
                    </div>
                </div>

            </div>
        </div>
    }
}

#[component()]
pub fn SongListItem(
    #[prop()] song: Song,
    #[prop()] is_selected: Box<dyn Fn() -> bool>,
) -> impl IntoView {
    let show_default_cover_img = create_rw_signal(false);
    let (cover_path, _) = create_signal(song.clone().song.song_cover_path_low.unwrap_or_default());
    let (provider_icon, _) = create_signal(song.clone().song.provider_extension);

    let player_store = use_context::<RwSignal<PlayerStore>>().unwrap();
    let play_now = create_write_slice(player_store, |store, value| store.play_now(value));

    let song_cloned = song.clone();

    view! {
        <div class="container-fluid wrapper w-100 mb-3" class:selectedItem=is_selected>
            <div class="row no-gutters align-content-center w-100">
                <div class="img-container justify-content-around ms-auto coverimg me-auto d-flex align-items-center">

                    {move || {
                        if !show_default_cover_img.get() {
                            view! {
                                <img
                                    class="fade-in-image"
                                    src=move || cover_path.get()
                                    on:error=move |_| { show_default_cover_img.set(true) }
                                />
                            }
                                .into_view()
                        } else {
                            console_log!("Rendering default");
                            view! { <SongDefaultIcon/> }.into_view()
                        }
                    }}
                    <div
                        class="play-button-song-list d-flex justify-content-center"
                        on:click=move |_| play_now.set(song_cloned.clone())
                    >
                        <PlayHoverIcon/>
                    </div>
                </div>

                <div class="col-5 align-self-center ml-2">
                    <div class="row no-gutters align-items-center">
                        <div class="col-auto d-flex">
                            <div class="title text-truncate mr-2">
                                {song.song.title.unwrap_or_default()}
                            </div>

                            {move || {
                                if provider_icon.get().is_some() {
                                    view! { <img src=""/> }.into_view()
                                } else {
                                    view! {}.into_view()
                                }
                            }}

                            <div class="d-flex provider-icon"></div>
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

                <div class="col-auto offset-1 align-self-center ml-auto timestamp">00:00</div>

                <div class="col-auto align-self-center button-icon ml-5">
                    <AddToQueueIcon title="test".to_string()/>
                </div>

                <div class="col-auto align-self-center ml-5 mr-3 py-2 ellipsis-icon">
                    <EllipsisIcon/>
                </div>
            </div>
        </div>
    }
}

#[component()]
pub fn SongList(
    #[prop()] song_list: ReadSignal<Vec<Song>>,
    #[prop()] selected_songs_sig: RwSignal<Vec<usize>>,
) -> impl IntoView {
    let is_ctrl_pressed = create_rw_signal(false);
    let is_shift_pressed = create_rw_signal(false);

    let show_searchbar = create_rw_signal(false);
    let searchbar_ref = create_node_ref();

    let filter = create_rw_signal(None::<String>);

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

    let add_to_selected = move |index: usize| {
        let is_ctrl_pressed = is_ctrl_pressed.get();
        let is_shift_pressed = is_shift_pressed.get();

        if is_shift_pressed {
            let selected_songs = selected_songs_sig.get();
            let first_selected = selected_songs.first();

            if let Some(first_selected) = first_selected.cloned() {
                let (i, j) = if first_selected < index {
                    (first_selected, index)
                } else {
                    (index, first_selected)
                };
                selected_songs_sig.set((i..=j).collect::<Vec<usize>>());
            }
            return;
        }

        if is_ctrl_pressed {
            selected_songs_sig.update(move |s| {
                s.push(index);
            });
            return;
        }

        selected_songs_sig.set(vec![index]);
    };

    view! {
        <div class="col-xl-9 col-8 h-100 song-list-compact">
            <div class="d-flex h-100 w-100">
                <div class="container-fluid">
                    <div class="container-fluid tab-carousel">
                        <div class="row no-gutters">
                            <div class="col song-header-options w-100">
                                <div class="row no-gutters align-items-center h-100">
                                    // Sort icons here
                                    <div class="col-auto ml-auto d-flex">

                                        {move || {
                                            if show_searchbar.get() {
                                                view! {
                                                    <div class="searchbar-container mr-3">
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
                                                            class="searchbar"
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
                                            <SearchIcon accent=false/>
                                        </div> <div class="align-self-center">
                                            <SortIcon/>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    </div>

                    <div class="row no-gutters h-100">
                        <div
                            class="scroller w-100 full-height"
                            style="height: calc(100% - 53px) !important;"
                        >

                            {move || {
                                song_list
                                    .get()
                                    .iter()
                                    .enumerate()
                                    .map(|(index, song)| {
                                        let title = song.song.title.clone();
                                        if title.is_none() {
                                            return view! {}.into_view();
                                        }
                                        if let Some(filter) = filter.get() {
                                            if !title.unwrap().contains(&filter) {
                                                return view! {}.into_view();
                                            }
                                        }
                                        view! {
                                            <SongListItem
                                                on:click=move |_| add_to_selected(index)
                                                is_selected=Box::new(move || {
                                                    selected_songs_sig.get().contains(&index)
                                                })

                                                song=song.clone()
                                            />
                                        }
                                    })
                                    .collect_view()
                            }}

                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

#[component()]
pub fn AllSongs() -> impl IntoView {
    console_log!("Loading all songs");
    let songs = create_rw_signal(vec![]);
    get_songs_by_option(GetSongOptions::default(), songs);

    let selected_songs: RwSignal<Vec<usize>> = create_rw_signal(vec![]);
    let last_selected_song = create_rw_signal(None::<Song>);

    create_effect(move |_| {
        console_log!("{:?}", selected_songs.get());
        let selected_song = selected_songs.get().last().cloned();
        if let Some(selected_song) = selected_song {
            let all_songs = songs.get();
            last_selected_song.set(all_songs.get(selected_song).cloned());
        }
    });

    spawn_local(async move {
        get_songs_by_option(GetSongOptions::default(), songs);
    });

    view! {
        <div class="w-100 h-100">
            <div class="container-fluid song-container h-100">
                <div class="row no-gutters h-100 compact-container">
                    <SongDetails selected_songs=last_selected_song.read_only()/>
                    <SongList song_list=songs.read_only() selected_songs_sig=selected_songs/>
                </div>
            </div>

        </div>
    }
}
