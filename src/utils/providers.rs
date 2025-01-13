#[macro_export]
macro_rules! dyn_provider_songs {
    ($selected_providers:ident, $entity:ident, $songs:ident, $fetch_fn: ident) => {{
        use leptos::prelude::Get;
        use std::sync::Arc;
        let provider_songs: RwSignal<HashMap<String, RwSignal<Vec<Song>>>> =
            create_rw_signal(HashMap::new());
        let next_page_tokens: RwSignal<
            HashMap<String, Arc<futures::lock::Mutex<types::providers::generic::Pagination>>>,
        > = create_rw_signal(HashMap::new());

        let fetch_selected_providers = Arc::new(Box::new(move || {
            let selected_providers = $selected_providers.get();

            let entity = $entity.get();
            if entity.is_none() {
                return;
            }
            let entity = entity.unwrap();

            tracing::debug!(
                "Providers selected: {:?}, entity: {:?}",
                selected_providers,
                entity
            );
            for provider in selected_providers {
                let entity = entity.clone();

                spawn_local(async move {
                    let provider_songs_inner = provider_songs.get_untracked();

                    if !provider_songs_inner.contains_key(&provider.clone()) {
                        let key = provider.clone();
                        provider_songs.update(|p| {
                            p.insert(key, create_rw_signal(vec![]));
                        });
                    }

                    let binding = provider_songs.get_untracked();
                    let binding = binding.get(&provider.clone());

                    tracing::debug!("fetching infinite");
                    let res = fetch_infinite!(
                        provider,
                        $fetch_fn,
                        *binding.unwrap(),
                        next_page_tokens,
                        entity.clone()
                    );

                    if let Err(e) = res {
                        tracing::error!("Error fetching content: {:?}", e);
                    }
                });
            }
        }));

        let fetch = fetch_selected_providers.clone();
        create_effect(move |_| {
            fetch.as_ref()();
        });

        let get_collective_songs = create_memo(move |_| {
            let mut ret = vec![];
            ret.extend($songs.get());

            let $selected_providers = $selected_providers.get();

            for (provider, songs) in provider_songs.get() {
                if $selected_providers.contains(&provider) {
                    let songs = songs.get();
                    ret.extend(songs);
                }
            }

            ret
        });

        (
            provider_songs,
            get_collective_songs,
            fetch_selected_providers,
        )
    }};
}
