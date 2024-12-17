#[macro_export]
macro_rules! generate_command {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        #[tracing::instrument(level = "trace", skip(db))]
        #[tauri::command(async)]
        pub fn $method_name(db: State<$state>, $($v: $t),*) -> types::errors::Result<$ret> {
            tracing::debug!("calling {}", stringify!($method_name));
            let ret = db.$method_name($($v,)*);
            if let Ok(ret) = &ret {
                tracing::trace!("Got result {:?}", ret);
            } else {
                tracing::error!("Error getting result {:?}", ret);
            }
            ret
        }
    };
}

#[macro_export]
macro_rules! generate_command_cached {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        // #[flame]
        #[tracing::instrument(level = "trace", skip(db, cache))]
        #[tauri::command(async)]
        pub async fn $method_name(db: State<'_, $state>, cache: State<'_, CacheHolder>, $($v: $t),*) -> types::errors::Result<$ret> {
            let mut cache_string = String::new();
            cache_string.push_str(stringify!($method_name));
            $(
                {
                    cache_string.push_str(format!("_{:?}", $v).as_str());
                }
            )*

            tracing::debug!("calling cached {}: {}", stringify!($method_name), cache_string);
            let cached = cache.get(cache_string.as_str());
            if cached.is_ok() {
                return cached;
            }

            let res = db.$method_name($($v,)*);
            match &res {
                Ok(res) => {
                    tracing::trace!("Got result {:?}", res);
                    let cache_res = cache.set(cache_string.as_str(), res, 7200);
                    if let Ok(cache_res) = cache_res {
                        tracing::trace!("Updated result in cache");
                    } else {
                        tracing::error!("Error updating cache {:?}", cache_res.unwrap_err());
                    }
                },
                Err(e) => {
                    tracing::error!("Error getting result {:?}", e);
                }
            }
            res
        }
    };
}

#[macro_export]
macro_rules! generate_command_async {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        // #[flame]
        #[tracing::instrument(level = "trace", skip(db))]
        #[tauri::command(async)]
        pub async fn $method_name(db: State<'_, $state>, $($v: $t),*) -> types::errors::Result<$ret> {
            tracing::debug!("calling async {}", stringify!($method_name));
            let ret = db.$method_name($($v,)*).await;
            if let Ok(ret) = &ret {
                tracing::trace!("Got result {:?}", ret);
            } else {
                tracing::error!("Error getting result {:?}", ret);
            }
            ret
        }
    };
}

#[macro_export]
macro_rules! generate_command_async_cached {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        // #[flame]
        #[tracing::instrument(level = "trace", skip(db, cache))]
        #[tauri::command(async)]
        pub async fn $method_name(db: State<'_, $state>, cache: State<'_, CacheHolder>, $($v: $t),*) -> types::errors::Result<$ret> {
            let mut cache_string = String::new();
            cache_string.push_str(stringify!($method_name));
            $(
                {
                    cache_string.push_str(format!("_{:?}", $v).as_str());
                }
            )*

            tracing::debug!("calling cached async {}: {}", stringify!($method_name), cache_string);
            let cached = cache.get(cache_string.as_str());

            if cached.is_ok() {
                tracing::debug!("got cached data");
                return cached;
            }

            let res = db.$method_name($($v,)*).await;
            match &res {
                Ok(res) => {
                    tracing::trace!("Got result {:?}", res);
                    let cache_res = cache.set(cache_string.as_str(), res, 7200);
                    if let Ok(cache_res) = cache_res {
                        tracing::trace!("Updated result in cache");
                    } else {
                        tracing::error!("Error updating cache {:?}", cache_res.unwrap_err());
                    }
                },
                Err(e) => {
                    tracing::error!("Error getting result {:?}", e);
                }
            }
            res
        }
    };
}
