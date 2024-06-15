use leptos::{
    component, create_effect, create_signal, view, CollectView, IntoView, ReadSignal, SignalGet,
    SignalSet, View, WriteSignal,
};

use crate::icons::{
    albums_icon::{AlbumsIcon, AlbumsIconProps},
    allsongs_icon::{AllSongsIcon, AllSongsIconProps},
    artists_icon::{ArtistsIcon, ArtistsIconProps},
    explore_icon::{ExploreIcon, ExploreIconProps},
    genres_icon::{GenresIcon, GenresIconProps},
    playlists_icon::{PlaylistsIcon, PlaylistsIconProps},
    queue_icon::{QueueIcon, QueueIconProps},
};

#[derive(Debug, Clone)]
struct Tab {
    pub title: String,
    pub icon: View,
    pub url: String,
}

impl Tab {
    fn new(title: String, icon: impl IntoView, url: String) -> Self {
        Tab {
            title,
            icon: icon.into_view(),
            url
        }
    }
}

#[component]
fn TabItem(
    #[prop()] tab: Tab,
    index: usize,
    active_tab: ReadSignal<usize>,
    set_active_tab: WriteSignal<usize>,
) -> impl IntoView {
    view! {
        <div class="d-flex button-bar" on:click=move |_| set_active_tab.set(index)>
            <div class="d-flex align-items-center icon-transition icon-padding-open w-100">
                <div class="icon">{tab.icon}</div>
                <div
                    class="text-padding"
                    style=move || {
                        if active_tab.get() == index {
                            "color: var(--accent)"
                        } else {
                            "color: var(--textPrimary)"
                        }
                    }
                >

                    {tab.title}
                </div>
            </div>
        </div>
    }
}

#[component]
pub fn Sidebar() -> impl IntoView {
    let mut active_write_signals = vec![];
    let mut active_read_signals = vec![];
    for _ in 0..7 {
        let (read, write) = create_signal(false);
        active_read_signals.push(read);
        active_write_signals.push(write);
    }

    let (active_tab, set_active_tab) = create_signal(1);
    active_write_signals[0].set(true);


    let tabs = [
        Tab::new(
            "Queue".into(),
            QueueIcon(QueueIconProps {
                active: active_read_signals[0],
            }),
            "".into()
        ),
        Tab::new(
            "All Songs".into(),
            AllSongsIcon(AllSongsIconProps {
                active: active_read_signals[1],
            }),
            "/".into()
        ),
        Tab::new(
            "Playlists".into(),
            PlaylistsIcon(PlaylistsIconProps {
                active: active_read_signals[2],
            }),
            "/playlists".into()
        ),
        Tab::new(
            "Artists".into(),
            ArtistsIcon(ArtistsIconProps {
                active: active_read_signals[3],
            }),
            "/artists".into()
        ),
        Tab::new(
            "Albums".into(),
            AlbumsIcon(AlbumsIconProps {
                active: active_read_signals[4],
            }),
            "/albums".into()
        ),
        Tab::new(
            "Genres".into(),
            GenresIcon(GenresIconProps {
                active: active_read_signals[5],
            }),
            "/genres".into()
        ),
        Tab::new(
            "Explore".into(),
            ExploreIcon(ExploreIconProps {
                active: active_read_signals[6],
            }),
            "/explore".into()
        ),
    ];

    let tab_urls: Vec<String> = tabs.iter().map(|v| v.url.clone()).collect();

    let navigate = leptos_router::use_navigate();

    create_effect(move |_| {
        let active_tab = active_tab.get();
        for (i, signal) in active_write_signals.iter().enumerate() {
            signal.set(i == active_tab);
        }
        navigate(tab_urls[active_tab].as_str(), Default::default());
    });

    view! {
        <div class="sidebar-container sidebar">
            <div tabindex="-1" class="b-sidebar-outer">
                <div
                    class="b-sidebar gradient sidebar-top-low-spacing"
                    id="sidebar"
                    tabindex="-1"
                    role="dialog"
                    aria-modal="false"
                    enterclass=""
                    leaveclass="show"
                    style="width: 261px;"
                >
                    <header class="b-sidebar-header">
                        <div class="d-flex w-100 mt-3 justify-content-between"></div>
                    </header>
                    <div class="b-sidebar-body">
                        <div class="extra-margin-top">
                            <div class="d-flex flex-column">

                                {tabs.clone()
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, tab)| {
                                        TabItem(TabItemProps {
                                            tab,
                                            index,
                                            active_tab,
                                            set_active_tab,
                                        })
                                    })
                                    .collect_view()}

                            </div>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
