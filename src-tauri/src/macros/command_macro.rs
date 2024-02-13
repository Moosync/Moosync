#[macro_export]
macro_rules! generate_command {
    ($method_name:ident, $state:ident, $ret:ty, $($v:ident: $t:ty),*) => {
        #[tauri::command(async)]
        pub fn $method_name(db: State<$state>, $($v: $t),*) -> Result<$ret, String> {
            db.$method_name($($v,)*).map_err(|e| e.to_string())
        }
    };
}
