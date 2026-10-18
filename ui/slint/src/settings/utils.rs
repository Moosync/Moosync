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

use std::sync::LazyLock;

use preferences_proto::moosync::types::{
    PreferenceItem as ProtoPreferenceItem, PreferenceOption as ProtoPreferenceOption,
    PreferenceType as ProtoPreferenceType, PreferenceValue, StringList, preference_value,
};
use slint::{Model, ModelRc, SharedString, VecModel, invoke_from_event_loop};
use state_manager::StateManager;

use crate::{MainWindow, PreferenceItem, PreferenceOption, PreferenceType};

#[tracing::instrument(level = "debug", skip_all)]
pub fn clone_lazy_preferences(
    preferences: &[&LazyLock<ProtoPreferenceItem>],
) -> Vec<ProtoPreferenceItem> {
    preferences.iter().map(|p| (**p).clone()).collect()
}

#[tracing::instrument(level = "debug", skip_all)]
pub fn to_preference_model(items: Vec<ProtoPreferenceItem>) -> ModelRc<PreferenceItem> {
    let slint_items: Vec<PreferenceItem> = items.into_iter().map(PreferenceItem::from).collect();
    ModelRc::new(VecModel::from(slint_items))
}

#[tracing::instrument(level = "debug", skip_all)]
pub async fn update_preferences_in_ui<F>(
    main_window_weak: slint::Weak<MainWindow>,
    state_manager: &'static StateManager,
    preferences: &[&LazyLock<ProtoPreferenceItem>],
    context_name: &'static str,
    on_render: F,
) where
    F: FnOnce(&MainWindow, Vec<ProtoPreferenceItem>) + Send + 'static,
{
    let config = state_manager.get_preference_config();
    let items: Vec<ProtoPreferenceItem> =
        preferences.iter().map(|pref| config.load(pref)).collect();

    if let Err(e) = invoke_from_event_loop(move || {
        if let Some(main_window) = main_window_weak.upgrade() {
            on_render(&main_window, items);
        }
    }) {
        tracing::error!(
            "Failed to invoke from event loop in {context_name} refresh_preferences: {e:?}"
        );
    }
}

impl From<ProtoPreferenceType> for PreferenceType {
    fn from(t: ProtoPreferenceType) -> Self {
        match t {
            ProtoPreferenceType::PathSelector => PreferenceType::PathSelector,
            ProtoPreferenceType::SingleFileInput => PreferenceType::SingleFileInput,
            ProtoPreferenceType::NumberInputGroup => PreferenceType::NumberInputGroup,
            ProtoPreferenceType::TextInputGroup => PreferenceType::TextInputGroup,
            ProtoPreferenceType::ToggleGroup => PreferenceType::ToggleGroup,
            ProtoPreferenceType::RadioGroup => PreferenceType::RadioGroup,
            ProtoPreferenceType::DropdownGroup => PreferenceType::DropdownGroup,
            ProtoPreferenceType::TextArrayInput => PreferenceType::TextArrayInput,
            ProtoPreferenceType::Unspecified => PreferenceType::TextInputGroup,
        }
    }
}

impl From<PreferenceType> for ProtoPreferenceType {
    fn from(t: PreferenceType) -> Self {
        match t {
            PreferenceType::PathSelector => ProtoPreferenceType::PathSelector,
            PreferenceType::SingleFileInput => ProtoPreferenceType::SingleFileInput,
            PreferenceType::NumberInputGroup => ProtoPreferenceType::NumberInputGroup,
            PreferenceType::TextInputGroup => ProtoPreferenceType::TextInputGroup,
            PreferenceType::ToggleGroup => ProtoPreferenceType::ToggleGroup,
            PreferenceType::RadioGroup => ProtoPreferenceType::RadioGroup,
            PreferenceType::DropdownGroup => ProtoPreferenceType::DropdownGroup,
            PreferenceType::TextArrayInput => ProtoPreferenceType::TextArrayInput,
        }
    }
}

impl From<ProtoPreferenceOption> for PreferenceOption {
    fn from(opt: ProtoPreferenceOption) -> Self {
        Self {
            id: opt.id.into(),
            label: opt.label.into(),
        }
    }
}

impl From<PreferenceOption> for ProtoPreferenceOption {
    fn from(opt: PreferenceOption) -> Self {
        Self {
            id: opt.id.to_string(),
            label: opt.label.to_string(),
        }
    }
}

impl From<ProtoPreferenceItem> for PreferenceItem {
    fn from(data: ProtoPreferenceItem) -> Self {
        let pref_type = match ProtoPreferenceType::try_from(data.pref_type) {
            Ok(t) => PreferenceType::from(t),
            Err(_) => PreferenceType::TextInputGroup,
        };

        let options: Vec<PreferenceOption> = data
            .options
            .into_iter()
            .map(PreferenceOption::from)
            .collect();

        let mut item = Self {
            id: data.id.into(),
            pref_type,
            title: data.title.into(),
            subtitle: data.subtitle.into(),
            placeholder: data.placeholder.unwrap_or_default().into(),
            validation: data.validation.into(),
            value_string: Default::default(),
            value_bool: false,
            value_number: 0.0,
            value_list: ModelRc::default(),
            options: ModelRc::new(VecModel::from(options)),
        };

        let value_to_use = data.value.or(data.default);
        if let Some(pref_val) = value_to_use {
            if let Some(val) = pref_val.value {
                match val {
                    preference_value::Value::StringValue(s) => item.value_string = s.into(),
                    preference_value::Value::BoolValue(b) => item.value_bool = b,
                    preference_value::Value::NumberValue(n) => item.value_number = n,
                    preference_value::Value::ListValue(l) => {
                        let shared_list: Vec<SharedString> =
                            l.values.iter().map(|s| s.as_str().into()).collect();
                        item.value_list = ModelRc::new(VecModel::from(shared_list));
                    }
                }
            }
        }
        item
    }
}

impl From<PreferenceItem> for ProtoPreferenceItem {
    fn from(item: PreferenceItem) -> Self {
        let pref_type = ProtoPreferenceType::from(item.pref_type);
        let options: Vec<ProtoPreferenceOption> = item
            .options
            .iter()
            .map(ProtoPreferenceOption::from)
            .collect();

        let value_val = match item.pref_type {
            PreferenceType::ToggleGroup => {
                Some(preference_value::Value::BoolValue(item.value_bool))
            }
            PreferenceType::NumberInputGroup => {
                Some(preference_value::Value::NumberValue(item.value_number))
            }
            PreferenceType::TextInputGroup
            | PreferenceType::SingleFileInput
            | PreferenceType::RadioGroup
            | PreferenceType::DropdownGroup => Some(preference_value::Value::StringValue(
                item.value_string.to_string(),
            )),
            PreferenceType::PathSelector | PreferenceType::TextArrayInput => {
                let list: Vec<String> = item.value_list.iter().map(|s| s.to_string()).collect();
                Some(preference_value::Value::ListValue(StringList {
                    values: list,
                }))
            }
        };

        ProtoPreferenceItem {
            id: item.id.to_string(),
            pref_type: pref_type as i32,
            title: item.title.to_string(),
            subtitle: item.subtitle.to_string(),
            placeholder: if item.placeholder.is_empty() {
                None
            } else {
                Some(item.placeholder.to_string())
            },
            validation: item.validation.to_string(),
            options,
            value: Some(PreferenceValue { value: value_val }),
            default: None,
        }
    }
}
