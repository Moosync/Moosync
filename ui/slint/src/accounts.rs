use std::path::{Path, PathBuf};

use extensions::ExtensionError;
use extensions_proto::moosync::types::{ExtensionProviderScope, PerformAccountLoginRequest};
use pulldown_cmark::{Event, Options, Parser, Tag, TagEnd};
use slint::{
    ComponentHandle, Image, ModelRc, Rgba8Pixel, SharedPixelBuffer, SharedString, StyledText,
    VecModel, Weak,
};
use state_manager::{StateManager, oauth_handler::ProcessedOauthResponse};
use tracing::Instrument;

#[cfg(target_os = "android")]
use crate::utils::android::get_android_context;
use crate::{
    AccountItem, AccountsProps, AppCallbacks, MainWindow, MarkdownPart, OAuthState,
    utils::image::download_image_as_pixel_buffer,
};

#[derive(Debug, Clone)]
pub(crate) enum ParsedMarkdownPart {
    Text(String),
    LocalImage(PathBuf),
    RemoteImage(SharedPixelBuffer<Rgba8Pixel>),
}

pub struct AccountsHandler;

impl AccountsHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn setup(main_window: &MainWindow, state_manager: &StateManager) {
        Self::setup_callbacks(main_window, state_manager);
        Self::setup_listeners(main_window, state_manager);
        Self::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn open_browser(url: &str) { let _ = open::that_detached(url); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn fetch_and_render_accounts(
        main_window_weak: Weak<MainWindow>,
        state_manager: StateManager,
    ) {
        tokio::spawn(
            async move {
                let extension_handler = state_manager.get_extension_handler();
                let extensions = extension_handler
                    .get_extensions_with_scope(ExtensionProviderScope::Accounts)
                    .await;

                let mut raw_accounts = Vec::new();
                for ext in &extensions {
                    for acc in ext.get_accounts() {
                        raw_accounts.push(acc);
                    }
                }

                tracing::debug!(
                    "fetch_and_render_accounts: found {} extensions with Accounts scope, {} raw_accounts",
                    extensions.len(),
                    raw_accounts.len()
                );

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        let account_items: Vec<AccountItem> =
                            raw_accounts.into_iter().map(AccountItem::from).collect();
                        tracing::debug!(
                            "fetch_and_render_accounts: setting AccountsProps with {} items",
                            account_items.len()
                        );
                        main_window
                            .global::<AccountsProps>()
                            .set_accounts(ModelRc::new(VecModel::from(account_items)));
                    }
                });
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_listeners(main_window: &MainWindow, state_manager: &StateManager) {
        let main_window_weak = main_window.as_weak();
        let state_manager = state_manager.clone();
        tokio::spawn(
            async move {
                let extension_handler = state_manager.get_extension_handler();
                let _cancel_accounts = extension_handler.on_accounts_updated.listen({
                    let main_window_weak = main_window_weak.clone();
                    let sm = state_manager.clone();
                    move |_| {
                        Self::fetch_and_render_accounts(main_window_weak.clone(), sm.clone());
                    }
                });
                let _cancel_extensions = extension_handler.on_extensions_updated.listen({
                    let main_window_weak = main_window_weak.clone();
                    let sm = state_manager.clone();
                    move |_| {
                        Self::fetch_and_render_accounts(main_window_weak.clone(), sm.clone());
                    }
                });
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn handle_account_login(
        package_name: String,
        account_id: String,
        state_manager: StateManager,
        main_window_weak: slint::Weak<MainWindow>,
    ) {
        let extension_handler = state_manager.get_extension_handler();
        let ext = match extension_handler.get_extension(&package_name) {
            Ok(ext) => ext,
            Err(e) => {
                tracing::error!("Failed to get extension {}: {:?}", package_name, e);
                return;
            }
        };

        let resp = match ext
            .perform_account_login(PerformAccountLoginRequest {
                account_id,
                login_status: true,
            })
            .await
        {
            Ok(resp) => resp,
            Err(e) => {
                tracing::error!("Account login error for {}: {:?}", package_name, e);
                return;
            }
        };

        tracing::debug!("perform_account_login response url: {}", resp.url);
        let url = resp.url;
        let device_code = resp.device_code.unwrap_or_default();
        #[cfg(not(test))]
        if !url.is_empty() {
            Self::open_browser(&url);
        }

        let pkg_shared: SharedString = package_name.into();
        let url_shared: SharedString = url.into();
        let device_code_shared: SharedString = device_code.clone().into();
        let modal_title: SharedString = if device_code.is_empty() {
            "Account Login".into()
        } else {
            "Device Login".into()
        };
        let _ = slint::invoke_from_event_loop(move || {
            if let Some(main_window) = main_window_weak.upgrade() {
                let oauth_state = main_window.global::<OAuthState>();
                oauth_state.set_active_package_name(pkg_shared);
                oauth_state.set_modal_title(modal_title);
                oauth_state.set_oauth_url(url_shared);
                oauth_state.set_oauth_code(SharedString::default());
                oauth_state.set_device_code(device_code_shared);
                oauth_state.set_has_extra_content(false);
                oauth_state.set_markdown_parts(ModelRc::new(VecModel::default()));
                oauth_state.set_error_message(SharedString::default());
                oauth_state.set_show_oauth_modal(true);
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_login_callback(main_window: &MainWindow, state_manager: &StateManager) {
        let main_window_weak = main_window.as_weak();
        let state_manager = state_manager.clone();
        main_window
            .global::<AppCallbacks>()
            .on_account_login(move |package_name, account_id| {
                tracing::debug!(
                    "on_account_login triggered: pkg={}, acc={}",
                    package_name,
                    account_id
                );
                let main_window_weak = main_window_weak.clone();
                let package_name = package_name.to_string();
                let account_id = account_id.to_string();
                let state_manager = state_manager.clone();
                tokio::spawn(
                    Self::handle_account_login(
                        package_name,
                        account_id,
                        state_manager,
                        main_window_weak,
                    )
                    .in_current_span(),
                );
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_logout_callback(main_window: &MainWindow, state_manager: &StateManager) {
        let state_manager = state_manager.clone();
        main_window
            .global::<AppCallbacks>()
            .on_account_logout(move |package_name, account_id| {
                let package_name = package_name.to_string();
                let account_id = account_id.to_string();
                let state_manager = state_manager.clone();
                tokio::spawn(
                    async move {
                        let extension_handler = state_manager.get_extension_handler();
                        if let Ok(ext) = extension_handler.get_extension(&package_name) {
                            if let Err(e) = ext
                                .perform_account_login(
                                    extensions_proto::moosync::types::PerformAccountLoginRequest {
                                        account_id,
                                        login_status: false,
                                    },
                                )
                                .await
                            {
                                tracing::error!(
                                    "Account logout error for {}: {:?}",
                                    package_name,
                                    e
                                );
                            }
                        }
                    }
                    .in_current_span(),
                );
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) fn build_callback_url(
        code: &str,
        active_package_name: &str,
        state_manager: &StateManager,
    ) -> String {
        let trimmed = code.trim();
        if trimmed.starts_with("moosync://")
            || trimmed.starts_with("http://")
            || trimmed.starts_with("https://")
        {
            return trimmed.to_string();
        }

        let host = state_manager
            .get_oauth_manager()
            .find_host_for_package(active_package_name)
            .unwrap_or_else(|| active_package_name.to_string());

        if trimmed.is_empty() {
            format!("moosync://{}", host)
        } else if trimmed.starts_with('?') {
            format!("moosync://{}{}", host, trimmed)
        } else if trimmed.starts_with("code=") {
            format!("moosync://{}?{}", host, trimmed)
        } else {
            format!("moosync://{}?code={}", host, trimmed)
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    async fn resolve_oauth_image(
        dest_url: &str,
        extension_path: &Path,
    ) -> Option<ParsedMarkdownPart> {
        if dest_url.starts_with("https://") {
            return download_image_as_pixel_buffer(dest_url)
                .await
                .map(ParsedMarkdownPart::RemoteImage);
        }

        if dest_url.contains("://") || dest_url.starts_with('/') || dest_url.starts_with('\\') {
            tracing::warn!("Ignoring non-https or absolute image URL: {}", dest_url);
            return None;
        }

        let canonical_ext = match extension_path.canonicalize() {
            Ok(path) => path,
            Err(err) => {
                tracing::error!(
                    "Failed to canonicalize extension path {:?}: {:?}",
                    extension_path,
                    err
                );
                return None;
            }
        };

        let candidate = extension_path.join(dest_url);
        let canonical_img = match candidate.canonicalize() {
            Ok(path) => path,
            Err(err) => {
                tracing::warn!("Failed to resolve local image {:?}: {:?}", candidate, err);
                return None;
            }
        };

        if canonical_img.starts_with(&canonical_ext) && canonical_img.is_file() {
            Some(ParsedMarkdownPart::LocalImage(canonical_img))
        } else {
            tracing::warn!(
                "Rejected image path outside extension directory: {:?}",
                canonical_img
            );
            None
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) async fn parse_oauth_markdown(
        raw_markdown: &str,
        extension_path: &Path,
    ) -> Vec<ParsedMarkdownPart> {
        let mut options = Options::empty();
        options.insert(Options::ENABLE_STRIKETHROUGH);

        let mut image_spans: Vec<(usize, usize, String)> = Vec::new();
        let mut img_start = None;
        let mut pending_img_url = None;

        for (event, range) in Parser::new_ext(raw_markdown, options).into_offset_iter() {
            match event {
                Event::Start(Tag::Image { dest_url, .. }) => {
                    img_start = Some(range.start);
                    pending_img_url = Some(dest_url.to_string());
                }
                Event::End(TagEnd::Image) => {
                    if let (Some(start), Some(url)) = (img_start.take(), pending_img_url.take()) {
                        image_spans.push((start, range.end, url));
                    }
                }
                _ => {}
            }
        }

        let mut parts = Vec::new();
        let mut cursor = 0;
        for (start, end, img_url) in image_spans {
            let slice = raw_markdown[cursor..start].trim();
            if !slice.is_empty() {
                parts.push(ParsedMarkdownPart::Text(slice.to_string()));
            }
            if let Some(img_part) = Self::resolve_oauth_image(&img_url, extension_path).await {
                parts.push(img_part);
            }
            cursor = end;
        }

        let tail = raw_markdown[cursor..].trim();
        if !tail.is_empty() {
            parts.push(ParsedMarkdownPart::Text(tail.to_string()));
        }

        parts
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn build_slint_markdown_parts(parts: Vec<ParsedMarkdownPart>) -> Vec<MarkdownPart> {
        let mut slint_parts = Vec::new();
        for part in parts {
            match part {
                ParsedMarkdownPart::Text(md) => {
                    let styled = StyledText::from_markdown(&md).unwrap_or_else(|err| {
                        tracing::warn!("Failed to parse markdown for StyledText: {:?}", err);
                        StyledText::from_plain_text(&md)
                    });
                    slint_parts.push(MarkdownPart {
                        is_image: false,
                        text: styled,
                        image_src: Image::default(),
                    });
                }
                ParsedMarkdownPart::LocalImage(path) => match Image::load_from_path(&path) {
                    Ok(img) => slint_parts.push(MarkdownPart {
                        is_image: true,
                        text: StyledText::default(),
                        image_src: img,
                    }),
                    Err(err) => {
                        tracing::error!("Failed to load markdown image {:?}: {:?}", path, err);
                    }
                },
                ParsedMarkdownPart::RemoteImage(buf) => slint_parts.push(MarkdownPart {
                    is_image: true,
                    text: StyledText::default(),
                    image_src: Image::from_rgba8(buf),
                }),
            }
        }
        slint_parts
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub(crate) async fn process_oauth_callback_result(
        res: Result<ProcessedOauthResponse, ExtensionError>,
        main_window_weak: Weak<MainWindow>,
    ) {
        enum UiOutcome {
            Close,
            ShowExtra(Vec<ParsedMarkdownPart>),
            ShowError(String),
        }

        let outcome = match res {
            Ok(resp) => {
                if let Some(raw_md) = resp.extra_content {
                    let parts = Self::parse_oauth_markdown(&raw_md, &resp.extension_path).await;
                    UiOutcome::ShowExtra(parts)
                } else {
                    UiOutcome::Close
                }
            }
            Err(err) => {
                tracing::error!("Failed to handle oauth callback: {:?}", err);
                UiOutcome::ShowError(err.to_string())
            }
        };

        let _ = slint::invoke_from_event_loop(move || {
            let Some(main_window) = main_window_weak.upgrade() else {
                return;
            };
            let oauth_state = main_window.global::<OAuthState>();

            match outcome {
                UiOutcome::Close => oauth_state.set_show_oauth_modal(false),
                UiOutcome::ShowExtra(parts) => {
                    let slint_parts = Self::build_slint_markdown_parts(parts);
                    oauth_state.set_markdown_parts(ModelRc::new(VecModel::from(slint_parts)));
                    oauth_state.set_has_extra_content(true);
                    oauth_state.set_error_message(SharedString::default());
                    oauth_state.set_show_oauth_modal(true);
                }
                UiOutcome::ShowError(err_msg) => {
                    oauth_state.set_error_message(err_msg.into());
                    oauth_state.set_show_oauth_modal(true);
                }
            }
        });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_oauth_code_callback(main_window: &MainWindow, state_manager: &StateManager) {
        let main_window_weak = main_window.as_weak();
        let state_manager = state_manager.clone();
        main_window
            .global::<AppCallbacks>()
            .on_submit_oauth_code(move |code| {
                let active_pkg = main_window_weak
                    .upgrade()
                    .map(|mw| {
                        mw.global::<OAuthState>()
                            .get_active_package_name()
                            .to_string()
                    })
                    .unwrap_or_default();
                let code_str = Self::build_callback_url(&code, &active_pkg, &state_manager);
                tracing::debug!("Using code str: {}", code_str);
                let main_window_weak = main_window_weak.clone();
                let state_manager = state_manager.clone();
                tokio::spawn(
                    async move {
                        let res = state_manager.handle_oauth_callback(&code_str).await;
                        Self::process_oauth_callback_result(res, main_window_weak).await;
                    }
                    .in_current_span(),
                );
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_callbacks(main_window: &MainWindow, state_manager: &StateManager) {
        Self::setup_login_callback(main_window, state_manager);
        Self::setup_logout_callback(main_window, state_manager);
        Self::setup_oauth_code_callback(main_window, state_manager);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn setup_uri_scheme(main_window: &MainWindow, state_manager: &StateManager) {
        Self::setup_deep_link(main_window, state_manager);
    }

    #[cfg(all(not(target_os = "android"), not(test)))]
    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_deep_link(main_window: &MainWindow, state_manager: &StateManager) {
        let mw_weak = main_window.as_weak();
        let state_manager = state_manager.clone();
        sysuri::register_handler(
            "moosync",
            sysuri::FnHandler::new(move |uri: &str| {
                if let Some(mw) = mw_weak.upgrade() {
                    Self::handle_deep_link(uri, state_manager.clone(), &mw);
                }
            }),
        );

        if let Ok(exe) = std::env::current_exe() {
            let scheme = sysuri::UriScheme::new("moosync", "Moosync Protocol", exe);
            if let Err(err) = sysuri::register(&scheme) {
                tracing::error!("Failed to register sysuri scheme: {:?}", err);
            }
        }

        if let Err(err) = sysuri::should_handle_uri() {
            tracing::error!("Failed to check sysuri handle: {:?}", err);
        }
    }

    #[cfg(all(not(target_os = "android"), test))]
    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_deep_link(_main_window: &MainWindow, _state_manager: &StateManager) {}

    #[cfg(target_os = "android")]
    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_deep_link(main_window: &MainWindow, state_manager: &StateManager) {
        let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<String>();
        match DEEP_LINK_TX.lock() {
            Ok(mut guard) => {
                *guard = Some(tx);
            }
            Err(err) => {
                tracing::error!("DEEP_LINK_TX lock poisoned: {:?}", err);
            }
        }
        let mw_weak = main_window.as_weak();
        let state_manager = state_manager.clone();
        tokio::spawn(
            async move {
                while let Some(url) = rx.recv().await {
                    if let Some(mw) = mw_weak.upgrade() {
                        Self::handle_deep_link(&url, state_manager.clone(), &mw);
                    }
                }
            }
            .in_current_span(),
        );

        let context = get_android_context();
        if let Err(err) = context.call_activity::<_, ()>("processPendingDeepLink", ()) {
            tracing::error!("Failed to process pending deep link via JNI: {:?}", err);
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn handle_deep_link(url: &str, state_manager: StateManager, main_window: &MainWindow) {
        let url_str = url.to_string();
        let main_window_weak = main_window.as_weak();
        tokio::spawn(
            async move {
                let res = state_manager.handle_oauth_callback(&url_str).await;
                Self::process_oauth_callback_result(res, main_window_weak).await;
            }
            .in_current_span(),
        );
    }
}

#[cfg(target_os = "android")]
static DEEP_LINK_TX: std::sync::Mutex<Option<tokio::sync::mpsc::UnboundedSender<String>>> =
    std::sync::Mutex::new(None);

#[cfg(target_os = "android")]
#[tracing::instrument(level = "debug", skip_all)]
pub fn on_deep_link(url: String) {
    tracing::info!("on_deep_link received: {}", url);
    match DEEP_LINK_TX.lock() {
        Ok(guard) => {
            if let Some(tx) = guard.as_ref() {
                if let Err(err) = tx.send(url.clone()) {
                    tracing::error!("Failed to forward deep link {}: {:?}", url, err);
                }
            } else {
                tracing::warn!(
                    "DEEP_LINK_TX not yet initialized when receiving deep link: {}",
                    url
                );
            }
        }
        Err(err) => {
            tracing::error!("DEEP_LINK_TX lock poisoned: {:?}", err);
        }
    }
}
