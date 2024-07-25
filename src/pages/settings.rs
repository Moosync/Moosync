use crate::{
    components::prefs::checkbox::CheckboxPref, components::prefs::input::InputPref,
    components::prefs::paths::PathsPref, console_log,
};
use leptos::{component, view, For, IntoView, SignalWith};
use leptos_router::use_params_map;
use types::ui::preferences::{Page, PreferenceTypes, PreferenceUIData, PreferenceUIFile};

#[component]
pub fn SettingsPage(#[prop()] prefs: PreferenceUIFile) -> impl IntoView {
    let params = use_params_map();
    let page = params.with(|params| params.get("page").cloned()).unwrap();
    let page = prefs.page.into_iter().find(|val| val.path == page).unwrap();

    view! {
        <div class="prefs-container">
            <For
                each=move || page.data.clone()
                key=|path| path.key.clone()
                children=move |data: PreferenceUIData| {
                    match data._type {
                        PreferenceTypes::Paths => view! { <PathsPref data=data /> },
                        PreferenceTypes::Text
                        | PreferenceTypes::Number
                        | PreferenceTypes::FilePicker => view! { <InputPref data=data /> },
                        PreferenceTypes::Checkbox => {
                            view! { <CheckboxPref data=data /> }.into_view()
                        }
                    }
                }
            />

        </div>
    }
}
