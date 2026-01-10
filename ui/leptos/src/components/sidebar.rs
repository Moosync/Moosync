// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use leptos::{component, prelude::*, view, IntoView};

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
        queue_icon::{QueueIcon, QueueIconProps},
        sidebar_toggle_icon::SidebarToggleIcon,
        system_icon::{SystemIcon, SystemIconProps},
        themes_icon::{ThemesIcon, ThemesIconProps},
    },
    store::ui_store::UiStore,
};

#[derive(Debug, Clone)]
pub struct Tab {
    pub title: fn() -> &'static str,
    pub icon: fn(signal: ReadSignal<bool>) -> AnyView,
    pub url: String,
}

impl Tab {
    #[tracing::instrument(level = "debug", skip(title, icon, url))]
    pub fn new(title: fn() -> &'static str, icon: &str, url: &str) -> Self {
        let icon = match icon {
            "Queue" => |active| QueueIcon(QueueIconProps { active }).into_any(),
            "Songs" | "Home" => |active| AllSongsIcon(AllSongsIconProps { active }).into_any(),
            "Playlists" => |active| PlaylistsIcon(PlaylistsIconProps { active }).into_any(),
            "Artists" => |active| ArtistsIcon(ArtistsIconProps { active }).into_any(),
            "Albums" => |active| AlbumsIcon(AlbumsIconProps { active }).into_any(),
            "Genres" => |active| GenresIcon(GenresIconProps { active }).into_any(),
            "Explore" => |active| ExploreIcon(ExploreIconProps { active }).into_any(),
            "Paths" => |active| PathsIcon(PathsIconProps { active }).into_any(),
            "System" => |active| SystemIcon(SystemIconProps { active }).into_any(),
            "Logs" => |active| LogsIcon(LogsIconProps { active }).into_any(),
            "Extensions" => |active| ExtensionsIcon(ExtensionsIconProps { active }).into_any(),
            "Themes" => |active| ThemesIcon(ThemesIconProps { active }).into_any(),
            _ => panic!("Icon not found: {icon}"),
        };
        Tab {
            title,
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
            id=tab.title
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

#[tracing::instrument(level = "debug", skip(tabs))]
#[component]
pub fn Sidebar(#[prop()] tabs: Vec<Tab>) -> impl IntoView {
    let mut active_write_signals = vec![];
    let mut active_read_signals = vec![];
    for _ in 0..tabs.len() {
        let (read, write) = signal(false);
        active_read_signals.push(read);
        active_write_signals.push(write);
    }

    let (active_tab, set_active_tab) = signal(1);
    active_write_signals[0].set(true);

    let tab_urls: Vec<String> = tabs.iter().map(|v| v.url.clone()).collect();

    let navigate = leptos_router::hooks::use_navigate();

    let ui_store = expect_context::<RwSignal<UiStore>>();

    let (sidebar_open, set_sidebar_open) = create_slice(
        ui_store,
        |u| u.get_sidebar_open(),
        |u, val| u.set_sidebar_open(val),
    );

    Effect::new(move || {
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
                                <div class="sidebar-toggle-icon d-flex justify-content-end align-self-center mt-2 w-100">
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
