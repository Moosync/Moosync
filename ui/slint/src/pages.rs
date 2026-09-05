use std::{cell::RefCell, rc::Rc};

use slint::ComponentHandle;
use state_manager::StateManager;

use crate::{AppCallbacks, MainWindow, Pages, SettingsPages, main_content, settings};

pub(crate) trait PageHandler {
    #[tracing::instrument(level = "debug", skip_all)]
    fn initialize(&self) {}
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_show(&self) {}
    #[tracing::instrument(level = "debug", skip_all)]
    fn on_hide(&self) {}
}

/// Generic linear navigation history tracker.
///
/// Keeps track of the history stack and current position pointer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NavigationHistory<T: Copy + PartialEq> {
    history: Vec<T>,
    current_index: usize,
}

impl<T: Copy + PartialEq> NavigationHistory<T> {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(initial: T) -> Self {
        Self {
            history: vec![initial],
            current_index: 0,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn current(&self) -> T { self.history[self.current_index] }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn can_go_back(&self) -> bool { self.current_index > 0 }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn can_go_forward(&self) -> bool { self.current_index + 1 < self.history.len() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn push(&mut self, item: T) -> bool {
        if self.current() == item {
            return false;
        }
        self.history.truncate(self.current_index + 1);
        self.history.push(item);
        self.current_index = self.history.len() - 1;
        true
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn go_back(&mut self) -> Option<T> {
        if self.can_go_back() {
            self.current_index -= 1;
            Some(self.current())
        } else {
            None
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn go_forward(&mut self) -> Option<T> {
        if self.can_go_forward() {
            self.current_index += 1;
            Some(self.current())
        } else {
            None
        }
    }
}

/// Manages both main content and settings navigation stacks.
///
/// NOTE: Do not call `MainWindow::set_active_page` manually elsewhere in the
/// codebase. All page transitions must route through `NavigationManager` /
/// `NavigationHistory` to keep navigation history, UI buttons, and lifecycle
/// events synchronized.
pub struct NavigationManager {
    pub main_history: NavigationHistory<Pages>,
    pub settings_history: NavigationHistory<SettingsPages>,
    pub settings_open: bool,
    pub queue_open: bool,
    pub is_navigating: bool,
}

impl NavigationManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(initial_main: Pages, initial_settings: SettingsPages) -> Self {
        Self {
            main_history: NavigationHistory::new(initial_main),
            settings_history: NavigationHistory::new(initial_settings),
            settings_open: false,
            queue_open: false,
            is_navigating: false,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn active_main_page(&self) -> Pages { self.main_history.current() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn active_settings_page(&self) -> SettingsPages { self.settings_history.current() }

    #[allow(dead_code)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn navigate_main(&mut self, page: Pages, main_window: &crate::MainWindow) -> bool {
        if self.main_history.push(page) {
            self.is_navigating = true;
            main_window.set_active_page(page);
            self.is_navigating = false;
            self.sync_main_ui(main_window);
            return true;
        }
        false
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_main_page_changed(
        &mut self,
        new_page: Pages,
        main_window: &crate::MainWindow,
    ) -> bool {
        if self.is_navigating {
            return false;
        }
        let changed = self.main_history.push(new_page);
        self.sync_main_ui(main_window);
        changed
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn on_settings_page_changed(&mut self, new_page: SettingsPages) -> bool {
        if self.is_navigating {
            return false;
        }
        self.settings_history.push(new_page)
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn go_back(&mut self, main_window: &crate::MainWindow) -> bool {
        if self.settings_open {
            if self.settings_history.go_back().is_some() {
                return true;
            }
            return false;
        }

        if let Some(page) = self.main_history.go_back() {
            self.is_navigating = true;
            main_window.set_active_page(page);
            self.is_navigating = false;
            self.sync_main_ui(main_window);
            return true;
        }
        false
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn go_forward(&mut self, main_window: &crate::MainWindow) -> bool {
        if self.settings_open {
            if self.settings_history.go_forward().is_some() {
                return true;
            }
            return false;
        }

        if let Some(page) = self.main_history.go_forward() {
            self.is_navigating = true;
            main_window.set_active_page(page);
            self.is_navigating = false;
            self.sync_main_ui(main_window);
            return true;
        }
        false
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn sync_main_ui(&self, main_window: &crate::MainWindow) {
        main_window.set_can_go_back(self.main_history.can_go_back());
        main_window.set_can_go_forward(self.main_history.can_go_forward());
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn setup_page_navigation(
        main_window: &'static MainWindow,
        lifecycle: Rc<RefCell<PageLifecycleManager>>,
        nav: Rc<RefCell<NavigationManager>>,
    ) {
        let nav_back = nav.clone();
        let lifecycle_back = lifecycle.clone();
        main_window
            .global::<AppCallbacks>()
            .on_navigate_back(move || {
                if nav_back.borrow_mut().go_back(main_window) {
                    lifecycle_back
                        .borrow_mut()
                        .update_all_visibility(&nav_back.borrow());
                }
            });

        let nav_fwd = nav.clone();
        let lifecycle_fwd = lifecycle.clone();
        main_window
            .global::<AppCallbacks>()
            .on_navigate_forward(move || {
                if nav_fwd.borrow_mut().go_forward(main_window) {
                    lifecycle_fwd
                        .borrow_mut()
                        .update_all_visibility(&nav_fwd.borrow());
                }
            });

        let nav_main = nav.clone();
        let lifecycle_main = lifecycle.clone();
        main_window
            .global::<AppCallbacks>()
            .on_active_page_changed(move |new_page| {
                nav_main
                    .borrow_mut()
                    .on_main_page_changed(new_page, main_window);
                lifecycle_main
                    .borrow_mut()
                    .update_all_visibility(&nav_main.borrow());
            });

        let nav_settings = nav.clone();
        let lifecycle_settings = lifecycle.clone();
        main_window
            .global::<AppCallbacks>()
            .on_settings_active_page_changed(move |new_page| {
                nav_settings.borrow_mut().on_settings_page_changed(new_page);
                lifecycle_settings
                    .borrow_mut()
                    .update_all_visibility(&nav_settings.borrow());
            });

        let nav_settings_toggle = nav.clone();
        let lifecycle_settings_toggle = lifecycle.clone();
        main_window
            .global::<AppCallbacks>()
            .on_settings_toggled(move |open| {
                nav_settings_toggle.borrow_mut().settings_open = open;
                lifecycle_settings_toggle
                    .borrow_mut()
                    .update_all_visibility(&nav_settings_toggle.borrow());
            });

        let nav_queue_toggle = nav.clone();
        let lifecycle_queue_toggle = lifecycle.clone();
        main_window
            .global::<AppCallbacks>()
            .on_queue_toggled(move |open| {
                nav_queue_toggle.borrow_mut().queue_open = open;
                lifecycle_queue_toggle
                    .borrow_mut()
                    .update_all_visibility(&nav_queue_toggle.borrow());
            });
    }
}

pub struct PageLifecycleManager {
    main_pages: Vec<(Pages, Box<dyn PageHandler + 'static>)>,
    settings_pages: Vec<(SettingsPages, Box<dyn PageHandler + 'static>)>,
    queue_page: Box<dyn PageHandler + 'static>,
    main_visible_states: Vec<(Pages, bool)>,
    settings_visible_states: Vec<(SettingsPages, bool)>,
    queue_visible: bool,
}

impl PageLifecycleManager {
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn new(
        main_pages: Vec<(Pages, Box<dyn PageHandler + 'static>)>,
        settings_pages: Vec<(SettingsPages, Box<dyn PageHandler + 'static>)>,
        queue_page: Box<dyn PageHandler + 'static>,
    ) -> Self {
        let main_visible_states = main_pages.iter().map(|(p, _)| (*p, false)).collect();
        let settings_visible_states = settings_pages.iter().map(|(p, _)| (*p, false)).collect();
        Self {
            main_pages,
            settings_pages,
            queue_page,
            main_visible_states,
            settings_visible_states,
            queue_visible: false,
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_main_pages(
        main_window: &'static MainWindow,
        state_manager: &'static StateManager,
    ) -> Vec<(Pages, Box<dyn PageHandler + 'static>)> {
        vec![
            (
                Pages::AllSongs,
                Box::new(main_content::all_songs::AllSongsPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::Albums,
                Box::new(main_content::albums::AlbumsPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::Artists,
                Box::new(main_content::artists::ArtistsPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::Playlists,
                Box::new(main_content::playlists::PlaylistsPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::Genres,
                Box::new(main_content::genres::GenresPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::Explore,
                Box::new(main_content::explore::ExplorePageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::Search,
                Box::new(main_content::search::SearchPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::PlaylistContent,
                Box::new(
                    main_content::playlist_content::PlaylistContentPageHandler::new(
                        main_window,
                        state_manager,
                    ),
                ),
            ),
            (
                Pages::AlbumContent,
                Box::new(main_content::album_content::AlbumContentPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::ArtistContent,
                Box::new(main_content::artist_content::ArtistContentPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                Pages::GenreContent,
                Box::new(main_content::genre_content::GenreContentPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
        ]
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_settings_pages(
        main_window: &'static MainWindow,
        state_manager: &'static StateManager,
    ) -> Vec<(SettingsPages, Box<dyn PageHandler + 'static>)> {
        vec![
            (
                SettingsPages::Paths,
                Box::new(settings::paths::PathsPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                SettingsPages::System,
                Box::new(settings::system::SystemPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                SettingsPages::Extensions,
                Box::new(settings::extensions::ExtensionsPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
            (
                SettingsPages::Themes,
                Box::new(settings::themes::ThemesPageHandler::new(
                    main_window,
                    state_manager,
                )),
            ),
        ]
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn get_queue_page(
        main_window: &'static MainWindow,
        state_manager: &'static StateManager,
    ) -> Box<dyn PageHandler + 'static> {
        Box::new(main_content::queue::QueuePageHandler::new(
            main_window,
            state_manager,
        ))
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn initialize_all(&self) {
        for (_, page) in self.main_pages.iter() {
            page.initialize();
        }
        for (_, page) in self.settings_pages.iter() {
            page.initialize();
        }
        self.queue_page.initialize();
    }

    #[allow(dead_code)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn main_pages(&self) -> &[(Pages, Box<dyn PageHandler + 'static>)] { &self.main_pages }

    #[allow(dead_code)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn settings_pages(&self) -> &[(SettingsPages, Box<dyn PageHandler + 'static>)] {
        &self.settings_pages
    }

    #[allow(dead_code)]
    #[tracing::instrument(level = "debug", skip_all)]
    pub fn queue_page(&self) -> &dyn PageHandler { self.queue_page.as_ref() }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn update_all_visibility(&mut self, nav: &NavigationManager) {
        let active_main = nav.active_main_page();
        let active_settings = nav.active_settings_page();
        let settings_open = nav.settings_open;
        let queue_open = nav.queue_open;

        let main_page_keys: Vec<Pages> = self.main_pages.iter().map(|(p, _)| *p).collect();
        let settings_page_keys: Vec<SettingsPages> =
            self.settings_pages.iter().map(|(p, _)| *p).collect();

        let main_actions = self.compute_main_visibility_changes(&main_page_keys, active_main);
        for (page, is_visible) in main_actions {
            if let Some((_, handler)) = self.main_pages.iter().find(|(p, _)| *p == page) {
                if is_visible {
                    handler.on_show();
                } else {
                    handler.on_hide();
                }
            }
        }

        let settings_actions = self.compute_settings_visibility_changes(
            &settings_page_keys,
            active_settings,
            settings_open,
        );
        for (page, is_visible) in settings_actions {
            if let Some((_, handler)) = self.settings_pages.iter().find(|(p, _)| *p == page) {
                if is_visible {
                    handler.on_show();
                } else {
                    handler.on_hide();
                }
            }
        }

        if let Some(is_visible) = self.compute_queue_visibility_change(queue_open) {
            if is_visible {
                self.queue_page.on_show();
            } else {
                self.queue_page.on_hide();
            }
        }
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn compute_main_visibility_changes(
        &mut self,
        main_pages: &[Pages],
        active_main_page: Pages,
    ) -> Vec<(Pages, bool)> {
        let mut actions = Vec::new();
        for &page in main_pages {
            let was_visible = self
                .main_visible_states
                .iter()
                .find(|(p, _)| *p == page)
                .map(|(_, v)| *v)
                .unwrap_or(false);
            let is_visible = page == active_main_page;
            if is_visible != was_visible {
                if let Some(entry) = self
                    .main_visible_states
                    .iter_mut()
                    .find(|(p, _)| *p == page)
                {
                    entry.1 = is_visible;
                } else {
                    self.main_visible_states.push((page, is_visible));
                }
                actions.push((page, is_visible));
            }
        }
        actions
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn compute_settings_visibility_changes(
        &mut self,
        settings_pages: &[SettingsPages],
        active_settings_page: SettingsPages,
        settings_open: bool,
    ) -> Vec<(SettingsPages, bool)> {
        let mut actions = Vec::new();
        for &page in settings_pages {
            let was_visible = self
                .settings_visible_states
                .iter()
                .find(|(p, _)| *p == page)
                .map(|(_, v)| *v)
                .unwrap_or(false);
            let is_visible = settings_open && (was_visible || page == active_settings_page);
            if is_visible != was_visible {
                if let Some(entry) = self
                    .settings_visible_states
                    .iter_mut()
                    .find(|(p, _)| *p == page)
                {
                    entry.1 = is_visible;
                } else {
                    self.settings_visible_states.push((page, is_visible));
                }
                actions.push((page, is_visible));
            }
        }
        actions
    }

    #[tracing::instrument(level = "debug", skip_all)]
    pub fn compute_queue_visibility_change(&mut self, queue_open: bool) -> Option<bool> {
        if self.queue_visible != queue_open {
            self.queue_visible = queue_open;
            Some(queue_open)
        } else {
            None
        }
    }
}
