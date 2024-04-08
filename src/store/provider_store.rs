use std::{collections::HashMap, fmt::Debug, rc::Rc, sync::Mutex};

use leptos::{
    spawn_local, RwSignal, SignalGet, SignalUpdate, SignalUpdateUntracked, SignalWithUntracked,
};
use types::ui::providers::ProviderStatus;
use types::errors::errors::Result;

use crate::providers::{generic::GenericProvider, spotify::SpotifyProvider};

#[derive(Debug, Default)]
pub struct ProviderStore {
    providers: HashMap<String, Rc<Mutex<dyn GenericProvider>>>,
    statuses: RwSignal<Vec<RwSignal<ProviderStatus>>>,
}

impl ProviderStore {
    pub fn new() -> Self {
        let mut store = Self {
            ..Default::default()
        };

        let spotify_provider =SpotifyProvider::new();
        store.providers.insert(
            spotify_provider.key().to_string(),
            Rc::new(Mutex::new(spotify_provider)),
        );

        store.initialize_all_providers();
        store
    }

    pub fn initialize_provider(&self, key: &str) {
        let provider = self.providers.get(key);
        if let Some(provider_rc) = provider {
            let provider = provider_rc.lock().unwrap();
            self.statuses
                .update_untracked(|s| s.push(provider.get_status()));

            drop(provider);

            let cloned = provider_rc.clone();
            spawn_local(async move {
                let mut provider = cloned.lock().unwrap();
                provider.initialize().await.unwrap();
                provider.fetch_user_details().await.unwrap();
            });
        }
    }

    pub fn initialize_all_providers(&self) {
        for key in self.providers.keys() {
            self.initialize_provider(key);    
        }
    }

    pub async fn login(&self, key: String) -> Result<()> {
        let provider = self.providers.get(&key);
        if let Some(provider) = provider {
            let mut provider = provider.lock().unwrap();
            provider.login().await?;
            return Ok(())
        }
        Err("Provider not found".into())
    }

    pub async fn authorize(&self, key: String, code: String) -> Result<()> {
        let provider = self.providers.get(&key);
        if let Some(provider) = provider {
            let mut provider = provider.lock().unwrap();
            provider.authorize(code.clone()).await?;
            return Ok(())
        }
        Err("Provider not found".into())
    }

    pub fn get_all_statuses(&self) -> RwSignal<Vec<RwSignal<ProviderStatus>>> {
        self.statuses
    }
}
