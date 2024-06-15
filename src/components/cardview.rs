use leptos::{component, view, IntoView};

pub struct SimplifiedCardItem {
    pub title: String,
    pub cover: Option<String>,
}

#[component()]
pub fn CardItem(#[prop()] item: SimplifiedCardItem) -> impl IntoView {
    view! {
        <div class="card mb-2 card-grow" style="width: 200px;">
            <div class="card-img-top">
                <div class="embed-responsive embed-responsive-1by1">
                    <div class="embed-responsive-item img-container">
                        // Ext icon
                        <div class="overlay me-auto justify-content-center d-flex align-items-center h-100 w-100"></div>
                        <img src=item.cover class="img-fluid w-100 h-100"/>
                    </div>
                </div>
            </div>
            <div class="card-body">
                <p class="card-title text-truncate">{item.title}</p>
            </div>
        </div>
    }
}
