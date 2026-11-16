use preferences_proto::moosync::types::{
    PreferenceItem as ProtoPreferenceItem, PreferenceOption as ProtoPreferenceOption,
    PreferenceType as ProtoPreferenceType, PreferenceValue, StatusDot as ProtoStatusDot,
    StatusDotColor as ProtoStatusDotColor, StatusDotList, StringList, preference_value,
};
use slint::Model;
use tracing_test::traced_test;

use crate::{PreferenceItem, PreferenceOption, PreferenceType, StatusDotColor, StatusDotItem};

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
        (
            ProtoPreferenceType::StatusGroup,
            PreferenceType::StatusGroup,
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

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_preference_item_status_group_roundtrip() {
    let proto_dots = vec![
        ProtoStatusDot {
            title: "ReShade".to_string(),
            subtitle: "6.8.0 (Add-on Edition)".to_string(),
            color: ProtoStatusDotColor::Info as i32,
        },
        ProtoStatusDot {
            title: "vkBasalt".to_string(),
            subtitle: "0.3.2.10-1".to_string(),
            color: ProtoStatusDotColor::Warn as i32,
        },
        ProtoStatusDot {
            title: "MangoHud".to_string(),
            subtitle: "Error".to_string(),
            color: ProtoStatusDotColor::Error as i32,
        },
        ProtoStatusDot {
            title: "Gamescope".to_string(),
            subtitle: "Disabled".to_string(),
            color: ProtoStatusDotColor::Off as i32,
        },
    ];
    let proto_item = ProtoPreferenceItem {
        id: "ext_status".to_string(),
        pref_type: ProtoPreferenceType::StatusGroup as i32,
        title: "Status".to_string(),
        subtitle: "Component status".to_string(),
        placeholder: None,
        validation: String::new(),
        options: vec![],
        value: Some(PreferenceValue {
            value: Some(preference_value::Value::StatusList(StatusDotList {
                values: proto_dots,
            })),
        }),
        default: None,
    };

    let ui_item: PreferenceItem = proto_item.clone().into();
    assert_eq!(ui_item.pref_type, PreferenceType::StatusGroup);
    assert_eq!(ui_item.status_dots.row_count(), 4);
    let first: StatusDotItem = ui_item.status_dots.row_data(0).unwrap();
    assert_eq!(first.title, "ReShade");
    assert_eq!(first.subtitle, "6.8.0 (Add-on Edition)");
    assert_eq!(first.color, StatusDotColor::Info);
    assert_eq!(
        ui_item.status_dots.row_data(1).unwrap().color,
        StatusDotColor::Warn
    );
    assert_eq!(
        ui_item.status_dots.row_data(2).unwrap().color,
        StatusDotColor::Error
    );
    assert_eq!(
        ui_item.status_dots.row_data(3).unwrap().color,
        StatusDotColor::Off
    );

    let converted_back: ProtoPreferenceItem = ui_item.into();
    assert_eq!(converted_back.id, proto_item.id);
    assert_eq!(converted_back.pref_type, proto_item.pref_type);
    assert_eq!(converted_back.value, proto_item.value);
}
