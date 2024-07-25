use leptos::{component, view, For, IntoView, SignalGet, SignalUpdate};
use types::ui::preferences::PreferenceUIData;

use crate::{
    components::prefs::common::setup_value_listeners_vec, icons::tooltip::Tooltip,
    utils::prefs::open_file_browser,
};

#[component()]
pub fn PathsPref(#[prop()] data: PreferenceUIData) -> impl IntoView {
    let (paths, selected_paths) = setup_value_listeners_vec(data.key.clone());

    view! {
        <div class="container-fluid mt-4">
            <div class="row no-gutters align-items-center">

                <div class="row no-gutters">
                    <div class="col-auto align-self-center title d-flex preference-title">
                        {data.name}
                    </div>
                    <div class="col-auto ml-2">
                        <Tooltip title=data.tooltip />
                    </div>
                </div>

                <div class="col-auto new-directories ml-auto justify-content-center">
                    <div>Refresh</div>
                </div>

                <div class="col-auto new-directories ml-4">
                    <div
                        class="add-directories-button"
                        on:click=move |_| open_file_browser(true, true, vec![], selected_paths)
                    >
                        Add Folder
                    </div>
                </div>
            </div>

            <div class="row no-gutters path-prefs-background w-100 mt-2 d-flex">

                <For
                    each=move || paths.get()
                    key=|p| p.clone()
                    children=move |path: String| {
                        view! {
                            <div class="row no-gutters mt-3 item w-100">
                                <div class="col col-md-8 col-lg-9 align-self-center justify-content-start ml-3 no-checkbox-margin">
                                    <div class="item-text text-truncate">{path.clone()}</div>
                                </div>
                                <div class="col-auto align-self-center ml-auto">
                                    <div
                                        class="remove-button w-100"
                                        on:click=move |_| {
                                            paths.update(|paths| paths.retain(|p| *p != path))
                                        }
                                    >
                                        Remove
                                    </div>
                                </div>
                            </div>
                        }
                    }
                />
            </div>
        </div>
    }
}
