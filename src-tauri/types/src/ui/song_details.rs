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

use std::sync::Arc;

#[derive(Default, Clone)]
pub struct SongDetailIcons {
    pub play: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
    pub add_to_queue: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
    pub random: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
    pub add_to_library: Option<Arc<Box<dyn Fn() + Send + Sync>>>,
}

#[derive(Default, Clone)]
pub struct DefaultDetails {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub icon: Option<String>,
}
