use std::rc::Rc;

use crate::components::provider_icon::ProviderIcon;
use leptos::{component, view, IntoView, SignalGet};
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
pub fn CardItem<T>(#[prop()] item: SimplifiedCardItem<T>) -> impl IntoView {
    view! {
        <div class="card mb-2 card-grow" style="width: 200px;">
            <div class="card-img-top">
                <div class="embed-responsive embed-responsive-1by1">
                    <div class="embed-responsive-item img-container">
                        // Ext icon
                        <div class="provider-icon-overlay me-auto justify-content-center d-flex align-items-center">
                            {if let Some(icon) = item.icon.clone() {
                                view! { <ProviderIcon extension=icon /> }
                            } else {
                                view! {}.into_view()
                            }}
                        </div>
                        <img src=item.cover class="img-fluid w-100 h-100" />
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
pub fn CardView<T, S, C>(#[prop()] items: S, #[prop()] card_item: C) -> impl IntoView
where
    T: 'static + Clone,
    C: Fn((usize, &T)) -> SimplifiedCardItem<T> + 'static,
    S: SignalGet<Value = Vec<T>> + Copy + 'static,
{
    view! {
        <VirtualGridScroller
            each=items
            item_width=275
            item_height=275
            children=move |data| {
                let data1 = data.1.clone();
                let card_item_data = card_item(data);
                let card_item_data1 = card_item_data.clone();
                view! {
                    <A href=format!("single?id={}", card_item_data.id.clone())>
                        <CardItem
                            on:contextmenu=move |ev| {
                                if let Some(cb) = &card_item_data1.context_menu {
                                    cb(ev, data1.clone());
                                }
                            }
                            item=card_item_data
                        />
                    </A>
                }
            }
        />
    }
}
