use quote::quote;
use rstest::rstest;
use tracing_test::traced_test;

use crate::{generate_internal, generate_plugin_system_internal, to_snake_case};

#[rstest]
#[case("Database", "database")]
#[case("PreferenceConfig", "preference_config")]
#[case("ScannerHolder", "scanner_holder")]
#[case("MprisHolder", "mpris_holder")]
#[case("simple", "simple")]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_to_snake_case(#[case] input: &str, #[case] expected: &str) {
    assert_eq!(to_snake_case(input), expected);
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_generate_internal_methods() {
    let input = quote! {
        impl MyService {
            pub fn shared_sync(&self, name: String, flag: &bool) -> i32 {
                42
            }

            pub async fn shared_async(&self, id: u32) {
            }

            pub fn mut_sync(&mut self, count: i32) -> bool {
                true
            }

            pub async fn mut_async(&mut self) -> Result<(), ()> {
                Ok(())
            }

            pub fn generic_sync<T>(&self, item: T) -> T {
                item
            }

            pub async fn generic_async<T>(&self, item: T) -> T {
                item
            }

            fn private_helper(&self) {
            }

            pub fn static_function() -> u64 {
                100
            }

            const MY_CONST: usize = 10;
        }
    };

    let output = generate_internal(input);
    let output_str = output.to_string();

    assert!(output_str.contains("pub trait MyServiceInterceptor"));
    assert!(output_str.contains("pub struct InterceptedMyService"));
    assert!(output_str.contains("fn before_shared_sync"));
    assert!(output_str.contains("fn after_shared_sync"));
    assert!(output_str.contains("fn before_mut_sync"));
    assert!(output_str.contains("fn after_mut_sync"));
    assert!(output_str.contains("impl < R > std :: ops :: Deref for InterceptedMyService < R >"));
    assert!(
        output_str.contains("impl < R > std :: ops :: DerefMut for InterceptedMyService < R >")
    );
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_generate_internal_invalid_syntax() {
    let input = quote! {
        not an impl block
    };

    let output = generate_internal(input);
    let output_str = output.to_string();

    assert!(output_str.contains("compile_error"));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_generate_plugin_system_internal() {
    let input = quote! {
        my_mod::sub::Database,
        StandalonePlugin
    };

    let output = generate_plugin_system_internal(input);
    let output_str = output.to_string();

    assert!(output_str.contains("pub struct Interceptors"));
    assert!(output_str.contains("pub database :"));
    assert!(output_str.contains("DatabaseInterceptor"));
    assert!(output_str.contains("pub standalone_plugin :"));
    assert!(output_str.contains("StandalonePluginInterceptor"));
    assert!(output_str.contains("pub fn get_database"));
    assert!(output_str.contains("pub fn get_standalone_plugin"));
    assert!(output_str.contains("pub fn init_all_plugins"));
}

#[rstest]
#[traced_test]
#[tracing::instrument(level = "debug", skip_all)]
fn test_generate_plugin_system_invalid_syntax() {
    let input = quote! {
        /// invalid :::
    };

    let output = generate_plugin_system_internal(input);
    let output_str = output.to_string();

    assert!(output_str.contains("compile_error"));
}
