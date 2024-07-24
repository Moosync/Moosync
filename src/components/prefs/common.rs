use leptos::{create_effect, create_rw_signal, RwSignal, SignalGet, SignalSet, SignalUpdate};
use serde::{de::DeserializeOwned, Serialize};

use crate::utils::prefs::{load_selective, save_selective};

pub fn setup_value_listeners_vec<T>(key: String) -> (RwSignal<Vec<T>>, RwSignal<Vec<T>>)
where
    T: Clone + DeserializeOwned + Serialize + 'static,
{
    let should_write = create_rw_signal(false);
    let pref_value = create_rw_signal::<Vec<T>>(vec![]);
    let pref_key = key;
    load_selective(pref_key.clone(), pref_value.write_only());

    let updated_value = create_rw_signal(vec![]);

    create_effect(move |_| {
        let new_paths = updated_value.get();
        if !new_paths.is_empty() {
            pref_value.update(|paths| paths.extend(new_paths.iter().cloned()));
        }
    });

    create_effect(move |_| {
        let value = pref_value.get();
        if !should_write.get() {
            should_write.set(true);
            return;
        }
        save_selective(pref_key.clone(), value);
    });

    (pref_value, updated_value)
}

pub fn setup_value_listeners<T>(key: String) -> RwSignal<T>
where
    T: Clone + Default + DeserializeOwned + Serialize + 'static,
{
    let should_write = create_rw_signal(false);
    let pref_value = create_rw_signal::<T>(Default::default());
    let pref_key = key;
    load_selective(pref_key.clone(), pref_value.write_only());

    create_effect(move |_| {
        let value = pref_value.get();
        if !should_write.get() {
            should_write.set(true);
            return;
        }
        save_selective(pref_key.clone(), value);
    });

    pref_value
}
