use std::sync::Arc;

use assertables::{assert_none, assert_some_eq_x};
use rstest::{fixture, rstest};
use tracing_test::traced_test;

use crate::plugin::{CallContext, Plugin, PluginContext, PluginRegistry};

struct DummyPlugin {
    value: u32,
}

impl Plugin for DummyPlugin {
    #[tracing::instrument(level = "debug", skip_all)]
    fn init(_: &PluginContext) -> Arc<Self> { Arc::new(DummyPlugin { value: 42 }) }
}

#[fixture]
#[tracing::instrument(level = "debug", skip_all)]
fn dummy_plugin() -> Arc<DummyPlugin> { Arc::new(DummyPlugin { value: 100 }) }

#[rstest]
#[tokio::test]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_plugin_registry_register_and_get(dummy_plugin: Arc<DummyPlugin>) {
    let mut registry = PluginRegistry::new();

    registry.register(dummy_plugin);

    let retrieved = registry.get::<DummyPlugin>();
    assert_eq!(retrieved.value, 100);
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
