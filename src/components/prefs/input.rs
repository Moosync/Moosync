use crate::{console_log, utils::prefs::open_file_browser_single};
use leptos::{component, event_target_value, view, IntoView, SignalGet, SignalSet};
use leptos_use::use_debounce_fn_with_arg;
use types::ui::preferences::{PreferenceTypes, PreferenceUIData};

use crate::{components::prefs::common::setup_value_listeners, icons::folder_icon::FolderIcon};

#[component()]
pub fn InputPref(#[prop()] data: PreferenceUIData) -> impl IntoView {
    let (show_input, inp_type) = match data._type {
        PreferenceTypes::FilePicker => (false, None),
        PreferenceTypes::Text => (true, Some("text")),
        PreferenceTypes::Number => (true, Some("number")),
        // Below case should never happen
        _ => (true, None),
    };

    let _type = data._type.clone();
    let pref_value = setup_value_listeners(data.key.clone());

    let debounced_update = use_debounce_fn_with_arg(
        move |event: web_sys::Event| {
            let value = event_target_value(&event);
            if _type == PreferenceTypes::Number && value.parse::<f64>().is_err() {
                console_log!("Invalid number");
                return;
            }
            pref_value.set(value);
        },
        500.0,
    );

    view! {
        <div class="container-fluid">
            <div class="row no-gutters">Title</div>

            <div class="row no-gutters input-prefs-background w-100 mt-2 d-flex align-content-center">

                {if !show_input {
                    view! {
                        <div class="col-auto align-self-center ml-4 folder-icon">
                            <FolderIcon on:click=move |_| open_file_browser_single(
                                true,
                                vec![],
                                pref_value,
                            ) />
                        </div>
                    }
                        .into_view()
                } else {
                    view! {}.into_view()
                }}
                <div class="col-auto ml-3 align-self-center flex-grow-1 justify-content-start">

                    {if show_input {
                        view! {
                            <input
                                class="ext-input w-100 ext-input-hover"
                                type=inp_type
                                prop:value=pref_value
                                on:input=move |e| {
                                    debounced_update(e);
                                }
                            />
                        }
                            .into_view()
                    } else {
                        view! {
                            <div class="item-text text-truncate file-picker-text">
                                {move || pref_value.get()}
                            </div>
                        }
                            .into_view()
                    }}
                </div>
            </div>

        </div>
    }
}
