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

use crate::ext_runner::ExtensionHandlerInner;

struct TestRunnerContext {
    pub _temp_dir: TempDir,
    pub runner: ExtensionHandlerInner,
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn runner_context() -> TestRunnerContext {
    let temp_dir = TempDir::new("moosync_ext_runner").expect("failed to create temp dir");
    let ext_dir = temp_dir.path().join("exts");
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&ext_dir).unwrap();
    fs::create_dir_all(&cache_dir).unwrap();

    let runner = ExtensionHandlerInner::new(ext_dir, cache_dir);
    TestRunnerContext {
        _temp_dir: temp_dir,
        runner,
    }
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_ext_runner_get_installed_extensions_empty(runner_context: TestRunnerContext) {
    let TestRunnerContext { runner, .. } = runner_context;

    let list = runner.get_installed_extensions();

    assert_is_empty!(&list);
}
