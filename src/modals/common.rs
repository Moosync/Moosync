use leptos::{
    component, create_node_ref, expect_context, html::Div, view, Children, IntoView, RwSignal,
    SignalUpdate,
};
use leptos_use::on_click_outside;

use crate::store::modal_store;

#[component]
pub fn GenericModal(#[prop()] size: String, children: Children) -> impl IntoView {
    let target = create_node_ref::<Div>();
    let modal_store = expect_context::<RwSignal<modal_store::ModalStore>>();

    on_click_outside(target, move |_| {
        modal_store.update(|s| s.clear_active_modal());
    });

    view! {
        <div style="position: absolute; z-index: 1040;">
            <div class="modal fade show">
                <div node_ref=target class=format!("modal-dialog {} modal-dialog-centered", size)>
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
