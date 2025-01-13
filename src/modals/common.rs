use leptos::{component, html::Div, prelude::*, view, IntoView};
use leptos_use::on_click_outside;

use crate::store::modal_store;

#[tracing::instrument(level = "trace", skip(size, children))]
#[component]
pub fn GenericModal<T>(#[prop()] size: T, children: Children) -> impl IntoView
where
    T: Fn() -> String + 'static + Send + Sync,
{
    let target = create_node_ref::<Div>();
    let modal_store = expect_context::<RwSignal<modal_store::ModalStore>>();

    let _ = on_click_outside(target, move |_| {
        modal_store.update(|s| s.clear_active_modal());
    });

    view! {
        <div style="position: absolute; z-index: 9999;">
            <div class="modal fade show">
                <div
                    node_ref=target
                    class=move || format!("modal-dialog {} modal-dialog-centered", size())
                >
                    <div class="modal-content">
                        <div class="modal-body">

                            {children()}

                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}
