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

use ev::{mousedown, mousemove, mouseup, touchend, touchmove, touchstart};
use html::Div;
use leptos::*;
use leptos::{IntoView, component, prelude::*, view};
use leptos_use::{UseEventListenerOptions, use_event_listener, use_event_listener_with_options};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::components::musicbar_components::{Controls, Details, ExtraControls, Slider};
use crate::components::musicinfo::{MusicInfo, MusicInfoMobile};
use crate::store::ui_store::UiStore;

const MUSICBAR_HEIGHT: i32 = 50 + 16;
const SIDEBAR_HEIGHT: i32 = 80;

fn transition_closed(musicbar: NodeRef<Div>, musicinfo: NodeRef<Div>) {
    if let Some(musicbar) = musicbar.get_untracked() {
        let _ = musicbar.style(("transition", "all 0.2s"));
        let _ = musicbar.style(("transform", "translateY(0px)"));
    }

    if let Some(musicinfo) = musicinfo.get_untracked() {
        let _ = musicinfo.style(("transition", "all 0.2s"));
        let _ = musicinfo.style((
            "transform",
            format!("translateY(calc(100vh - {MUSICBAR_HEIGHT}px))"),
        ));
    }

    let sidebar = document()
        .get_element_by_id("sidebar")
        .map(|e| e.dyn_into::<HtmlElement>().unwrap());
    if let Some(sidebar) = sidebar {
        let style = sidebar.style();
        style.set_property("visibility", "visible").unwrap();
        style.set_property("transition", "all 0.2s").unwrap();
        style.set_property("opacity", "1").unwrap();
    }
}

fn transition_opened(musicbar: NodeRef<Div>, musicinfo: NodeRef<Div>) {
    if let Some(musicbar) = musicbar.get() {
        let _ = musicbar.style(("transition", "all 0.2s"));
        let _ = musicbar.style((
            "transform",
            format!("translateY(calc(-100vh + {MUSICBAR_HEIGHT}px))"),
        ));
    }

    if let Some(musicinfo) = musicinfo.get() {
        let _ = musicinfo.style(("transition", "all 0.2s"));
        let _ = musicinfo.style(("transform", "translateY(0px)"));
    }
    let sidebar = document()
        .get_element_by_id("sidebar")
        .map(|e| e.dyn_into::<HtmlElement>().unwrap());
    if let Some(sidebar) = sidebar {
        let style = sidebar.style();
        style.set_property("transition", "all 0.2s").unwrap();
        style.set_property("opacity", "0").unwrap();
        style.set_property("visibility", "hidden").unwrap();
    }
}

fn musicinfo_drag(musicbar: NodeRef<Div>, musicinfo: NodeRef<Div>) {
    let is_dragging = RwSignal::new(false);
    let start_offset = RwSignal::new(0);
    let page_height = RwSignal::new(0);
    let has_moved = RwSignal::new(false);

    let listener = move |client_y: i32| {
        start_offset.set(client_y);
        page_height.set(document().body().unwrap().client_height() - SIDEBAR_HEIGHT);
        is_dragging.set(true);
        has_moved.set(false);

        let sidebar = document()
            .get_element_by_id("sidebar")
            .map(|e| e.dyn_into::<HtmlElement>().unwrap());
        if let Some(sidebar_elem) = sidebar {
            let style = sidebar_elem.style();
            style.set_property("transition", "all 0s").unwrap();
            style.set_property("display", "flex").unwrap();
        }

        if let Some(musicbar) = musicbar.get() {
            let _ = musicbar.style(("transition", "all 0s"));
        }

        if let Some(musicinfo) = musicinfo.get() {
            let _ = musicinfo.style(("transition", "all 0s"));
        }
    };
    let _ = use_event_listener(musicinfo, mousedown, move |ev| {
        ev.stop_propagation();
        listener(ev.client_y())
    });

    let _ = use_event_listener(musicinfo, touchstart, move |ev| {
        ev.stop_propagation();
        let touch = ev.touches().item(0);
        if let Some(touch) = touch {
            listener(touch.client_y())
        }
    });

    let listener = move || {
        if is_dragging.get_untracked() {
            is_dragging.set(false);

            if has_moved.get_untracked() {
                transition_closed(musicbar, musicinfo);
            } else {
                transition_opened(musicbar, musicinfo);
            }
        }
    };
    let _ = use_event_listener(document().body(), mouseup, move |_| {
        listener();
    });
    let _ = use_event_listener(document().body(), touchend, move |_| {
        listener();
    });

    let listener = move |client_y: i32| {
        if is_dragging.get_untracked()
            && let Some(musicinfo) = musicinfo.get_untracked()
            && let Some(musicbar) = musicbar.get_untracked()
        {
            let start_offset = start_offset.get_untracked();
            let page_height = page_height.get_untracked();
            let client_y_diff = (client_y - start_offset).clamp(0, page_height);
            let musicbar_pos = client_y_diff - page_height;
            let _ = musicinfo.style(("transform", format!("translateY({client_y_diff}px)")));
            let _ = musicbar.style(("transform", format!("translateY({musicbar_pos}px)")));

            let sidebar = document()
                .get_element_by_id("sidebar")
                .map(|e| e.dyn_into::<HtmlElement>().unwrap());
            if let Some(sidebar) = sidebar {
                let style = sidebar.style();
                let opacity = client_y_diff as f64 / start_offset as f64;
                style
                    .set_property("opacity", &format!("{}", opacity.abs()))
                    .unwrap();
            }

            if client_y_diff.abs() > (0.2 * page_height as f64) as i32 {
                has_moved.set(true);
            }
        }
    };
    let options = UseEventListenerOptions::default().passive::<bool>(Some(true));
    let _ = use_event_listener_with_options(
        document().body(),
        mousemove,
        move |ev| listener(ev.client_y()),
        options,
    );
    let _ = use_event_listener_with_options(
        document().body(),
        touchmove,
        move |ev| {
            let touch = ev.touches().item(0);
            if let Some(touch) = touch {
                listener(touch.client_y())
            }
        },
        options,
    );
}

