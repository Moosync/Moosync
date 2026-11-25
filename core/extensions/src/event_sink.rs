use std::sync::{Arc, Weak};

use extensions_proto::moosync::types::ExtensionCommand;

use crate::{errors::ExtensionError, extension::AsyncContext};

#[async_trait::async_trait]
pub trait ExtensionEventSink: Send + Sync {
    async fn dispatch(&self, command: ExtensionCommand) -> Result<(), ExtensionError>;
}

struct ContextEventSink {
    context: Weak<AsyncContext>,
}

#[tracing::instrument(level = "debug", skip_all)]
pub(crate) fn context_event_sink(context: Weak<AsyncContext>) -> Arc<dyn ExtensionEventSink> {
    Arc::new(ContextEventSink { context })
}

#[async_trait::async_trait]
impl ExtensionEventSink for ContextEventSink {
    #[tracing::instrument(level = "debug", skip_all)]
    async fn dispatch(&self, command: ExtensionCommand) -> Result<(), ExtensionError> {
        let Some(context) = self.context.upgrade() else {
            return Err(ExtensionError::NoContext);
        };
        let extension_context = context.wait().await?;
        let _ = extension_context.execute_command(command).await?;
        Ok(())
    }
}
