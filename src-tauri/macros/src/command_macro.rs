#[macro_export]
macro_rules! generate_command {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        // #[flame]
        #[tauri::command(async)]
        pub fn $method_name(db: State<$state>, $($v: $t),*) -> types::errors::errors::Result<$ret> {
            println!("calling {}", stringify!($method_name));
            db.$method_name($($v,)*)
        }
    };
}

#[macro_export]
macro_rules! generate_command_cached {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        // #[flame]
        #[tauri::command(async)]
        pub async fn $method_name(db: State<'_, $state>, cache: State<'_, CacheHolder>, $($v: $t),*) -> types::errors::errors::Result<$ret> {
            println!("calling cached {}", stringify!($method_name));
            let mut cache_string = String::new();
            cache_string.push_str(stringify!($method_name));
            $(
                {
                    cache_string.push_str(format!("_{:?}", $v).as_str());
                }
            )*

            let cached = cache.get(cache_string.as_str());
            if cached.is_ok() {
                return cached;
            }

            let res = db.$method_name($($v,)*)?;
            let _ = cache.set(cache_string.as_str(), &res, 7200);
            Ok(res)
        }
    };
}

#[macro_export]
macro_rules! generate_command_async {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        // #[flame]
        #[tauri::command(async)]
        pub async fn $method_name(db: State<'_, $state>, $($v: $t),*) -> types::errors::errors::Result<$ret> {
            println!("calling async {}", stringify!($method_name));
            db.$method_name($($v,)*).await
        }
    };
}

#[macro_export]
macro_rules! generate_command_async_cached {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        // #[flame]
        #[tauri::command(async)]
        pub async fn $method_name(db: State<'_, $state>, cache: State<'_, CacheHolder>, $($v: $t),*) -> types::errors::errors::Result<$ret> {
            println!("calling cached async {}", stringify!($method_name));
            let mut cache_string = String::new();
            cache_string.push_str(stringify!($method_name));
            $(
                {
                    cache_string.push_str(format!("_{:?}", $v).as_str());
                }
            )*

            let cached = cache.get(cache_string.as_str());

            if cached.is_ok() {
                println!("got cached data {}", cache_string);
                return cached;
            }

            let res = db.$method_name($($v,)*).await?;
            let _ = cache.set(cache_string.as_str(), &res, 7200);
            Ok(res)
        }
    };
}
