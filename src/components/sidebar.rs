use leptos::{
    component, create_effect, create_signal, view, CollectView, IntoView, ReadSignal, Show,
    SignalGet, SignalSet, View, WriteSignal,
};

use crate::icons::{
    albums_icon::{AlbumsIcon, AlbumsIconProps},
    allsongs_icon::{AllSongsIcon, AllSongsIconProps},
    artists_icon::{ArtistsIcon, ArtistsIconProps},
    explore_icon::{ExploreIcon, ExploreIconProps},
    extensions_icon::{ExtensionsIcon, ExtensionsIconProps},
    genres_icon::{GenresIcon, GenresIconProps},
    logs_icon::{LogsIcon, LogsIconProps},
    paths_icon::{PathsIcon, PathsIconProps},
    playlists_icon::{PlaylistsIcon, PlaylistsIconProps},
    prev_icon::PrevIcon,
    queue_icon::{QueueIcon, QueueIconProps},
    system_icon::{SystemIcon, SystemIconProps},
    themes_icon::{ThemesIcon, ThemesIconProps},
};

#[derive(Debug, Clone)]
pub struct Tab {
    pub title: String,
    pub icon: fn(signal: ReadSignal<bool>) -> View,
    pub url: String,
}

impl Tab {
    pub fn new(title: &str, icon: &str, url: &str) -> Self {
        let icon = match icon {
            "Queue" => |active| QueueIcon(QueueIconProps { active }).into_view(),
            "All Songs" => |active| AllSongsIcon(AllSongsIconProps { active }).into_view(),
            "Playlists" => |active| PlaylistsIcon(PlaylistsIconProps { active }).into_view(),
            "Artists" => |active| ArtistsIcon(ArtistsIconProps { active }).into_view(),
            "Albums" => |active| AlbumsIcon(AlbumsIconProps { active }).into_view(),
            "Genres" => |active| GenresIcon(GenresIconProps { active }).into_view(),
            "Explore" => |active| ExploreIcon(ExploreIconProps { active }).into_view(),
            "Paths" => |active| PathsIcon(PathsIconProps { active }).into_view(),
            "System" => |active| SystemIcon(SystemIconProps { active }).into_view(),
            "Logs" => |active| LogsIcon(LogsIconProps { active }).into_view(),
            "Extensions" => |active| ExtensionsIcon(ExtensionsIconProps { active }).into_view(),
            "Themes" => |active| ThemesIcon(ThemesIconProps { active }).into_view(),

            _ => todo!("Icon not found: {}", icon),
        };
        Tab {
            title: title.to_string(),
            icon,
            url: url.to_string(),
        }
    }
}

#[component]
fn TabItem(
    #[prop()] tab: Tab,
    index: usize,
    active_tab: ReadSignal<usize>,
    active_tab_icon_signal: ReadSignal<bool>,
    set_active_tab: WriteSignal<usize>,
) -> impl IntoView {
    view! {
        <div
            class="d-flex button-bar"
            class:button-active=move || active_tab.get() == index
            on:click=move |_| set_active_tab.set(index)
        >
            <Show when=move || active_tab.get() == index>
                <div class="whitebar whitebar-active"></div>
            </Show>
            <div class="d-flex align-items-center icon-transition icon-padding-open w-100">
                <div class="icon">{move || { (tab.icon)(active_tab_icon_signal) }}</div>
                <div
                    class="text-padding"
                    style:color=move || {
                        if active_tab.get() == index {
                            "var(--accent)"
                        } else {
                            "var(--textPrimary)"
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
pub fn Sidebar(
    #[prop()] tabs: Vec<Tab>,
    #[prop(optional = true, default = false)] show_back: bool,
) -> impl IntoView {
    let mut active_write_signals = vec![];
    let mut active_read_signals = vec![];
    for _ in 0..tabs.len() {
        let (read, write) = create_signal(false);
        active_read_signals.push(read);
        active_write_signals.push(write);
    }

    let (active_tab, set_active_tab) = create_signal(1);
    active_write_signals[0].set(true);

    let tab_urls: Vec<String> = tabs.iter().map(|v| v.url.clone()).collect();

    let navigate = leptos_router::use_navigate();

    create_effect(move |_| {
        let active_tab = active_tab.get();
        for (i, signal) in active_write_signals.iter().enumerate() {
            signal.set(i == active_tab);
        }
        let url = tab_urls.get(active_tab);
        if let Some(url) = url {
            navigate(url.as_str(), Default::default());
        }
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
                            <Show when=move || show_back fallback=|| view! {}>
                                <div class="icon-padding-open d-flex">
                                    <PrevIcon on:click=move |_| {
                                        let navigate = leptos_router::use_navigate();
                                        navigate("/main/allsongs", Default::default());
                                    } />
                                </div>
                            </Show>
                            <div class="d-flex flex-column">

                                {tabs
                                    .into_iter()
                                    .enumerate()
                                    .map(|(index, tab)| {
                                        TabItem(TabItemProps {
                                            tab: tab.clone(),
                                            index,
                                            active_tab,
                                            active_tab_icon_signal: *active_read_signals
                                                .get(index)
                                                .unwrap(),
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
