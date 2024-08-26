use std::rc::Rc;

use crate::{
    components::provider_icon::ProviderIcon,
    icons::{play_hover_icon::PlayHoverIcon, song_default_icon::SongDefaultIcon},
};
use leptos::{component, create_rw_signal, view, IntoView, SignalGet, SignalSet};
use leptos_router::A;
use leptos_virtual_scroller::VirtualGridScroller;

#[derive(Clone)]
pub struct SimplifiedCardItem<T> {
    pub title: String,
    pub cover: Option<String>,
    pub id: String,
    pub icon: Option<String>,
    pub context_menu: Option<Rc<Box<dyn Fn(leptos::ev::MouseEvent, T)>>>,
}

#[component()]
pub fn CardItem<T>(
    #[prop()] item: SimplifiedCardItem<T>,
    #[prop(optional, default = false)] songs_view: bool,
    #[prop(optional)] on_click: Option<Rc<Box<dyn Fn()>>>,
) -> impl IntoView {
    let show_default_icon = create_rw_signal(item.cover.is_none());

    view! {
        <div class="card mb-2 card-grow" style="width: 200px;">
            <div class="card-img-top">
                <div class="embed-responsive embed-responsive-1by1">
                    <div class="embed-responsive-item img-container">

                        <div class="card_overlay">
                            {move || {
                                let on_click = on_click.clone();
                                if songs_view {
                                    view! {
                                        <div
                                            class="play-button-song-list card-overlay-background d-flex justify-content-center w-100 h-100"
                                            on:click=move |_| {
                                                let on_click = on_click.clone();
                                                if let Some(cb) = on_click {
                                                    cb();
                                                }
                                            }
                                        >
                                            <PlayHoverIcon />
                                        </div>
                                    }
                                        .into_view()
                                } else {
                                    view! {}.into_view()
                                }
                            }}
                        </div>

                        <div class="provider-icon-overlay me-auto justify-content-center d-flex align-items-center">
                            {if let Some(icon) = item.icon.clone() {
                                view! { <ProviderIcon extension=icon /> }
                            } else {
                                view! {}.into_view()
                            }}
                        </div>
                        {move || {
                            if show_default_icon.get() {
                                view! {
                                    <SongDefaultIcon class="rounded-corners img-fluid w-100 h-100"
                                        .into() />
                                }
                                    .into_view()
                            } else {
                                view! {
                                    <img
                                        src=item.cover.clone()
                                        class="rounded-corners img-fluid w-100 h-100"
                                        on:error=move |_| show_default_icon.set(true)
                                    />
                                }
                                    .into_view()
                            }
                        }}

                    </div>
                </div>
            </div>
            <div class="card-body">
                <p class="card-title text-truncate">{item.title}</p>
            </div>
        </div>
    }
}

#[component()]
pub fn CardView<T, S, C>(
    #[prop()] items: S,
    #[prop()] card_item: C,
    #[prop(optional, default = false)] songs_view: bool,
    #[prop(optional)] on_click: Option<Box<dyn Fn(T)>>,
) -> impl IntoView
where
    T: 'static + Clone,
    C: Fn((usize, &T)) -> SimplifiedCardItem<T> + 'static,
    S: SignalGet<Value = Vec<T>> + Copy + 'static,
{
    let on_click = on_click.map(Rc::new);
    view! {
        <VirtualGridScroller
            each=items
            item_width=275
            item_height=275
            children=move |data| {
                let data1 = data.1.clone();
                let data2 = data.1.clone();
                let card_item_data = card_item(data);
                let card_item_data1 = card_item_data.clone();
                let on_click = on_click.clone();
                if songs_view {
                    view! {
                        <CardItem
                            on:contextmenu=move |ev| {
                                if let Some(cb) = &card_item_data1.context_menu {
                                    cb(ev, data1.clone());
                                }
                            }
                            item=card_item_data
                            songs_view=songs_view
                            on_click=Rc::new(
                                Box::new(move || {
                                    if let Some(cb) = on_click.clone() {
                                        cb(data2.clone());
                                    }
                                }),
                            )
                        />
                    }
                } else {
                    view! {
                        <A href=format!("single?id={}", card_item_data.id.clone())>
                            <CardItem
                                on:contextmenu=move |ev| {
                                    if let Some(cb) = &card_item_data1.context_menu {
                                        cb(ev, data1.clone());
                                    }
                                }
                                item=card_item_data
                                songs_view=songs_view
                            />
                        </A>
                    }
                }
            }
        />
    }
}
