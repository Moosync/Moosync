use leptos::{
    component, create_effect, create_read_slice, create_rw_signal, create_signal, create_slice,
    expect_context, view, CollectView, IntoView, ReadSignal, RwSignal, Show, SignalGet,
    SignalGetUntracked, SignalSet, SignalUpdate, View, WriteSignal,
};

use crate::{
    icons::{
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
        sidebar_toggle_icon::SidebarToggleIcon,
        system_icon::{SystemIcon, SystemIconProps},
        themes_icon::{ThemesIcon, ThemesIconProps},
    },
    store::ui_store::UiStore,
};

#[derive(Debug, Clone)]
pub struct Tab {
    pub title: String,
    pub icon: fn(signal: ReadSignal<bool>) -> View,
    pub url: String,
}

impl Tab {
    #[tracing::instrument(level = "trace", skip(title, icon, url))]
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
            _ => panic!("Icon not found: {}", icon),
        };
        Tab {
            title: title.to_string(),
            icon,
            url: url.to_string(),
        }
    }
}

#[tracing::instrument(
    level = "trace",
    skip(tab, index, active_tab, active_tab_icon_signal, set_active_tab)
)]
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
            id=tab.title.clone()
            class:button-active=move || active_tab.get() == index
            on:click=move |_| set_active_tab.set(index)
        >
            <Show when=move || active_tab.get() == index>
                <div class="whitebar whitebar-active"></div>
            </Show>
            <div class="d-flex align-items-center icon-transition icon-padding-open">
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

#[tracing::instrument(level = "trace", skip(tabs, show_back))]
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

    let ui_store = expect_context::<RwSignal<UiStore>>();

    let (sidebar_open, set_sidebar_open) = create_slice(
        ui_store,
        |u| u.get_sidebar_open(),
        |u, val| u.set_sidebar_open(val),
    );

    create_effect(move |_| {
        let active_tab = active_tab.get();
        for (i, signal) in active_write_signals.iter().enumerate() {
            signal.set(i == active_tab);
        }
        let url = tab_urls.get(active_tab);
        if let Some(url) = url {
            if url == "queue" {
                ui_store.update(|s| s.show_queue(true));
            } else {
                navigate(url.as_str(), Default::default());
            }
        }
    });

    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

    view! {
        <div class="sidebar-container sidebar" class:sidebar-mobile=is_mobile>
            <div tabindex="-1" class="b-sidebar-outer">
                <div
                    class="b-sidebar gradient sidebar-top-low-spacing"
                    id="sidebar"
                    tabindex="-1"
                    role="dialog"
                    aria-modal="false"
                    enterclass=""
                    leaveclass="show"
                    style:width=move || if sidebar_open.get() { "261px" } else { "70px" }
                >
                    <header class="b-sidebar-header">
                        <div class="d-flex w-100 mt-3 justify-content-between"></div>
                    </header>
                    <div class="b-sidebar-body">
                        <div class="extra-margin-top">
                            <div
                                class="d-flex mr-4 mb-3 sidebar-header"
                                style="justify-content: space-between;"
                            >
                                <div class="icon-padding-open d-flex">
                                    <Show when=move || show_back fallback=|| view! {}>
                                        <PrevIcon on:click=move |_| {
                                            let navigate = leptos_router::use_navigate();
                                            navigate("/main/allsongs", Default::default());
                                        } />
                                    </Show>
                                </div>
                                <div class="sidebar-toggle-icon d-flex justify-content-end align-self-center mt-2">
                                    <SidebarToggleIcon on:click=move |_| {
                                        set_sidebar_open.set(!sidebar_open.get_untracked());
                                    } />
                                </div>
                            </div>
                            <div class="d-flex tabs-holder">

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
