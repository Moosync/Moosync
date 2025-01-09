use ev::{mousedown, mousemove, mouseup, touchend, touchmove, touchstart, transitionend};
use html::Div;
use leptos::*;
use leptos::{component, view, IntoView, RwSignal, SignalGet, SignalSet};
use leptos_use::{use_event_listener, use_event_listener_with_options, UseEventListenerOptions};
use wasm_bindgen::JsCast;
use web_sys::HtmlElement;

use crate::components::musicbar_components::{Controls, Details, ExtraControls, Slider};
use crate::components::musicinfo::{MusicInfo, MusicInfoMobile};
use crate::store::ui_store::UiStore;

const MUSICBAR_HEIGHT: i32 = 50 + 16;
const SIDEBAR_HEIGHT: i32 = 80;

fn transition_closed(
    musicbar: NodeRef<Div>,
    musicinfo: NodeRef<Div>,
    sidebar: RwSignal<Option<HtmlElement>>,
) {
    if let Some(musicbar) = musicbar.get_untracked() {
        let _ = musicbar
            .style("transition", "all 0.2s")
            .style("transform", "translateY(0px)");
    }

    if let Some(musicinfo) = musicinfo.get_untracked() {
        let _ = musicinfo.style("transition", "all 0.2s").style(
            "transform",
            format!("translateY(calc(100vh - {}px))", MUSICBAR_HEIGHT),
        );
    }

    if let Some(sidebar) = sidebar.get_untracked() {
        let style = sidebar.style();
        style.set_property("visibility", "visible").unwrap();
        style.set_property("transition", "all 0.2s").unwrap();
        style.set_property("opacity", "1").unwrap();
    }
}

fn transition_opened(
    musicbar: NodeRef<Div>,
    musicinfo: NodeRef<Div>,
    sidebar: RwSignal<Option<HtmlElement>>,
) {
    if let Some(musicbar) = musicbar.get() {
        let _ = musicbar.style("transition", "all 0.2s").style(
            "transform",
            format!("translateY(calc(-100vh + {}px))", MUSICBAR_HEIGHT),
        );
    }

    if let Some(musicinfo) = musicinfo.get() {
        let _ = musicinfo
            .style("transition", "all 0.2s")
            .style("transform", "translateY(0px)");
    }

    if let Some(sidebar) = sidebar.get_untracked() {
        let style = sidebar.style();
        style.set_property("transition", "all 0.2s").unwrap();
        style.set_property("opacity", "0").unwrap();
        style.set_property("visibility", "hidden").unwrap();
    }
}

fn musicinfo_drag(musicbar: NodeRef<Div>, musicinfo: NodeRef<Div>) {
    let is_dragging = create_rw_signal(false);
    let start_offset = create_rw_signal(0);
    let page_height = create_rw_signal(0);
    let sidebar = create_rw_signal(None);
    let has_moved = create_rw_signal(false);

    let listener = move |client_y: i32| {
        tracing::info!("dragging musicinfo");
        start_offset.set_untracked(client_y);
        page_height.set_untracked(document().body().unwrap().client_height() - SIDEBAR_HEIGHT);
        is_dragging.set_untracked(true);
        has_moved.set_untracked(false);

        let sidebar_elem = document()
            .get_element_by_id("sidebar")
            .map(|e| e.dyn_into::<HtmlElement>().unwrap());
        if let Some(sidebar_elem) = sidebar_elem {
            let style = sidebar_elem.style();
            style.set_property("transition", "all 0s").unwrap();
            style.set_property("display", "flex").unwrap();
            sidebar.set_untracked(Some(sidebar_elem));
        }

        if let Some(musicbar) = musicbar.get() {
            let _ = musicbar.style("transition", "all 0s");
        }

        if let Some(musicinfo) = musicinfo.get() {
            let _ = musicinfo.style("transition", "all 0s");
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
            tracing::info!("dragging musicinfo stop");
            is_dragging.set_untracked(false);

            if has_moved.get_untracked() {
                transition_closed(musicbar, musicinfo, sidebar);
            } else {
                transition_opened(musicbar, musicinfo, sidebar);
            }
        }
    };
    let _ = use_event_listener(document().body(), mouseup, move |ev| {
        ev.stop_propagation();
        listener();
    });
    let _ = use_event_listener(document().body(), touchend, move |ev| {
        ev.stop_propagation();
        listener();
    });

    let listener = move |client_y: i32| {
        if is_dragging.get_untracked() {
            if let Some(musicinfo) = musicinfo.get_untracked() {
                if let Some(musicbar) = musicbar.get_untracked() {
                    let start_offset = start_offset.get_untracked();
                    let page_height = page_height.get_untracked();
                    let client_y_diff =
                        (client_y - start_offset).clamp(-MUSICBAR_HEIGHT, page_height);
                    let musicbar_pos = client_y_diff - page_height;

                    let _ =
                        musicinfo.style("transform", format!("translateY({}px)", client_y_diff));
                    let _ = musicbar.style("transform", format!("translateY({}px)", musicbar_pos));

                    if let Some(sidebar) = sidebar.get_untracked() {
                        let style = sidebar.style();
                        let opacity = client_y_diff as f64 / start_offset as f64;
                        style
                            .set_property("opacity", &format!("{}", opacity.abs()))
                            .unwrap();
                    }

                    if client_y_diff.abs() > (0.2 * page_height as f64) as i32 {
                        has_moved.set_untracked(true);
                    }
                }
            }
        }
    };
    let _ = use_event_listener(document().body(), mousemove, move |ev| {
        ev.stop_propagation();
        listener(ev.client_y())
    });
    let _ = use_event_listener(document().body(), touchmove, move |ev| {
        ev.stop_propagation();
        let touch = ev.touches().item(0);
        if let Some(touch) = touch {
            listener(touch.client_y())
        }
    });
}

