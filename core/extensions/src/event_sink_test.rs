use std::sync::Weak;

use extensions_proto::moosync::types::ExtensionCommand;

use crate::{errors::ExtensionError, event_sink::context_event_sink, extension::AsyncContext};

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_context_event_sink_returns_no_context_when_dropped() {
    let weak: Weak<AsyncContext> = Weak::new();
    let sink = context_event_sink(weak);

    let result = sink.dispatch(ExtensionCommand::default()).await;

    assert!(matches!(result, Err(ExtensionError::NoContext)));
}
