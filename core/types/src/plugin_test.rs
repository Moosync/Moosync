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

use std::sync::Arc;

use assertables::{assert_none, assert_some_eq_x};
use rstest::{fixture, rstest};
use tokio::sync::RwLock;
use tracing_test::traced_test;

use crate::plugin::{CallContext, Plugin, PluginContext, PluginRegistry};

struct DummyPlugin {
    value: u32,
}

impl Plugin for DummyPlugin {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(_: &PluginContext) -> Arc<RwLock<Self>> {
        Arc::new(RwLock::new(DummyPlugin { value: 42 }))
    }
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn dummy_plugin() -> Arc<RwLock<DummyPlugin>> { Arc::new(RwLock::new(DummyPlugin { value: 100 })) }

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_plugin_registry_register_and_get(dummy_plugin: Arc<RwLock<DummyPlugin>>) {
    let mut registry = PluginRegistry::new();

    registry.register(dummy_plugin);

    let retrieved = registry.get::<DummyPlugin>();
    let guard = retrieved.read().await;
    assert_eq!(guard.value, 100);
}

#[test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_call_context_insert_get_remove() {
    let mut context = CallContext::default();

    context.insert(12345u64);
    assert_eq!(*context.get_mut::<u64>().unwrap(), 12345u64);

    *context.get_mut::<u64>().unwrap() = 54321u64;
    assert_some_eq_x!(context.remove::<u64>(), 54321u64);
    assert_none!(context.get_mut::<u64>());
}
