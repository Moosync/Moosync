use extensions_proto::moosync::types::{ExtensionDetail, ExtensionProviderScope};
use slint::{ComponentHandle, ModelRc, VecModel, Weak};
use state_manager::StateManager;
use tracing::Instrument;

use crate::{AccountItem, AccountsProps, AppCallbacks, MainWindow, OAuthState};

pub struct AccountsHandler;

impl AccountsHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn setup(main_window: &'static MainWindow, state_manager: &'static StateManager) {
        Self::setup_callbacks(main_window, state_manager);
        Self::setup_listeners(main_window, state_manager);
        Self::fetch_and_render_accounts(main_window.as_weak(), state_manager);
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn open_browser(url: &str) { let _ = open::that_detached(url); }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn fetch_and_render_accounts(
        main_window_weak: Weak<MainWindow>,
        state_manager: &'static StateManager,
    ) {
        let state_manager = state_manager.clone();
        tokio::spawn(
            async move {
                let extension_handler = state_manager.get_extension_handler().await;
                let extensions = extension_handler
                    .get_extensions_with_scope(ExtensionProviderScope::Accounts)
                    .await;

                let mut raw_accounts = Vec::new();
                for ext in extensions {
                    let ext_detail: ExtensionDetail = (&*ext).into();
                    for acc in ext.get_accounts() {
                        raw_accounts.push((acc, ext_detail.clone()));
                    }
                }

                let _ = slint::invoke_from_event_loop(move || {
                    if let Some(main_window) = main_window_weak.upgrade() {
                        let account_items: Vec<AccountItem> = raw_accounts
                            .into_iter()
                            .map(|(acc, ext_detail)| AccountItem::from((acc, Some(&ext_detail))))
                            .collect();
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
    fn setup_listeners(main_window: &'static MainWindow, state_manager: &'static StateManager) {
        let main_window_weak = main_window.as_weak();
        tokio::spawn(
            async move {
                let extension_handler = state_manager.get_extension_handler().await;
                let _cancel = extension_handler.on_accounts_updated({
                    let main_window_weak = main_window_weak.clone();
                    move |_| {
                        Self::fetch_and_render_accounts(main_window_weak.clone(), state_manager);
                    }
                });
            }
            .in_current_span(),
        );
    }

    #[tracing::instrument(level = "debug", skip_all)]
    fn setup_callbacks(main_window: &'static MainWindow, state_manager: &'static StateManager) {
        let main_window_weak = main_window.as_weak();
        let sm = state_manager.clone();
        main_window
            .global::<AppCallbacks>()
            .on_account_login(move |package_name, account_id| {
                let main_window_weak = main_window_weak.clone();
                let sm = sm.clone();
                let package_name = package_name.to_string();
                let account_id = account_id.to_string();
                tokio::spawn(
                    async move {
                        let extension_handler = sm.get_extension_handler().await;
                        match extension_handler.get_extension(&package_name) {
                            Ok(ext) => {
                                match ext
                                    .perform_account_login(
                                        extensions_proto::moosync::types::PerformAccountLoginRequest {
                                            account_id,
                                            login_status: true,
                                        },
                                    )
                                    .await
                                {
                                    Ok(resp) => {
                                        let url = resp.status;
                                        #[cfg(not(test))]
                                        Self::open_browser(&url);

                                        let url_shared: slint::SharedString = url.into();
                                        let _ = slint::invoke_from_event_loop(move || {
                                            if let Some(main_window) = main_window_weak.upgrade() {
                                                let oauth_state = main_window.global::<OAuthState>();
                                                oauth_state.set_oauth_url(url_shared);
                                                oauth_state.set_oauth_code(slint::SharedString::default());
                                                oauth_state.set_show_oauth_modal(true);
                                            }
                                        });
                                    }
                                    Err(e) => {
                                        tracing::error!("Account login error for {}: {:?}", package_name, e);
                                    }
                                }
                            }
                            Err(e) => {
                                tracing::error!("Failed to get extension {}: {:?}", package_name, e);
                            }
                        }
                    }
                    .in_current_span(),
                );
            });

        let sm = state_manager.clone();
        main_window
            .global::<AppCallbacks>()
            .on_account_logout(move |package_name, account_id| {
                let sm = sm.clone();
                let package_name = package_name.to_string();
                let account_id = account_id.to_string();
                tokio::spawn(
                    async move {
                        let extension_handler = sm.get_extension_handler().await;
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

        let main_window_weak = main_window.as_weak();
        let sm = state_manager.clone();
        main_window
            .global::<AppCallbacks>()
            .on_submit_oauth_code(move |code| {
                let sm = sm.clone();
                let code_str = if !code.starts_with("moosync://") {
                    format!("moosync://{}", code)
                } else {
                    code.to_string()
                };
                tracing::debug!("Using code str: {}", code_str);
                let main_window_weak = main_window_weak.clone();
                tokio::spawn(
                    async move {
                        match sm.handle_oauth_callback(&code_str).await {
                            Ok(_) => {
                                let _ = slint::invoke_from_event_loop(move || {
                                    if let Some(main_window) = main_window_weak.upgrade() {
                                        main_window
                                            .global::<OAuthState>()
                                            .set_show_oauth_modal(false);
                                    }
                                });
                            }
                            Err(e) => {
                                tracing::error!("Failed to handle oauth callback code: {:?}", e);
                            }
                        }
                    }
                    .in_current_span(),
                );
            });
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn setup_uri_scheme(
        _main_window: &'static MainWindow,
        _state_manager: &'static StateManager,
    ) {
        #[cfg(not(any(target_os = "android", test)))]
        {
            let mw_weak = _main_window.as_weak();
            let sm = _state_manager.clone();
            sysuri::register_handler(
                "moosync",
                sysuri::FnHandler::new(move |uri: &str| {
                    if let Some(mw) = mw_weak.upgrade() {
                        Self::handle_deep_link(uri, &sm, &mw);
                    }
                }),
            );

            if let Ok(exe) = std::env::current_exe() {
                let scheme = sysuri::UriScheme::new("moosync", "Moosync Protocol", exe);
                let _ = sysuri::register(&scheme);
            }

            let _ = sysuri::should_handle_uri();
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn handle_deep_link(url: &str, state_manager: &StateManager, main_window: &MainWindow) {
        let sm = state_manager.clone();
        let url_str = url.to_string();
        let main_window_weak = main_window.as_weak();
        tokio::spawn(
            async move {
                match sm.handle_oauth_callback(&url_str).await {
                    Ok(_) => {
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(main_window) = main_window_weak.upgrade() {
                                main_window
                                    .global::<OAuthState>()
                                    .set_show_oauth_modal(false);
                            }
                        });
                    }
                    Err(e) => {
                        tracing::error!("Failed to handle deep link callback {}: {:?}", url_str, e);
                    }
                }
            }
            .in_current_span(),
        );
    }
}
