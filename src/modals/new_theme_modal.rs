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

use std::collections::HashMap;

use crate::{
    components::color_picker::ColorPicker,
    icons::{
        cross_icon::CrossIcon, folder_icon::FolderIcon, import_theme_icon::ImportThemeIcon,
        new_theme_button_icon::NewThemeButtonIcon, theme_view_icon::ThemeViewIcon,
        tooltip::Tooltip,
    },
    store::{modal_store::ModalStore, ui_store::UiStore},
    utils::{
        invoke::{download_theme, get_themes_manifest},
        prefs::{import_theme, open_file_browser_single, save_theme},
    },
};
use leptos::{component, prelude::*, task::spawn_local, view, IntoView};
use leptos_use::on_click_outside;
use types::{
    themes::{ThemeDetails, ThemeItem},
    ui::themes::ThemeModalState,
    window::DialogFilter,
};

use crate::modals::common::GenericModal;

#[tracing::instrument(level = "debug", skip())]
#[component]
pub fn NewThemeModal(#[prop()] initial_state: Box<ThemeModalState>) -> impl IntoView {
    let state = RwSignal::new(initial_state);
    let theme_path = RwSignal::new(String::new());
    Effect::new(move || {
        let state = state.get();

        if let ThemeModalState::ImportTheme = *state {
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

    let modal_store: RwSignal<ModalStore> = expect_context();
    let close_modal = move || {
        modal_store.update(|s| s.clear_active_modal());
    };

    Effect::new(move || {
        let theme_path = theme_path.get();
        if theme_path.is_empty() {
            return;
        }

        import_theme(theme_path, close_modal);
    });

    let ui_store = expect_context::<RwSignal<UiStore>>();
    let is_mobile = create_read_slice(ui_store, |u| u.get_is_mobile()).get();

    view! {
        <GenericModal size=move || {
            {
                match *state.get() {
                    ThemeModalState::None => "modal-md",
                    ThemeModalState::NewTheme(_) => "modal-xl",
                    ThemeModalState::ImportTheme => "modal-lg",
                    ThemeModalState::DiscoverTheme => "modal-xl",
                }
            }
                .into()
        }>

            {move || match *state.get() {
                ThemeModalState::None => {
                    view! {
                        <div class="container">
                            <div class="row no-gutters">
                                <div class="col-6 pr-2" style="padding-right: 7.5px;">
                                    <div
                                        class="embed-responsive embed-responsive-1by1"
                                        style="border-radius: 20px;"
                                    >
                                        <div
                                            class="embed-responsive-item d-flex flex-column justify-content-center align-items-center"
                                            style="background-color: var(--secondary);"
                                            on:click=move |_| {
                                                state
                                                    .set(
                                                        Box::new(
                                                            ThemeModalState::NewTheme(Box::new(ThemeDetails::new())),
                                                        ),
                                                    )
                                            }
                                        >
                                            <div class="item-icon mb-2">
                                                <NewThemeButtonIcon />
                                            </div>
                                            <div class="item-title text-center">New theme</div>
                                        </div>
                                    </div>
                                </div>
                                <div class="col-6 pl-2" style="padding-left: 7.5px;">
                                    <div
                                        class="embed-responsive embed-responsive-1by1"
                                        style="border-radius: 20px;"
                                    >
                                        <div
                                            class="embed-responsive-item d-flex flex-column justify-content-center align-items-center"
                                            style="background-color: var(--secondary);"
                                            on:click=move |_| {
                                                state.set(Box::new(ThemeModalState::DiscoverTheme))
                                            }
                                        >
                                            <div class="item-icon mb-2">
                                                <ImportThemeIcon />
                                            </div>
                                            <div class="item-title text-center">Discover themes</div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        </div>
                    }
                        .into_any()
                }
                ThemeModalState::NewTheme(theme) => {
                    let new_theme = RwSignal::new(theme);
                    let active = RwSignal::new(false);
                    let show_color_picker = RwSignal::new((false, 0));
                    let colors = create_read_slice(new_theme, move |t| t.theme.clone());
                    let hex_code = RwSignal::new(String::new());
                    let color_picker_coords_x = RwSignal::new("0px".to_string());
                    let color_picker_coords_y = RwSignal::new("0px".to_string());
                    let color_picker_ref = NodeRef::new();
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
                    Effect::new(move || {
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
                                        .into_any()
                                } else {
                                    ().into_any()
                                }
                            }} <div class="row no-gutters">
                                <div class="col h-100">
                                    <div class="row no-gutters metadata mb-3">
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
                                        <div class="col-6 preview-col">
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
                                                                <tr>
                                                                    <td class="color-title pr-5" title=color_name>
                                                                        {color_name}
                                                                    </td>
                                                                    <td
                                                                        class="pr-4"
                                                                        on:click=move |ev| {
                                                                            if !is_mobile {
                                                                                color_picker_coords_x.set(format!("{}px", ev.page_x()));
                                                                            } else {
                                                                                color_picker_coords_x.set("0px".into());
                                                                            }
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
                                                on:click=move |_| {
                                                    spawn_local(async move {
                                                        if let Ok(res) = crate::utils::invoke::open_file_browser(
                                                                false,
                                                                false,
                                                                vec![
                                                                    DialogFilter {
                                                                        name: "Css".into(),
                                                                        extensions: vec!["css".into()],
                                                                    },
                                                                ],
                                                            )
                                                            .await
                                                        {
                                                            if let Some(file) = res.first() {
                                                                new_theme
                                                                    .update(|t| {
                                                                        t.theme.custom_css = Some(file.path.clone());
                                                                    });
                                                            }
                                                        }
                                                    });
                                                }
                                                align-self="center"
                                            >
                                                <FolderIcon />
                                            </div>
                                            <div class="col-auto align-self-center ml-3 justify-content-start">
                                                <div
                                                    class="item-text text-truncate theme-custom-css"
                                                    title=move || {
                                                        let theme = new_theme.get();
                                                        theme.theme.custom_css
                                                    }
                                                >
                                                    {move || {
                                                        let theme = new_theme.get();
                                                        theme.theme.custom_css
                                                    }}
                                                </div>
                                            </div>
                                        </div>
                                        <div
                                            class="col-auto align-self-center ml-4 custom-css-cross-icon"
                                            on:click=move |_| {
                                                new_theme
                                                    .update(|t| {
                                                        t.theme.custom_css = None;
                                                    });
                                            }
                                            align-self="center"
                                        >
                                            <CrossIcon />
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
                        .into_any()
                }
                ThemeModalState::ImportTheme => ().into_any(),
                ThemeModalState::DiscoverTheme => {
                    {
                        let themes = RwSignal::new(HashMap::new());
                        let (active, _) = signal(false);
                        spawn_local(async move {
                            if let Ok(manifest) = get_themes_manifest().await {
                                tracing::debug!("Got themes manifest {:?}", manifest);
                                themes.set(manifest);
                            }
                        });
                        let install_theme = move |url: String| {
                            spawn_local(async move {
                                if let Err(e) = download_theme(url).await {
                                    tracing::error!("Failed to download theme: {:?}", e);
                                } else {
                                    close_modal();
                                }
                            });
                        };
                        if !is_mobile {
                            view! {
                                <div class="container-fluid h-100 w-100">
                                    <div class="row no-gutters">
                                        <For
                                            each=move || themes.get()
                                            key=|(key, _)| key.clone()
                                            children=move |(key, theme)| {
                                                view! {
                                                    <div class="col-xl-3 col-5 p-2">
                                                        <div class="theme-component-container">
                                                            <div
                                                                class="theme-download-wrapper"
                                                                on:click=move |_| {
                                                                    install_theme(key.clone());
                                                                }
                                                            >
                                                                <ImportThemeIcon />
                                                            </div>
                                                            <ThemeViewIcon
                                                                active=active
                                                                theme=Box::new(theme.clone())
                                                            />
                                                            <div class="theme-title-text">{theme.name}</div>
                                                            <div class="theme-author">{theme.author}</div>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                    <div class="row no-gutters d-flex justify-contents-end">
                                        <div class="col col-auto">
                                            <button
                                                class="btn btn-secondary create-button ml-3"
                                                on:click=move |_| {
                                                    state.set(Box::new(ThemeModalState::ImportTheme));
                                                }
                                            >
                                                Install from file
                                            </button>
                                        </div>
                                    </div>
                                </div>
                            }
                                .into_any()
                        } else {
                            view! {
                                <div class="container-fluid">
                                    <div class="d-flex flex-column">
                                        <For
                                            each=move || themes.get()
                                            key=|(key, _)| key.clone()
                                            children=move |(key, theme)| {
                                                view! {
                                                    <div
                                                        class="d-flex align-items-center p-3 mb-2"
                                                        style="cursor: pointer; background-color: transparent; border: none;"
                                                        on:click=move |_| {
                                                            install_theme(key.clone());
                                                        }
                                                    >
                                                        <div class="mr-3" style="min-width: 200px;">
                                                            <ThemeViewIcon
                                                                active=active
                                                                theme=Box::new(theme.clone())
                                                            />
                                                        </div>
                                                        <div class="d-flex flex-column">
                                                            <div class="theme-title-text font-weight-bold text-left">
                                                                {theme.name}
                                                            </div>
                                                            <div class="theme-author text-muted text-left">
                                                                {theme.author}
                                                            </div>
                                                        </div>
                                                    </div>
                                                }
                                            }
                                        />
                                    </div>
                                </div>
                            }
                                .into_any()
                        }
                    }
                        .into_any()
                }
            }}
        </GenericModal>
    }
}
