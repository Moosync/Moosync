#[macro_export]
macro_rules! generate_command {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        #[tauri::command(async)]
        pub fn $method_name(db: State<$state>, $($v: $t),*) -> types::errors::errors::Result<$ret> {
            db.$method_name($($v,)*)
        }
    };
}

#[macro_export]
macro_rules! generate_command_async {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        #[tauri::command(async)]
        pub async fn $method_name(db: State<'_, $state>, $($v: $t),*) -> types::errors::errors::Result<$ret> {
            Ok(db.$method_name($($v,)*).await?)
        }
    };
}
