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

use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;
use types::plugin::{Plugin, PluginContext};

use crate::LyricsFetcher;

struct PluginSmokeContext {
    pub _temp_dir: TempDir,
    pub context: PluginContext,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn smoke_context() -> PluginSmokeContext {
    let temp_dir = TempDir::new("moosync_lyrics_plugin_smoke").expect("failed to create temp dir");
    let test_dir = temp_dir.path().to_path_buf();
    let context = PluginContext {
        data_dir: test_dir.clone(),
        cache_dir: test_dir.clone(),
        tmp_dir: test_dir.clone(),
        #[cfg(target_os = "android")]
        android_context: types::android::AndroidJNIContext::default(),
    };
    PluginSmokeContext {
        _temp_dir: temp_dir,
        context,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_lyrics_plugin_init(smoke_context: PluginSmokeContext) {
    let PluginSmokeContext { context, .. } = smoke_context;

    let plugin = LyricsFetcher::init(&context);
    let _guard = plugin.blocking_read();
}
