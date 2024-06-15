use std::{collections::HashMap, fmt::Debug, rc::Rc, sync::Mutex};

use leptos::{spawn_local, RwSignal, SignalGet, SignalUpdateUntracked};
use types::errors::errors::Result;
use types::ui::providers::ProviderStatus;

use crate::console_log;
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

        let spotify_provider = SpotifyProvider::new();
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
                match provider.initialize().await {
                    Ok(_) => {
                        if let Err(err) = provider.fetch_user_details().await {
                            console_log!("Error fetching user details for provider {}: {:?}", provider.key(), err)
                        }
                    },
                    Err(err) => console_log!("Error initializing provider {}: {:?}", provider.key(), err),
                }
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
            return Ok(());
        }
        Err("Provider not found".into())
    }

    pub async fn authorize(&self, key: String, code: String) -> Result<()> {
        let provider = self.providers.get(&key);
        if let Some(provider) = provider {
            let mut provider = provider.lock().unwrap();
            provider.authorize(code.clone()).await?;
            return Ok(());
        }
        Err("Provider not found".into())
    }

    pub fn get_all_statuses(&self) -> RwSignal<Vec<RwSignal<ProviderStatus>>> {
        self.statuses
    }

    pub fn get_provider_keys(&self) -> Vec<String> {
        self.providers.keys().cloned().collect()
    }

    pub fn get_provider_by_key(&self, key: String) -> Result<&Rc<Mutex<dyn GenericProvider>>> {
        let provider = self.providers.get(&key);
        if provider.is_none() {
            return Err("Provider not found".into());
        }

        Ok(provider.unwrap())
    }

    pub fn get_provider_by_id(&self, id: String) -> Option<&Rc<Mutex<dyn GenericProvider>>> {
        for (_, provider) in &self.providers{
            let provider_lock = provider.lock().unwrap();
            if provider_lock.match_id(id.clone()) {
                return Some(provider);
            }
        }

        return None
    }
}
