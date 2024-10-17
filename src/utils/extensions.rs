use leptos::spawn_local;
use types::extensions::{ExtensionExtraEvent, ExtensionExtraEventArgs};

use crate::utils::common::invoke;

#[tracing::instrument(level = "trace", skip(args))]
pub fn send_extension_event(args: ExtensionExtraEvent) {
    spawn_local(async move {
        #[derive(serde::Serialize)]
        struct ExtraEventArgs {
            args: ExtensionExtraEventArgs,
        }
        let res = invoke(
            "send_extra_event",
            serde_wasm_bindgen::to_value(&ExtraEventArgs {
                args: ExtensionExtraEventArgs {
                    data: args,
                    package_name: "".into(),
                },
            })
            .unwrap(),
        )
        .await;

        if let Err(e) = res {
            tracing::error!("Failed to send extension event: {:?}", e);
        }
    });
}
