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

use std::fs;

use assertables::assert_is_empty;
use rstest::{fixture, rstest};
use tempdir::TempDir;
use tracing_test::traced_test;

use crate::ExtensionHandler;

struct TestHandlerContext {
    pub _temp_dir: TempDir,
    pub handler: ExtensionHandler,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn handler_context() -> TestHandlerContext {
    let temp_dir = TempDir::new("moosync_ext_lib").expect("failed to create temp dir");
    let ext_dir = temp_dir.path().join("exts");
    let tmp_dir = temp_dir.path().join("tmp");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&ext_dir).unwrap();
    fs::create_dir_all(&tmp_dir).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let handler = ExtensionHandler::new(ext_dir, tmp_dir, cache_dir);
    TestHandlerContext {
        _temp_dir: temp_dir,
        handler,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_extension_handler_init_and_get_all(handler_context: TestHandlerContext) {
    let TestHandlerContext { handler, .. } = handler_context;

    let all = handler.get_all_extensions();

    assert_is_empty!(&all);
}