fn musicbar_drag(musicbar: NodeRef<Div>, musicinfo: NodeRef<Div>) {
    let is_dragging = create_rw_signal(false);
    let start_offset = create_rw_signal(0);
    let page_height = create_rw_signal(0);
    let sidebar = create_rw_signal(None);
    let has_moved = create_rw_signal(false);

    let listener = move |client_y: i32| {
        tracing::info!("dragging");
        start_offset.set_untracked(client_y);
        page_height.set_untracked(document().body().unwrap().client_height() - SIDEBAR_HEIGHT);
        is_dragging.set_untracked(true);
        has_moved.set_untracked(false);

        let sidebar_elem = document()
            .get_element_by_id("sidebar")
            .map(|e| e.dyn_into::<HtmlElement>().unwrap());
        if let Some(sidebar_elem) = sidebar_elem {
            let style = sidebar_elem.style();
            style.set_property("transition", "all 0s").unwrap();
            sidebar.set_untracked(Some(sidebar_elem));
        }

        if let Some(musicbar) = musicbar.get_untracked() {
            let _ = musicbar.style("transition", "all 0s");
        }

        if let Some(musicinfo) = musicinfo.get_untracked() {
            let _ = musicinfo.style("transition", "all 0s");
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
            tracing::info!("dragging stop");
            is_dragging.set_untracked(false);

            if has_moved.get_untracked() {
                transition_opened(musicbar, musicinfo, sidebar);
            } else {
                transition_closed(musicbar, musicinfo, sidebar);
            }
        }
    };
    let _ = use_event_listener(document().body(), mouseup, move |ev| {
        ev.stop_propagation();
        listener();
    });
    let _ = use_event_listener(document().body(), touchend, move |ev| {
        ev.stop_propagation();
        listener();
    });

    let listener = move |client_y: i32| {
        if is_dragging.get_untracked() {
            if let Some(musicinfo) = musicinfo.get_untracked() {
                if let Some(musicbar) = musicbar.get_untracked() {
                    let page_height = page_height.get_untracked();
                    let start_offset = start_offset.get_untracked();
                    let client_y_diff =
                        (client_y - start_offset).clamp(-(page_height - MUSICBAR_HEIGHT), 0);
                    let musicinfo_pos = page_height + client_y_diff;

                    let _ =
                        musicinfo.style("transform", format!("translateY({}px)", musicinfo_pos));
                    let _ = musicbar.style("transform", format!("translateY({}px)", client_y_diff));

                    if let Some(sidebar) = sidebar.get_untracked() {
                        let style = sidebar.style();
                        let opacity = client_y as f64 / start_offset as f64;
                        style
                            .set_property("opacity", &format!("{}", opacity))
                            .unwrap();
                    }

                    if client_y_diff.abs() > (0.2 * page_height as f64) as i32 {
                        has_moved.set_untracked(true);
                    }
                }
            }
        }
    };

    let _ = use_event_listener(document().body(), mousemove, move |ev| {
        ev.stop_propagation();
        listener(ev.client_y())
    });
    let _ = use_event_listener(document().body(), touchmove, move |ev| {
        ev.stop_propagation();
        let touch = ev.touches().item(0);
        if let Some(touch) = touch {
            listener(touch.client_y())
        }
    });
}

#[tracing::instrument(level = "trace")]
#[component]
pub fn MusicBar() -> impl IntoView {
    let ui_store = expect_context::<RwSignal<UiStore>>();
    let show_musicinfo = create_read_slice(ui_store, move |s| s.get_show_queue());

    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

    let musicinfo_ref = create_node_ref();
    let musicbar_ref = create_node_ref();

    if is_mobile {
        musicbar_drag(musicbar_ref, musicinfo_ref);
        musicinfo_drag(musicbar_ref, musicinfo_ref);
    }

    view! {
        <div class="musicbar-content d-flex" class:musicbar-content-mobile=is_mobile>
            {move || {
                if is_mobile {
                    view! { <MusicInfoMobile show=show_musicinfo node_ref=musicinfo_ref /> }
                } else {
                    view! { <MusicInfo show=show_musicinfo node_ref=musicinfo_ref /> }
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
