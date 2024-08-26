use crate::{
    components::color_picker::ColorPicker,
    console_log,
    icons::{
        folder_icon::FolderIcon, import_theme_icon::ImportThemeIcon,
        new_theme_button_icon::NewThemeButtonIcon, theme_view_icon::ThemeViewIcon,
        tooltip::Tooltip,
    },
    store::modal_store::ModalStore,
    utils::prefs::{import_theme, open_file_browser_single, save_theme},
};
use leptos::{
    component, create_effect, create_memo, create_node_ref, create_read_slice, create_rw_signal,
    event_target_value, expect_context, view, CollectView, IntoView, RwSignal, SignalGet,
    SignalGetUntracked, SignalSet, SignalUpdate,
};
use leptos_use::on_click_outside;
use types::{
    themes::{ThemeDetails, ThemeItem},
    window::DialogFilter,
};
use web_sys::MouseEvent;

use crate::modals::common::GenericModal;

#[derive(Debug, Clone)]
enum State {
    None,
    NewTheme,
    ImportTheme,
}

#[component]
pub fn NewThemeModal() -> impl IntoView {
    let state = create_rw_signal(State::None);
    let theme_path = create_rw_signal(String::new());
    create_effect(move |_| {
        let state = state.get();

        if let State::ImportTheme = state {
            open_file_browser_single(
                false,
                vec![DialogFilter {
                    name: "Moosync theme (.mstx)".into(),
                    extensions: vec!["mstx".into(), "zip".into()],
                }],
                theme_path,
            );
        }
    });

    let close_modal = move || {
        let modal_store: RwSignal<ModalStore> = expect_context();
        modal_store.update(|s| s.clear_active_modal());
    };

    create_effect(move |_| {
        let theme_path = theme_path.get();
        if theme_path.is_empty() {
            return;
        }

        import_theme(theme_path, close_modal);
    });

    view! {
        <GenericModal size=move || {
            {
                match state.get() {
                    State::None => "modal-md",
                    State::NewTheme => "modal-xl",
                    State::ImportTheme => "modal-lg",
                }
            }
                .into()
        }>

            {move || match state.get() {
                State::None => {
                    view! {
                        <div class="container">
                            <div class="row h-100">
                                <div
                                    class="col d-flex"
                                    on:click=move |_| state.set(State::NewTheme)
                                >
                                    <div class="row item-box align-self-center">
                                        <div class="col-auto d-flex playlist-modal-item-container w-100">
                                            <div class="row w-100">
                                                <div class="col d-flex justify-content-center w-100">
                                                    <div class="item-icon">
                                                        <NewThemeButtonIcon />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="row">
                                                <div class="col d-flex justify-content-center item-title">
                                                    New theme
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                                <div
                                    class="col d-flex"
                                    on:click=move |_| state.set(State::ImportTheme)
                                >
                                    <div class="row item-box align-self-center">
                                        <div class="col-auto d-flex playlist-modal-item-container w-100">

                                            <div class="row w-100">
                                                <div class="col d-flex justify-content-center w-100">
                                                    <div class="item-icon">
                                                        <ImportThemeIcon />
                                                    </div>
                                                </div>
                                            </div>
                                            <div class="row">
                                                <div class="col d-flex justify-content-center item-title">
                                                    Import theme
                                                </div>
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                        .into_view()
                }
                State::NewTheme => {
                    let new_theme = create_rw_signal(ThemeDetails::new());
                    let active = create_rw_signal(false);
                    let show_color_picker = create_rw_signal((false, 0));
                    let colors = create_read_slice(new_theme, move |t| t.theme.clone());
                    let hex_code = create_rw_signal(String::new());
                    let color_picker_coords_x = create_rw_signal("0px".to_string());
                    let color_picker_coords_y = create_rw_signal("0px".to_string());
                    let color_picker_ref = create_node_ref();
                    let _ = on_click_outside(
                        color_picker_ref,
                        move |_| {
                            show_color_picker.set((false, 0));
                        },
                    );
                    let get_color_name = move |index: i32, color: &ThemeItem| {
                        match index {
                            0 => ("Primary", color.primary.clone()),
                            1 => ("Secondary", color.secondary.clone()),
                            2 => ("Tertiary", color.tertiary.clone()),
                            3 => ("Text Primary", color.text_primary.clone()),
                            4 => ("Text Secondary", color.text_secondary.clone()),
                            5 => ("Text Inverse", color.text_inverse.clone()),
                            6 => ("Accent", color.accent.clone()),
                            _ => unreachable!(),
                        }
                    };
                    let save_theme = move || { save_theme(new_theme.get_untracked(), close_modal) };
                    create_effect(move |_| {
                        let (show, index) = show_color_picker.get();
                        if !show {
                            return;
                        }
                        let hex_code = hex_code.get();
                        new_theme
                            .update(move |theme| {
                                match index {
                                    0 => theme.theme.primary = hex_code,
                                    1 => theme.theme.secondary = hex_code,
                                    2 => theme.theme.tertiary = hex_code,
                                    3 => theme.theme.text_primary = hex_code,
                                    4 => theme.theme.text_secondary = hex_code,
                                    5 => theme.theme.text_inverse = hex_code,
                                    6 => theme.theme.accent = hex_code,
                                    _ => unreachable!(),
                                }
                            });
                    });
                    view! {
                        <div class="container-fluid h-100 w-100">
                            {move || {
                                let (show, index) = show_color_picker.get();
                                let (_, color) = get_color_name(index, &colors.get_untracked());
                                if show {
                                    view! {
                                        <div
                                            class="color-picker-wrapper"
                                            style:top=color_picker_coords_y
                                            style:left=color_picker_coords_x
                                        >
                                            <ColorPicker
                                                hex_code_setter=hex_code.write_only()
                                                force_color=color
                                                node_ref=color_picker_ref
                                            />
                                        </div>
                                    }
                                        .into_view()
                                } else {
                                    view! {}.into_view()
                                }
                            }} <div class="row no-gutters" no-gutters="">
                                <div class="col h-100">
                                    <div class="row no-gutters metadata mb-3" no-gutters="">
                                        <input
                                            class="form-control theme-title"
                                            type="text"
                                            placeholder="Theme Name"
                                            maxlength="20"
                                            id="__BVID__465"
                                            prop:value=move || { new_theme.get().name }
                                            on:input=move |ev| {
                                                new_theme
                                                    .update(move |t| {
                                                        t.name = event_target_value(&ev);
                                                    });
                                            }
                                        />
                                        <input
                                            class="form-control theme-title"
                                            type="text"
                                            placeholder="Author"
                                            maxlength="20"
                                            id="__BVID__466"
                                            prop:value=move || {
                                                new_theme.get().author.unwrap_or_default()
                                            }
                                            on:input=move |ev| {
                                                new_theme
                                                    .update(move |t| {
                                                        t.author = Some(event_target_value(&ev));
                                                    });
                                            }
                                        />
                                    </div>
                                    <div class="row no-gutters" no-gutters="">
                                        <div class="col-6 preview-col" cols="6">
                                            <div
                                                class="row no-gutters preview mb-5 w-100 h-100"
                                                no-gutters=""
                                            >
                                                {move || {
                                                    let new_theme = new_theme.get();
                                                    view! {
                                                        <ThemeViewIcon active=active.read_only() theme=new_theme />
                                                    }
                                                }}
                                            </div>

                                        </div>
                                        <div class="col-auto color-col ml-5">
                                            <div class="row no-gutters" no-gutters="">
                                                <div
                                                    class="col-auto align-self-center title d-flex preference-title"

                                                    align-self="center"
                                                >
                                                    Colors
                                                </div>
                                                <div class="col-auto ml-2">
                                                    <Tooltip>{"Change colors"}</Tooltip>
                                                </div>
                                            </div>
                                            <table>
                                                {move || {
                                                    let mut ret = vec![];
                                                    let color = colors.get();
                                                    for i in 0..=6 {
                                                        let (color_name, color) = get_color_name(i, &color);
                                                        ret.push(
                                                            view! {
                                                                <tr height="60">
                                                                    <td class="color-title pr-5" title=color_name>
                                                                        {color_name}
                                                                    </td>
                                                                    <td
                                                                        class="pr-4"
                                                                        on:click=move |ev| {
                                                                            color_picker_coords_x.set(format!("{}px", ev.page_x()));
                                                                            color_picker_coords_y.set(format!("{}px", ev.page_y()));
                                                                            show_color_picker.set((true, i));
                                                                        }
                                                                    >
                                                                        <div class="color-box" style:background=color></div>
                                                                    </td>
                                                                    <td></td>
                                                                </tr>
                                                            },
                                                        );
                                                    }
                                                    ret.collect_view()
                                                }}
                                            </table>
                                        </div>
                                    </div>
                                    <div class="row no-gutters mt-4" no-gutters="">
                                        <div
                                            class="col-auto align-self-center title d-flex preference-title"

                                            align-self="center"
                                        >
                                            Custom CSS
                                        </div>
                                        <div class="col-auto ml-2">
                                            <Tooltip>{"Optional"}</Tooltip>
                                        </div>
                                    </div>
                                    <div
                                        class="row no-gutters custom-css-background w-100 mt-2 d-flex"
                                        no-gutters=""
                                    >
                                        <div
                                            class="row no-gutters mt-3 custom-css-item"
                                            no-gutters=""
                                        >
                                            <div
                                                class="col-auto align-self-center ml-4 folder-icon"

                                                align-self="center"
                                            >
                                                <FolderIcon />
                                            </div>
                                            <div
                                                class="col-auto align-self-center ml-3 justify-content-start"
                                                id="3065be4c-d78c-4e72-b485-7b4a6ff54e21"

                                                align-self="center"
                                                title=""
                                            >
                                                <div class="item-text text-truncate"></div>
                                            </div>
                                        </div>
                                        <div
                                            class="col-auto align-self-center ml-4 custom-css-cross-icon"

                                            align-self="center"
                                        >
                                            <FolderIcon />
                                        </div>
                                    </div>
                                    <div class="row justify-content-end mt-5 mr-4" align-h="end">
                                        <button
                                            class="btn btn-secondary cancel-button mr-4"
                                            type="button"
                                            on:click=move |_| close_modal()
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            class="btn btn-secondary confirm-button"
                                            type="button"
                                            on:click=move |_| { save_theme() }
                                        >
                                            Save
                                        </button>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                        .into_view()
                }
                State::ImportTheme => view! {}.into_view(),
            }}
        </GenericModal>
    }
}
