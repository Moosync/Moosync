use rstest::rstest;
use slint::{ComponentHandle, Model};

use crate::{
    AccountsProps,
    accounts::AccountsHandler,
    test_utils::{TestSlintSmContext, main_window, state_manager_fixture},
};

#[rstest]
#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_accounts_handler_setup_and_empty_render(
    state_manager_fixture: TestSlintSmContext,
    main_window: crate::MainWindow,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));

    AccountsHandler::setup(main_window, state_manager);

    assert_eq!(
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_count(),
        0
    );
}

#[rstest]
#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_accounts_handler_callbacks(
    state_manager_fixture: TestSlintSmContext,
    main_window: crate::MainWindow,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));

    AccountsHandler::setup(main_window, state_manager);

    let callbacks = main_window.global::<crate::AppCallbacks>();
    callbacks.invoke_account_login("nonexistent.pkg".into(), "acc_1".into());
    callbacks.invoke_account_logout("nonexistent.pkg".into(), "acc_1".into());
    callbacks.invoke_submit_oauth_code("test_code".into());
    callbacks.invoke_submit_oauth_code("moosync://test_code_with_scheme".into());

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    state_manager
        .get_extension_handler()
        .on_accounts_updated
        .emit(None);
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}

#[rstest]
#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_accounts_handler_deep_link_and_browser(
    state_manager_fixture: TestSlintSmContext,
    main_window: crate::MainWindow,
) {
    let TestSlintSmContext { sm, .. } = state_manager_fixture;
    let main_window = Box::leak(Box::new(main_window));
    let state_manager = Box::leak(Box::new(sm));

    AccountsHandler::handle_deep_link(
        "moosync://oauth_callback?code=abc",
        state_manager,
        main_window,
    );
    AccountsHandler::setup_uri_scheme(main_window, state_manager);

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
}
