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

use leptos::task::spawn_local;
use types::ui::extensions::{ExtensionExtraEvent, ExtensionExtraEventArgs};

use crate::utils::invoke::send_extra_event;

#[tracing::instrument(level = "trace", skip(args))]
pub fn send_extension_event(args: ExtensionExtraEvent) {
    spawn_local(async move {
        let res = send_extra_event(ExtensionExtraEventArgs {
            data: args,
            package_name: "".into(),
        })
        .await;

        if let Err(e) = res {
            tracing::error!("Failed to send extension event: {:?}", e);
        }
    });
}
