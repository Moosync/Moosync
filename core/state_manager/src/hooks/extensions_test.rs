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

use crate::{
    StateManager,
    hooks::{Hook, extensions::ExtensionsHook},
};

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_hook_on_startup() {
    let sm = StateManager::new(
        #[cfg(target_os = "android")]
        types::android::AndroidJNIContext::default(),
    )
    .unwrap();

    let hook = ExtensionsHook::new();
    let res = hook.on_startup(&sm).await;
    assert!(res.is_ok());

    let pref = sm.get_preference_config().await;
    let _ = pref.save(
        preferences::keys::ExtensionRegistries,
        vec!["https://example.com/custom_manifest.json".to_string()],
    );

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    let ext = sm.get_extension_handler().await;
    let registries = ext.get_registries();
    assert!(registries.contains("https://example.com/custom_manifest.json"));
}

#[tokio::test]
#[tracing::instrument(level = "debug", skip_all)]
async fn test_extensions_hook_triggers_update_on_registry_change() {
    let sm = StateManager::new(
        #[cfg(target_os = "android")]
        types::android::AndroidJNIContext::default(),
    )
    .unwrap();

    let hook = ExtensionsHook::new();
    let _ = hook.on_startup(&sm).await;

    let updated_flag = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let ext = sm.get_extension_handler().await;
    let flag_clone = updated_flag.clone();
    let _cancel = ext.on_extensions_updated(move |_| {
        flag_clone.store(true, std::sync::atomic::Ordering::SeqCst);
    });

    let pref = sm.get_preference_config().await;
    let _ = pref.save(
        preferences::keys::ExtensionRegistries,
        vec!["https://example.com/new_registry.json".to_string()],
    );

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;

    assert!(updated_flag.load(std::sync::atomic::Ordering::SeqCst));
}
