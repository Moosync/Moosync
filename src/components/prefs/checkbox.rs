use crate::console_log;
use leptos::{
    component, event_target_checked, view, CollectView, IntoView, SignalGet, SignalUpdate,
};
use types::preferences::CheckboxPreference;
use types::ui::preferences::PreferenceUIData;

use crate::components::prefs::common::setup_value_listeners;

#[component()]
pub fn CheckboxPref(#[prop()] data: PreferenceUIData) -> impl IntoView {
    let pref_value = setup_value_listeners::<Vec<CheckboxPreference>>(data.key.clone());

    view! {
        <div class="container-fluid">
            <div class="row no-gutters">Title</div>

            {move || {
                data.items
                    .clone()
                    .unwrap()
                    .into_iter()
                    .map(|item| {
                        let enabled = pref_value
                            .get()
                            .iter()
                            .find(|val| val.key == item.key.clone())
                            .map(|item| item.enabled)
                            .unwrap_or(false);
                        let checkbox_key = format!(
                            "checkbox-{}-{}",
                            data.key.clone(),
                            item.key.clone(),
                        );
                        console_log!("checkbox_key: {}: {}", checkbox_key, enabled);
                        view! {
                            <div class="row no-gutters item w-100">
                                <div class="col-auto align-self-center">
                                    <div class="custom-control custom-checkbox">
                                        <input
                                            type="checkbox"
                                            class="custom-control-input"
                                            prop:checked=enabled
                                            id=checkbox_key.clone()
                                            on:change=move |ev| {
                                                let enabled = event_target_checked(&ev);
                                                pref_value
                                                    .update(|val| {
                                                        let found = val
                                                            .iter_mut()
                                                            .find(|val| val.key == item.key.clone());
                                                        if let Some(item) = found {
                                                            item.enabled = enabled;
                                                        } else {
                                                            val.push(CheckboxPreference {
                                                                key: item.key.clone(),
                                                                enabled,
                                                            });
                                                        }
                                                    });
                                            }
                                        />
                                        <label
                                            for=checkbox_key
                                            class="custom-control-label"
                                        ></label>
                                    </div>
                                </div>

                                <div class="col-md-8 col-lg-9 col align-self-center ml-3 justify-content-start">
                                    <div class="item-text text-truncate">{item.name}</div>
                                </div>
                            </div>
                        }
                    })
                    .collect_view()
            }}
        </div>
    }
}
