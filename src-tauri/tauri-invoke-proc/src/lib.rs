use proc_macro::TokenStream;

mod common;
mod core;
mod ui;

#[proc_macro_attribute]
pub fn parse_tauri_command(attr: TokenStream, item: TokenStream) -> TokenStream {
    core::generate_tauri_invoke_wrapper(attr, item)
}

#[proc_macro]
pub fn generate_tauri_invoke(item: TokenStream) -> TokenStream {
    ui::generate_tauri_invoke_wrapper(item)
}
