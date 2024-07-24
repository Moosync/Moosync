use crate::{
    components::prefs::checkbox::CheckboxPref, components::prefs::input::InputPref,
    components::prefs::paths::PathsPref, console_log,
};
use leptos::{component, view, For, IntoView};
use types::ui::preferences::{PreferenceTypes, PreferenceUIData, PreferenceUIFile};

#[component]
pub fn Settings() -> impl IntoView {
    let prefs: PreferenceUIFile = serde_yaml::from_str(include_str!("../prefs.yaml")).unwrap();
    console_log!("prefs: {:?}", prefs);

    let first_page = prefs.page.first().unwrap().clone();
    view! {
        <div>

            <For
                each=move || first_page.data.clone()
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
