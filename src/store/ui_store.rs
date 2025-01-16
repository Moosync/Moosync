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

// Moosync
// Copyright (C) 2025 Moosync
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
// along with this program. If not, see <http://www.gnu.org/licenses/>.

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum SongSortByColumns {
    Album,
    Artist,
    Date,
    Genre,
    PlayCount,
    Title,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum PlaylistSortByColumns {
    Title,
    Provider,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct SongSortBy {
    pub asc: bool,
    pub sort_by: SongSortByColumns,
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct PlaylistSortBy {
    pub asc: bool,
    pub sort_by: PlaylistSortByColumns,
}

pub struct UiStore {
    song_sort_by: SongSortBy,
    playlist_sort_by: PlaylistSortBy,
    show_queue: bool,
    sidebar_open: bool,
    is_mobile: bool,
    is_mobile_player: bool,
}

impl UiStore {
    #[tracing::instrument(level = "trace", skip())]
    pub fn new() -> Self {
        Self {
            song_sort_by: SongSortBy {
                asc: true,
                sort_by: SongSortByColumns::Album,
            },
            playlist_sort_by: PlaylistSortBy {
                asc: true,
                sort_by: PlaylistSortByColumns::Provider,
            },
            show_queue: false,
            sidebar_open: true,
            is_mobile: false,
            is_mobile_player: false,
        }
    }

    #[tracing::instrument(level = "trace", skip(self, sort_by))]
    pub fn set_song_sort_by(&mut self, sort_by: SongSortBy) {
        self.song_sort_by = sort_by;
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_song_sort_by(&self) -> SongSortBy {
        self.song_sort_by
    }

    #[tracing::instrument(level = "trace", skip(self, sort_by))]
    pub fn set_playlist_sort_by(&mut self, sort_by: PlaylistSortBy) {
        self.playlist_sort_by = sort_by;
    }

    #[tracing::instrument(level = "trace", skip(self))]
    pub fn get_playlist_sort_by(&self) -> PlaylistSortBy {
        self.playlist_sort_by
    }
    pub fn show_queue(&mut self, show: bool) {
        self.show_queue = show;
    }

    pub fn get_show_queue(&self) -> bool {
        self.show_queue
    }

    pub fn toggle_show_queue(&mut self) {
        self.show_queue = !self.show_queue;
    }

    pub fn get_sidebar_open(&self) -> bool {
        self.sidebar_open
    }

    pub fn set_sidebar_open(&mut self, val: bool) {
        self.sidebar_open = val;
    }

    pub fn set_is_mobile(&mut self, val: bool) {
        self.is_mobile = val;
    }

    pub fn get_is_mobile(&self) -> bool {
        self.is_mobile
    }

    pub fn set_is_mobile_player(&mut self, val: bool) {
        self.is_mobile_player = val;
    }

    pub fn get_is_mobile_player(&self) -> bool {
        self.is_mobile_player
    }
}
