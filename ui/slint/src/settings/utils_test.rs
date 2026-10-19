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

use preferences_proto::moosync::types::{
    PreferenceItem as ProtoPreferenceItem, PreferenceOption as ProtoPreferenceOption,
    PreferenceType as ProtoPreferenceType, PreferenceValue, StringList, preference_value,
};
use slint::Model;
use tracing_test::traced_test;

use crate::{PreferenceItem, PreferenceOption, PreferenceType};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_option_from_proto() {
    let proto_opt = ProtoPreferenceOption {
        id: "test_opt".to_string(),
        label: "Test Label".to_string(),
    };

    let opt: PreferenceOption = proto_opt.into();

    assert_eq!(opt.id, "test_opt");
    assert_eq!(opt.label, "Test Label");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_item_from_proto() {
    let proto_data = ProtoPreferenceItem {
        id: "test_key".to_string(),
        pref_type: ProtoPreferenceType::ToggleGroup as i32,
        title: "Test Title".to_string(),
        subtitle: "Test Subtitle".to_string(),
        placeholder: Some("placeholder".to_string()),
        validation: "validation".to_string(),
        options: vec![ProtoPreferenceOption {
            id: "opt1".to_string(),
            label: "Opt 1".to_string(),
        }],
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::BoolValue(true)),
        }),
        default: None,
    };

    let item: PreferenceItem = proto_data.into();

    assert_eq!(item.id, "test_key");
    assert_eq!(item.pref_type, PreferenceType::ToggleGroup);
    assert_eq!(item.title, "Test Title");
    assert_eq!(item.subtitle, "Test Subtitle");
    assert_eq!(item.placeholder, "placeholder");
    assert_eq!(item.validation, "validation");
    assert!(item.value_bool);
    assert_eq!(item.options.row_count(), 1);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_item_roundtrip() {
    let proto_item = ProtoPreferenceItem {
        id: "paths".to_string(),
        pref_type: ProtoPreferenceType::PathSelector as i32,
        title: "Paths".to_string(),
        subtitle: "Sub".to_string(),
        placeholder: Some("ph".to_string()),
        validation: "val".to_string(),
        options: vec![],
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::ListValue(StringList {
                values: vec!["/music".to_string()],
            })),
        }),
        default: None,
    };

    let ui_item: PreferenceItem = proto_item.clone().into();
    assert_eq!(ui_item.value_list.row_count(), 1);
    assert_eq!(ui_item.value_list.row_data(0).unwrap(), "/music");

    let converted_back: ProtoPreferenceItem = ui_item.into();
    assert_eq!(converted_back.id, proto_item.id);
    assert_eq!(converted_back.pref_type, proto_item.pref_type);
    assert_eq!(converted_back.value, proto_item.value);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_item_number_and_text_roundtrip() {
    let number_item = ProtoPreferenceItem {
        id: "num".to_string(),
        pref_type: ProtoPreferenceType::NumberInputGroup as i32,
        title: "Number".to_string(),
        subtitle: "".to_string(),
        placeholder: None,
        validation: "".to_string(),
        options: vec![],
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::NumberValue(42.5)),
        }),
        default: None,
    };
    let ui_num: PreferenceItem = number_item.clone().into();
    assert_eq!(ui_num.value_number, 42.5);
    let back_num: ProtoPreferenceItem = ui_num.into();
    assert_eq!(back_num.value, number_item.value);

    let text_item = ProtoPreferenceItem {
        id: "txt".to_string(),
        pref_type: ProtoPreferenceType::TextInputGroup as i32,
        title: "Text".to_string(),
        subtitle: "".to_string(),
        placeholder: None,
        validation: "".to_string(),
        options: vec![],
        value: None,
        default: Some(PreferenceValue {
            value: Some(preference_value::Value::StringValue(
                "default_str".to_string(),
            )),
        }),
    };
    let ui_txt: PreferenceItem = text_item.into();
    assert_eq!(ui_txt.value_string.as_str(), "default_str");
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_type_all_variants_roundtrip() {
    let variants = [
        (
            ProtoPreferenceType::PathSelector,
            PreferenceType::PathSelector,
        ),
        (
            ProtoPreferenceType::SingleFileInput,
            PreferenceType::SingleFileInput,
        ),
        (
            ProtoPreferenceType::NumberInputGroup,
            PreferenceType::NumberInputGroup,
        ),
        (
            ProtoPreferenceType::TextInputGroup,
            PreferenceType::TextInputGroup,
        ),
        (
            ProtoPreferenceType::ToggleGroup,
            PreferenceType::ToggleGroup,
        ),
        (ProtoPreferenceType::RadioGroup, PreferenceType::RadioGroup),
        (
            ProtoPreferenceType::DropdownGroup,
            PreferenceType::DropdownGroup,
        ),
        (
            ProtoPreferenceType::TextArrayInput,
            PreferenceType::TextArrayInput,
        ),
    ];

    for (proto_type, slint_type) in variants {
        let converted: PreferenceType = proto_type.into();
        assert_eq!(converted, slint_type);
        let back: ProtoPreferenceType = converted.into();
        assert_eq!(back, proto_type);
    }

    let unspecified = ProtoPreferenceType::Unspecified;
    let converted: PreferenceType = unspecified.into();
    assert_eq!(converted, PreferenceType::TextInputGroup);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_option_roundtrip() {
    let proto_opt = ProtoPreferenceOption {
        id: "opt_id".to_string(),
        label: "Opt Label".to_string(),
    };
    let opt: PreferenceOption = proto_opt.clone().into();
    let back: ProtoPreferenceOption = opt.into();
    assert_eq!(back.id, proto_opt.id);
    assert_eq!(back.label, proto_opt.label);
}
