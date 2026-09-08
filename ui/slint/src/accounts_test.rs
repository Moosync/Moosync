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
