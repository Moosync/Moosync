use i_slint_backend_testing::ElementHandle;
use slint::{ComponentHandle, Model};
use slint_app::{
    AccountsProps, AppCallbacks, MainWindow, OAuthState,
    accounts::AccountsHandler,
    test_utils::integration::{ExtensionFixture, integration_test, wait_until},
};
use state_manager::StateManager;

#[tracing::instrument(level = "debug", skip_all)]
async fn do_accounts_open_popup(main_window: MainWindow, state_manager: StateManager) {
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());
    tokio::time::sleep(std::time::Duration::from_millis(100)).await;

    let empty = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_count()
            == 0
    })
    .await;
    assert!(empty);

    let accounts_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Accounts").collect();
    assert_eq!(accounts_handles.len(), 1);
    assert!(accounts_handles[0].is_valid());

    accounts_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    assert_eq!(
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_count(),
        0
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_accounts_login_opens_oauth_modal(main_window: MainWindow, state_manager: StateManager) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        let accounts = main_window.global::<AccountsProps>().get_accounts();
        accounts.row_count() == 1
            && accounts
                .row_data(0)
                .is_some_and(|a| a.package_name == "rs.sample" && a.id == "sample_spotify")
    })
    .await;
    assert!(loaded);

    let accounts_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Accounts").collect();
    assert_eq!(accounts_handles.len(), 1);
    accounts_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_spotify".into());

    let modal_shown = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        oauth.get_show_oauth_modal()
            && oauth.get_oauth_url() == "https://example.com/oauth/authorize"
    })
    .await;
    assert!(modal_shown);
    assert!(main_window.global::<OAuthState>().get_show_oauth_modal());
    assert_eq!(
        main_window.global::<OAuthState>().get_oauth_url(),
        "https://example.com/oauth/authorize"
    );

    let submit_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Submit").collect();
    assert_eq!(submit_handles.len(), 1);
    assert!(submit_handles[0].is_valid());
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_accounts_submit_oauth_code_and_modal_dismissal(
    main_window: MainWindow,
    state_manager: StateManager,
) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        let accounts = main_window.global::<AccountsProps>().get_accounts();
        accounts.row_count() == 1
            && accounts
                .row_data(0)
                .is_some_and(|a| a.package_name == "rs.sample" && a.id == "sample_spotify")
    })
    .await;
    assert!(loaded);

    let accounts_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Accounts").collect();
    assert_eq!(accounts_handles.len(), 1);
    accounts_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_spotify".into());

    let modal_shown = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        oauth.get_show_oauth_modal()
            && oauth.get_oauth_url() == "https://example.com/oauth/authorize"
    })
    .await;
    assert!(modal_shown);

    main_window
        .global::<OAuthState>()
        .set_oauth_code("sample_callback?code=sample_auth_code_123".into());

    let submit_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Submit").collect();
    assert_eq!(submit_handles.len(), 1);
    assert!(submit_handles[0].is_valid());

    main_window
        .global::<AppCallbacks>()
        .invoke_submit_oauth_code("sample_callback?code=sample_auth_code_123".into());

    let logged_in = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        !oauth.get_show_oauth_modal()
            && main_window
                .global::<AccountsProps>()
                .get_accounts()
                .row_data(0)
                .is_some_and(|a| a.logged_in && a.username == "SampleUser")
    })
    .await;
    assert!(logged_in);

    let submit_handles_after: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Submit").collect();
    assert_eq!(submit_handles_after.len(), 0);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_accounts_logout(main_window: MainWindow, state_manager: StateManager) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        let accounts = main_window.global::<AccountsProps>().get_accounts();
        accounts.row_count() == 1
            && accounts
                .row_data(0)
                .is_some_and(|a| a.package_name == "rs.sample" && a.id == "sample_spotify")
    })
    .await;
    assert!(loaded);

    let accounts_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Accounts").collect();
    assert_eq!(accounts_handles.len(), 1);
    accounts_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_spotify".into());

    let _ = wait_until(|| main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    main_window
        .global::<OAuthState>()
        .set_oauth_code("sample_callback?code=sample_code".into());

    let submit_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Submit").collect();
    assert_eq!(submit_handles.len(), 1);

    main_window
        .global::<AppCallbacks>()
        .invoke_submit_oauth_code("sample_callback?code=sample_code".into());

    let logged_in = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_data(0)
            .is_some_and(|a| a.logged_in)
    })
    .await;
    assert!(logged_in);

    main_window
        .global::<AppCallbacks>()
        .invoke_account_logout("rs.sample".into(), "sample_spotify".into());

    let logged_out = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_data(0)
            .is_some_and(|a| !a.logged_in)
    })
    .await;
    assert!(logged_out);
    assert!(
        !main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_data(0)
            .unwrap()
            .logged_in
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_accounts_deep_link_login(main_window: MainWindow, state_manager: StateManager) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        let accounts = main_window.global::<AccountsProps>().get_accounts();
        accounts.row_count() == 1
            && accounts
                .row_data(0)
                .is_some_and(|a| a.package_name == "rs.sample" && a.id == "sample_spotify")
    })
    .await;
    assert!(loaded);

    let accounts_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Accounts").collect();
    assert_eq!(accounts_handles.len(), 1);
    accounts_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_spotify".into());

    let modal_shown = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        oauth.get_show_oauth_modal()
            && oauth.get_oauth_url() == "https://example.com/oauth/authorize"
    })
    .await;
    assert!(modal_shown);

    AccountsHandler::handle_deep_link(
        "moosync://sample_callback?code=deep_link_456",
        state_manager.clone(),
        &main_window,
    );

    let logged_in = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        !oauth.get_show_oauth_modal()
            && main_window
                .global::<AccountsProps>()
                .get_accounts()
                .row_data(0)
                .is_some_and(|a| a.logged_in && a.username == "SampleUser")
    })
    .await;
    assert!(logged_in);

    let submit_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Submit").collect();
    assert_eq!(submit_handles.len(), 0);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_oauth_modal_plain_code_routing(main_window: MainWindow, state_manager: StateManager) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_count()
            == 1
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_spotify".into());

    let modal_shown =
        wait_until(|| main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    assert!(modal_shown);

    main_window
        .global::<AppCallbacks>()
        .invoke_submit_oauth_code("plain_code_789".into());

    let logged_in = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        !oauth.get_show_oauth_modal()
            && main_window
                .global::<AccountsProps>()
                .get_accounts()
                .row_data(0)
                .is_some_and(|a| a.logged_in)
    })
    .await;
    assert!(logged_in);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_oauth_modal_extra_markdown_pairing(
    main_window: MainWindow,
    state_manager: StateManager,
) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_count()
            == 1
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_spotify".into());

    let modal_shown =
        wait_until(|| main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    assert!(modal_shown);

    main_window
        .global::<AppCallbacks>()
        .invoke_submit_oauth_code("pairing_md".into());

    let pairing_shown = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        oauth.get_show_oauth_modal() && oauth.get_has_extra_content()
    })
    .await;
    assert!(pairing_shown);
    assert_eq!(
        main_window
            .global::<OAuthState>()
            .get_markdown_parts()
            .row_count(),
        1
    );

    let done_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Done").collect();
    assert_eq!(done_handles.len(), 1);
    done_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let closed = wait_until(|| !main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    assert!(closed);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_oauth_modal_error_keeps_modal_open(
    main_window: MainWindow,
    state_manager: StateManager,
) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_count()
            == 1
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_spotify".into());

    let modal_shown =
        wait_until(|| main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    assert!(modal_shown);

    main_window
        .global::<AppCallbacks>()
        .invoke_submit_oauth_code("fail_oauth".into());

    let error_shown = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        oauth.get_show_oauth_modal() && !oauth.get_error_message().is_empty()
    })
    .await;
    assert!(error_shown);
    assert!(main_window.global::<OAuthState>().get_show_oauth_modal());
    assert!(
        main_window
            .global::<OAuthState>()
            .get_error_message()
            .contains("Invalid authorization code")
    );
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_oauth_modal_device_code_and_refresh(
    main_window: MainWindow,
    state_manager: StateManager,
) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_count()
            == 1
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_device".into());

    let device_shown = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        oauth.get_show_oauth_modal()
            && oauth.get_device_code() == "PAIR-1234"
            && oauth.get_oauth_url() == "https://example.com/device"
            && oauth.get_modal_title() == "Device Login"
    })
    .await;
    assert!(device_shown);

    let refresh_ready = wait_until(|| {
        ElementHandle::find_by_accessible_label(&main_window, "Refresh").count() == 1
    })
    .await;
    assert!(refresh_ready);

    let refresh_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Refresh").collect();
    assert_eq!(refresh_handles.len(), 1);
    refresh_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let closed = wait_until(|| {
        let oauth = main_window.global::<OAuthState>();
        !oauth.get_show_oauth_modal()
            && main_window
                .global::<AccountsProps>()
                .get_accounts()
                .row_data(0)
                .is_some_and(|a| a.logged_in)
    })
    .await;
    assert!(closed);
}

