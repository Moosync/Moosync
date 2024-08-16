use crate::components::provider_icon::ProviderIcon;
use leptos::{component, view, IntoView, SignalGet};
use leptos_router::A;
use leptos_virtual_scroller::VirtualGridScroller;

pub struct SimplifiedCardItem {
    pub title: String,
    pub cover: Option<String>,
    pub id: String,
    pub icon: Option<String>,
}

#[component()]
pub fn CardItem(#[prop()] item: SimplifiedCardItem) -> impl IntoView {
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
    C: Fn((usize, &T)) -> SimplifiedCardItem + 'static,
    S: SignalGet<Value = Vec<T>> + Copy + 'static,
{
    view! {
        <VirtualGridScroller
            each=items
            item_width=275
            item_height=275
            children=move |data| {
                let data = card_item(data);
                view! {
                    <A href=format!("single?id={}", data.id.clone())>
                        <CardItem item=data />
                    </A>
                }
            }
        />
    }
}
