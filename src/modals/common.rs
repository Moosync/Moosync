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

use leptos::{component, html::Div, prelude::*, view, IntoView};
use leptos_use::on_click_outside;

use crate::store::modal_store;

#[tracing::instrument(level = "trace", skip(size, children))]
#[component]
pub fn GenericModal<T>(#[prop()] size: T, children: Children) -> impl IntoView
where
    T: Fn() -> String + 'static + Send + Sync,
{
    let target = NodeRef::<Div>::new();
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