#[tracing::instrument(level = "debug", skip_all)]
async fn do_oauth_modal_click_inside_and_cancel(
    main_window: MainWindow,
    state_manager: StateManager,
) {
    let _ext = ExtensionFixture::new(&state_manager).await;
    AccountsHandler::fetch_and_render_accounts(main_window.as_weak(), state_manager.clone());

    let loaded = wait_until(|| {
        main_window
            .global::<AccountsProps>()
            .get_accounts()
            .row_count()
            == 1
    })
    .await;
    assert!(loaded);

    main_window
        .global::<AppCallbacks>()
        .invoke_account_login("rs.sample".into(), "sample_spotify".into());

    let modal_shown =
        wait_until(|| main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    assert!(modal_shown);

    let title_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Account Login").collect();
    if let Some(title) = title_handles.first() {
        title
            .single_click(slint::platform::PointerEventButton::Left)
            .await;
    }
    assert!(main_window.global::<OAuthState>().get_show_oauth_modal());

    let cancel_handles: Vec<ElementHandle> =
        ElementHandle::find_by_accessible_label(&main_window, "Cancel").collect();
    assert_eq!(cancel_handles.len(), 1);
    cancel_handles[0]
        .single_click(slint::platform::PointerEventButton::Left)
        .await;

    let closed = wait_until(|| !main_window.global::<OAuthState>().get_show_oauth_modal()).await;
    assert!(closed);
}

integration_test!(
    test_accounts_open_popup => do_accounts_open_popup,
    test_accounts_login_opens_oauth_modal => do_accounts_login_opens_oauth_modal,
    test_accounts_submit_oauth_code_and_modal_dismissal => do_accounts_submit_oauth_code_and_modal_dismissal,
    test_accounts_logout => do_accounts_logout,
    test_accounts_deep_link_login => do_accounts_deep_link_login,
    test_oauth_modal_plain_code_routing => do_oauth_modal_plain_code_routing,
    test_oauth_modal_extra_markdown_pairing => do_oauth_modal_extra_markdown_pairing,
    test_oauth_modal_error_keeps_modal_open => do_oauth_modal_error_keeps_modal_open,
    test_oauth_modal_device_code_and_refresh => do_oauth_modal_device_code_and_refresh,
    test_oauth_modal_click_inside_and_cancel => do_oauth_modal_click_inside_and_cancel,
);
