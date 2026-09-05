use rstest::rstest;
use tracing_test::traced_test;

use crate::{
    MainWindow, Pages, SettingsPages,
    pages::{NavigationHistory, NavigationManager, PageLifecycleManager},
    test_utils::main_window,
};

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_initial_state() {
    let history = NavigationHistory::new(Pages::AllSongs);

    assert_eq!(history.current(), Pages::AllSongs);
    assert!(!history.can_go_back());
    assert!(!history.can_go_forward());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_push_new() {
    let mut history = NavigationHistory::new(Pages::AllSongs);

    let pushed = history.push(Pages::Albums);

    assert!(pushed);
    assert_eq!(history.current(), Pages::Albums);
    assert!(history.can_go_back());
    assert!(!history.can_go_forward());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_push_duplicate_ignored() {
    let mut history = NavigationHistory::new(Pages::AllSongs);

    let pushed = history.push(Pages::AllSongs);

    assert!(!pushed);
    assert_eq!(history.current(), Pages::AllSongs);
    assert!(!history.can_go_back());
    assert!(!history.can_go_forward());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_go_back() {
    let mut history = NavigationHistory::new(Pages::AllSongs);
    history.push(Pages::Albums);
    history.push(Pages::Artists);

    let back_page = history.go_back();

    assert_eq!(back_page, Some(Pages::Albums));
    assert_eq!(history.current(), Pages::Albums);
    assert!(history.can_go_back());
    assert!(history.can_go_forward());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_go_back_at_start() {
    let mut history = NavigationHistory::new(Pages::AllSongs);

    let back_page = history.go_back();

    assert_eq!(back_page, None);
    assert_eq!(history.current(), Pages::AllSongs);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_go_forward() {
    let mut history = NavigationHistory::new(Pages::AllSongs);
    history.push(Pages::Albums);
    history.go_back();

    let forward_page = history.go_forward();

    assert_eq!(forward_page, Some(Pages::Albums));
    assert_eq!(history.current(), Pages::Albums);
    assert!(history.can_go_back());
    assert!(!history.can_go_forward());
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_go_forward_at_end() {
    let mut history = NavigationHistory::new(Pages::AllSongs);
    history.push(Pages::Albums);

    let forward_page = history.go_forward();

    assert_eq!(forward_page, None);
    assert_eq!(history.current(), Pages::Albums);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_branching_truncation() {
    let mut history = NavigationHistory::new(Pages::AllSongs);
    history.push(Pages::Albums);
    history.push(Pages::Artists);
    history.go_back();
    let pushed = history.push(Pages::Playlists);

    assert!(pushed);
    assert_eq!(history.current(), Pages::Playlists);
    assert!(history.can_go_back());
    assert!(!history.can_go_forward());
    assert_eq!(history.go_back(), Some(Pages::Albums));
    assert_eq!(history.go_back(), Some(Pages::AllSongs));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_history_settings_pages() {
    let mut history = NavigationHistory::new(SettingsPages::Extensions);
    history.push(SettingsPages::Paths);
    history.push(SettingsPages::Themes);

    assert_eq!(history.current(), SettingsPages::Themes);
    assert_eq!(history.go_back(), Some(SettingsPages::Paths));
    assert_eq!(history.go_back(), Some(SettingsPages::Extensions));
    assert_eq!(history.go_forward(), Some(SettingsPages::Paths));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_manager_initial_state(main_window: MainWindow) {
    let nav = NavigationManager::new(Pages::AllSongs, SettingsPages::Extensions);

    nav.sync_main_ui(&main_window);

    assert_eq!(nav.active_main_page(), Pages::AllSongs);
    assert_eq!(nav.active_settings_page(), SettingsPages::Extensions);
    assert!(!main_window.get_can_go_back());
    assert!(!main_window.get_can_go_forward());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_manager_navigate_main(main_window: MainWindow) {
    let mut nav = NavigationManager::new(Pages::AllSongs, SettingsPages::Extensions);

    let navigated = nav.navigate_main(Pages::Albums, &main_window);

    assert!(navigated);
    assert_eq!(nav.active_main_page(), Pages::Albums);
    assert_eq!(main_window.get_active_page(), Pages::Albums);
    assert!(main_window.get_can_go_back());
    assert!(!main_window.get_can_go_forward());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_manager_navigate_settings(_main_window: MainWindow) {
    let mut nav = NavigationManager::new(Pages::AllSongs, SettingsPages::Extensions);

    let changed = nav.on_settings_page_changed(SettingsPages::System);

    assert!(changed);
    assert_eq!(nav.active_settings_page(), SettingsPages::System);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_manager_go_back_main(main_window: MainWindow) {
    let mut nav = NavigationManager::new(Pages::AllSongs, SettingsPages::Extensions);
    nav.navigate_main(Pages::Albums, &main_window);

    let went_back = nav.go_back(&main_window);

    assert!(went_back);
    assert_eq!(nav.active_main_page(), Pages::AllSongs);
    assert_eq!(main_window.get_active_page(), Pages::AllSongs);
    assert!(!main_window.get_can_go_back());
    assert!(main_window.get_can_go_forward());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_manager_go_forward_main(main_window: MainWindow) {
    let mut nav = NavigationManager::new(Pages::AllSongs, SettingsPages::Extensions);
    nav.navigate_main(Pages::Albums, &main_window);
    nav.go_back(&main_window);

    let went_forward = nav.go_forward(&main_window);

    assert!(went_forward);
    assert_eq!(nav.active_main_page(), Pages::Albums);
    assert_eq!(main_window.get_active_page(), Pages::Albums);
    assert!(main_window.get_can_go_back());
    assert!(!main_window.get_can_go_forward());
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_manager_go_back_settings_when_open(main_window: MainWindow) {
    let mut nav = NavigationManager::new(Pages::AllSongs, SettingsPages::Extensions);
    nav.settings_open = true;
    nav.on_settings_page_changed(SettingsPages::Paths);

    let went_back = nav.go_back(&main_window);

    assert!(went_back);
    assert_eq!(nav.active_settings_page(), SettingsPages::Extensions);
    assert_eq!(nav.active_main_page(), Pages::AllSongs);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_navigation_manager_go_forward_settings_when_open(main_window: MainWindow) {
    let mut nav = NavigationManager::new(Pages::AllSongs, SettingsPages::Extensions);
    nav.settings_open = true;
    nav.on_settings_page_changed(SettingsPages::Paths);
    nav.go_back(&main_window);

    let went_forward = nav.go_forward(&main_window);

    assert!(went_forward);
    assert_eq!(nav.active_settings_page(), SettingsPages::Paths);
    assert_eq!(nav.active_main_page(), Pages::AllSongs);
}

use std::sync::atomic::{AtomicUsize, Ordering};

struct DummyPage {
    show_count: AtomicUsize,
    hide_count: AtomicUsize,
    init_count: AtomicUsize,
}

impl DummyPage {
    #[tracing::instrument(level = "debug", skip_all)]
    fn new() -> Self {
        Self {
            show_count: AtomicUsize::new(0),
            hide_count: AtomicUsize::new(0),
            init_count: AtomicUsize::new(0),
        }
    }
}

impl crate::pages::PageHandler for DummyPage {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) { self.init_count.fetch_add(1, Ordering::SeqCst); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) { self.show_count.fetch_add(1, Ordering::SeqCst); }

    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) { self.hide_count.fetch_add(1, Ordering::SeqCst); }
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lifecycle_initial_visibility() {
    let main_pages = [Pages::AllSongs, Pages::Albums, Pages::Artists];
    let mut lifecycle = PageLifecycleManager::new(
        vec![
            (Pages::AllSongs, Box::new(DummyPage::new())),
            (Pages::Albums, Box::new(DummyPage::new())),
            (Pages::Artists, Box::new(DummyPage::new())),
        ],
        vec![
            (SettingsPages::Paths, Box::new(DummyPage::new())),
            (SettingsPages::System, Box::new(DummyPage::new())),
        ],
        Box::new(DummyPage::new()),
    );

    let actions = lifecycle.compute_main_visibility_changes(&main_pages, Pages::AllSongs);

    assert_eq!(actions, vec![(Pages::AllSongs, true)]);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lifecycle_main_page_switch() {
    let main_pages = [Pages::AllSongs, Pages::Albums];
    let mut lifecycle = PageLifecycleManager::new(
        vec![
            (Pages::AllSongs, Box::new(DummyPage::new())),
            (Pages::Albums, Box::new(DummyPage::new())),
        ],
        vec![(SettingsPages::Paths, Box::new(DummyPage::new()))],
        Box::new(DummyPage::new()),
    );
    lifecycle.compute_main_visibility_changes(&main_pages, Pages::AllSongs);

    let actions = lifecycle.compute_main_visibility_changes(&main_pages, Pages::Albums);

    assert_eq!(
        actions,
        vec![(Pages::AllSongs, false), (Pages::Albums, true)]
    );
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lifecycle_settings_open_close() {
    let settings_pages = [SettingsPages::Paths, SettingsPages::System];
    let mut lifecycle = PageLifecycleManager::new(
        vec![(Pages::AllSongs, Box::new(DummyPage::new()))],
        vec![
            (SettingsPages::Paths, Box::new(DummyPage::new())),
            (SettingsPages::System, Box::new(DummyPage::new())),
        ],
        Box::new(DummyPage::new()),
    );

    let open_actions =
        lifecycle.compute_settings_visibility_changes(&settings_pages, SettingsPages::Paths, true);
    let close_actions =
        lifecycle.compute_settings_visibility_changes(&settings_pages, SettingsPages::Paths, false);

    assert_eq!(open_actions, vec![(SettingsPages::Paths, true)]);
    assert_eq!(close_actions, vec![(SettingsPages::Paths, false)]);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lifecycle_queue_open_close() {
    let mut lifecycle = PageLifecycleManager::new(
        vec![(Pages::AllSongs, Box::new(DummyPage::new()))],
        vec![(SettingsPages::Paths, Box::new(DummyPage::new()))],
        Box::new(DummyPage::new()),
    );

    let open_action = lifecycle.compute_queue_visibility_change(true);
    let no_change_action = lifecycle.compute_queue_visibility_change(true);
    let close_action = lifecycle.compute_queue_visibility_change(false);

    assert_eq!(open_action, Some(true));
    assert_eq!(no_change_action, None);
    assert_eq!(close_action, Some(false));
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lifecycle_initialize_and_update_all_visibility() {
    let mut lifecycle = PageLifecycleManager::new(
        vec![
            (Pages::AllSongs, Box::new(DummyPage::new())),
            (Pages::Albums, Box::new(DummyPage::new())),
        ],
        vec![(SettingsPages::Paths, Box::new(DummyPage::new()))],
        Box::new(DummyPage::new()),
    );
    let nav = NavigationManager::new(Pages::AllSongs, SettingsPages::Paths);

    lifecycle.initialize_all();
    lifecycle.update_all_visibility(&nav);

    assert_eq!(lifecycle.main_pages().len(), 2);
    assert_eq!(lifecycle.settings_pages().len(), 1);
}