fn musicbar_drag(musicbar: NodeRef<Div>, musicinfo: NodeRef<Div>) {
    let is_dragging = RwSignal::new(false);
    let start_offset = RwSignal::new(0);
    let page_height = RwSignal::new(0);
    let has_moved = RwSignal::new(false);

    let listener = move |client_y: i32| {
        start_offset.set(client_y);
        page_height.set(document().body().unwrap().client_height() - SIDEBAR_HEIGHT);
        is_dragging.set(true);
        has_moved.set(false);

        let sidebar_elem = document()
            .get_element_by_id("sidebar")
            .map(|e| e.dyn_into::<HtmlElement>().unwrap());
        if let Some(sidebar_elem) = sidebar_elem {
            let style = sidebar_elem.style();
            style.set_property("transition", "all 0s").unwrap();
        }

        if let Some(musicbar) = musicbar.get_untracked() {
            let _ = musicbar.style(("transition", "all 0s"));
        }

        if let Some(musicinfo) = musicinfo.get_untracked() {
            let _ = musicinfo.style(("transition", "all 0s"));
        }
    };
    let _ = use_event_listener(musicbar, mousedown, move |ev| {
        ev.stop_propagation();
        listener(ev.client_y())
    });

    let _ = use_event_listener(musicbar, touchstart, move |ev| {
        ev.stop_propagation();
        let touch = ev.touches().item(0);
        if let Some(touch) = touch {
            listener(touch.client_y())
        }
    });

    let listener = move || {
        if is_dragging.get_untracked() {
            is_dragging.set(false);

            if has_moved.get_untracked() {
                transition_opened(musicbar, musicinfo);
            } else {
                transition_closed(musicbar, musicinfo);
            }
        }
    };
    let _ = use_event_listener(document().body(), mouseup, move |_| {
        listener();
    });
    let _ = use_event_listener(document().body(), touchend, move |_| {
        listener();
    });

    let listener = move |client_y: i32| {
        if is_dragging.get_untracked()
            && let Some(musicinfo) = musicinfo.get_untracked()
            && let Some(musicbar) = musicbar.get_untracked()
        {
            let page_height = page_height.get_untracked();
            let start_offset = start_offset.get_untracked();
            let client_y_diff =
                (client_y - start_offset).clamp(-(page_height - MUSICBAR_HEIGHT), 0);
            let musicinfo_pos = page_height + client_y_diff;

            let _ = musicinfo.style(("transform", format!("translateY({musicinfo_pos}px)")));
            let _ = musicbar.style(("transform", format!("translateY({client_y_diff}px)")));

            let sidebar = document()
                .get_element_by_id("sidebar")
                .map(|e| e.dyn_into::<HtmlElement>().unwrap());
            if let Some(sidebar) = sidebar {
                let style = sidebar.style();
                let opacity = client_y as f64 / start_offset as f64;
                style
                    .set_property("opacity", &format!("{opacity}"))
                    .unwrap();
            }

            if client_y_diff.abs() > (0.2 * page_height as f64) as i32 {
                has_moved.set(true);
            }
        }
    };

    let _ = use_event_listener(document().body(), mousemove, move |ev| {
        listener(ev.client_y())
    });
    let _ = use_event_listener(document().body(), touchmove, move |ev| {
        let touch = ev.touches().item(0);
        if let Some(touch) = touch {
            listener(touch.client_y())
        }
    });
}

#[tracing::instrument(level = "debug")]
#[component]
pub fn MusicBar() -> impl IntoView {
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let show_musicinfo = create_read_slice(ui_store, move |s| s.get_show_queue());

    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

    let musicinfo_ref = NodeRef::new();
    let musicbar_ref = NodeRef::new();

    if is_mobile {
        musicbar_drag(musicbar_ref, musicinfo_ref);
        musicinfo_drag(musicbar_ref, musicinfo_ref);
    }

    view! {
        <div class="musicbar-content d-flex" class:musicbar-content-mobile=is_mobile>
            {move || {
                if is_mobile {
                    view! { <MusicInfoMobile show=show_musicinfo node_ref=musicinfo_ref /> }
                        .into_any()
                } else {
                    view! { <MusicInfo show=show_musicinfo node_ref=musicinfo_ref /> }.into_any()
                }
            }}
            <div
                class="musicbar-background w-100"
                class:musicinfo-show=show_musicinfo
                node_ref=musicbar_ref
            >
                <div class="musicbar h-100">
                    <Slider />
                    <div class="container-fluid d-flex bar-container h-100 pb-2">
                        <div class="row no-gutters align-items-center justify-content-center align-content-center no-gutters w-100 control-row justify-content-between">
                            <div class="col-4 no-gutters details-col w-100">
                                <Details />
                            </div>

                            <div class="col align-self-center no-gutters controls-col">
                                <Controls />
                            </div>
                            <div class="col-lg-auto col-1 align-self-center no-gutters extra-col">
                                <ExtraControls />
                            </div>
                        </div>
                    </div>
                </div>
            </div>

        </div>
    }
}
