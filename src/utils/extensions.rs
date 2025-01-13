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
