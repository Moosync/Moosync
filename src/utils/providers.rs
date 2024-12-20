#[macro_export]
macro_rules! dyn_provider_songs {
    ($selected_providers:ident, $entity:ident, $provider_store:ident, $songs:ident, $fetch_fn: ident) => {{
        let provider_songs: RwSignal<HashMap<String, RwSignal<Vec<Song>>>> =
            create_rw_signal(HashMap::new());

        create_effect(move |_| {
            let $selected_providers = $selected_providers.get();
            tracing::debug!("Providers selected: {:?}", $selected_providers);

            let $entity = $entity.get();
            let $entity = $entity.first();
            if $entity.is_none() {
                return;
            }
            let $entity = $entity.unwrap();
            for provider in $selected_providers {
                let $provider_store = $provider_store.clone();
                let $entity = $entity.clone();

                spawn_local(async move {
                    let provider_songs_inner = provider_songs.get();

                    if !provider_songs_inner.contains_key(&provider.clone()) {
                        let key = provider.clone();
                        provider_songs.update(|p| {
                            p.insert(key, create_rw_signal(vec![]));
                        });
                        let binding = provider_songs.get();
                        let binding = binding.get(&provider.clone());

                        fetch_infinite!(
                            $provider_store,
                            provider,
                            $fetch_fn,
                            *binding.unwrap(),
                            $entity.clone()
                        );
                    };
                });
            }
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

        (provider_songs, get_collective_songs)
    }};
}
