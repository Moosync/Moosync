use std::{collections::HashMap, env, fs, path::Path};

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, ToTokens};
use syn::{parse_macro_input, ExprArray, Ident};

use crate::common::FnDetails;

pub fn generate_tauri_invoke_wrapper(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ExprArray);
    let valid_crates = input
        .elems
        .iter()
        .filter_map(|e| {
            if let syn::Expr::Lit(lit) = e {
                if let syn::Lit::Str(lit) = &lit.lit {
                    return Some(lit.value());
                }
            }
            None
        })
        .collect();

    let out_dir = env::var("TAURI_INVOKE_PROC_DIR")
        .expect("TAURI_INVOKE_PROC_DIR environment variable is not set");

    let file_path = Path::new(&out_dir).join("function_details.json");

    let json_content = fs::read_to_string(file_path).expect("Failed to read function_details.json");
    let json: HashMap<String, Vec<FnDetails>> = serde_json::from_str(&json_content)
        .expect("Failed to parse JSON from function_details.json");

    let generated_fns = parse_crate(json, valid_crates);

    let output = quote! {
        #(#generated_fns)*
    };

    output.into()
}

fn parse_crate(
    json: HashMap<String, Vec<FnDetails>>,
    valid_crates: Vec<String>,
) -> Vec<proc_macro2::TokenStream> {
    let mut ret = vec![];
    for funcs in json.values() {
        for func in funcs {
            ret.push(parse_fn(func, &valid_crates));
        }
    }

    ret
}

#[derive(Clone)]
struct FnNameArg {
    name: proc_macro2::Ident,
    typ: syn::Type,
}

impl ToTokens for FnNameArg {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let ident = &self.name;
        let typ = &self.typ; // Destructure the tuple
        let generated = quote! {
            #ident: #typ
        };
        tokens.extend(generated);
    }
}

fn is_external_allowed(type_name: &str) -> bool {
    matches!(type_name, "serde_json" | "std")
}

fn is_primitive_type(type_name: &str) -> bool {
    matches!(
        type_name,
        "bool"
            | "char"
            | "str"
            | "String"
            | "i8"
            | "i16"
            | "i32"
            | "i64"
            | "i128"
            | "isize"
            | "u8"
            | "u16"
            | "u32"
            | "u64"
            | "u128"
            | "usize"
            | "f32"
            | "f64"
            | "Option"
            | "Vec"
    )
}

fn is_valid_type(arg_type: &syn::Type, valid_crates: &Vec<String>) -> bool {
    match arg_type {
        // Check simple type paths (e.g., String, serde_json::Value)
        syn::Type::Path(type_path) => {
            if let Some(last_segment) = type_path.path.segments.last() {
                // Check for generics in the type (e.g., Vec<valid_type>)
                if let syn::PathArguments::AngleBracketed(args) = &last_segment.arguments {
                    // Validate all generic arguments
                    let is_valid = args.args.iter().all(|arg| match arg {
                        syn::GenericArgument::Type(inner_type) => {
                            is_valid_type(inner_type, valid_crates)
                        }
                        _ => true, // Ignore lifetimes or other non-type arguments
                    });

                    if !is_valid {
                        return false;
                    }
                }
            }

            // Extract the leading segment (crate or module name)
            if let Some(first_segment) = type_path.path.segments.first() {
                let type_name = first_segment.ident.to_string();
                // Allow all primitive types implicitly
                if is_primitive_type(&type_name) || is_external_allowed(&type_name) {
                    return true;
                }

                // Check if the type belongs to a valid crate or is a primitive
                if valid_crates
                    .iter()
                    .any(|valid| type_name.starts_with(valid))
                {
                    return true;
                }
            }
            false
        }
        // Allow primitives explicitly
        syn::Type::Reference(type_ref) => {
            is_valid_type(&type_ref.elem, valid_crates) // Validate referenced type
        }
        syn::Type::Tuple(type_tuple) => {
            // Validate all elements of a tuple (e.g., (valid_type, valid_type))
            type_tuple
                .elems
                .iter()
                .all(|elem| is_valid_type(elem, valid_crates))
        }
        _ => false, // Disallow all other types
    }
}

fn replace_serde_json_with_jsvalue(
    ty: &syn::Type,
    has_generics: bool,
    generic_count: u64,
) -> (u64, syn::Type) {
    let mut new_count = generic_count;
    match ty {
        syn::Type::Path(type_path) => {
            let mut type_path = type_path.clone();

            // Check if the type is `serde_json::Value`
            if let Some(first_segment) = type_path.path.segments.first_mut() {
                if first_segment.ident == "serde_json" {
                    if let syn::PathArguments::None = first_segment.arguments {
                        if type_path.path.segments.len() == 2
                            && type_path.path.segments[1].ident == "Value"
                        {
                            if has_generics {
                                return (
                                    new_count + 1,
                                    syn::parse_str(format!("T{}", new_count + 1).as_str())
                                        .expect("Failed to parse replacement generic type"),
                                );
                            }
                            // Replace `serde_json::Value` with `serde_wasm_bindgen::JsValue`
                            return (
                                new_count + 1,
                                syn::parse_str("wasm_bindgen::JsValue")
                                    .expect("Failed to parse replacement type"),
                            );
                        }
                    }
                }
            }

            // Recursively process generic arguments, if any
            for segment in &mut type_path.path.segments {
                if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    for generic_arg in &mut args.args {
                        if let syn::GenericArgument::Type(inner_ty) = generic_arg {
                            let (generic_count, new_val) =
                                replace_serde_json_with_jsvalue(inner_ty, has_generics, new_count);
                            *inner_ty = new_val;
                            new_count = generic_count;
                        }
                    }
                }
            }

            (new_count, syn::Type::Path(type_path))
        }
        syn::Type::Reference(type_ref) => {
            replace_serde_json_with_jsvalue(&type_ref.elem, has_generics, new_count)
            // Validate referenced type
        }
        syn::Type::Tuple(type_tuple) => {
            // Validate all elements of a tuple (e.g., (valid_type, valid_type))
            let mut type_tuple = type_tuple.clone();
            type_tuple.elems = type_tuple
                .elems
                .into_iter()
                .map(|elem| {
                    let (generic_count, typ) =
                        replace_serde_json_with_jsvalue(&elem, has_generics, new_count);
                    new_count = generic_count;
                    typ
                })
                .collect();
            (new_count, syn::Type::Tuple(type_tuple))
        }
        _ => (new_count, ty.clone()),
    }
}

fn parse_fn(dets: &FnDetails, valid_crates: &Vec<String>) -> proc_macro2::TokenStream {
    let invoke_name_lit = dets.name.clone();
    let func_name_ident = syn::Ident::new(&dets.name, Span::call_site());

    let mut generics_needed = 0;

    let args = dets
        .args
        .iter()
        .filter_map(|arg| {
            if !arg.arg_type.starts_with("tauri::") {
                let arg_name = syn::Ident::new(&arg.name, proc_macro2::Span::call_site());

                let mut arg_type = syn::parse_str::<syn::Type>(&arg.arg_type).unwrap();
                if !is_valid_type(&arg_type, valid_crates) {
                    eprintln!(
                        "Found not allowed type {}. Parsing it as serde_json::Value",
                        arg.arg_type
                    );
                    arg_type = syn::parse_str::<syn::Type>("wasm_bindgen::JsValue").unwrap();
                }

                let (generic_count, arg_type) = replace_serde_json_with_jsvalue(&arg_type, true, 0);
                generics_needed = generic_count;
                return Some(FnNameArg {
                    name: arg_name,
                    typ: arg_type,
                });
            }
            None
        })
        .collect::<Vec<_>>();

    let mut is_ret_value = false;

    // Parse return type, if present
    let ret_type = if let Some(ret) = &dets.ret {
        // Parse the return type string into a `syn::Type`
        let mut parsed_ret_type =
            syn::parse_str::<syn::Type>(ret).expect("Failed to parse return type");
        if !is_valid_type(&parsed_ret_type, valid_crates) {
            eprintln!(
                "Found not allowed type {}. Parsing it as wasm_bindgen::JsValue",
                ret
            );
            is_ret_value = true;
            parsed_ret_type =
                syn::parse_str::<syn::Type>("types::errors::Result<wasm_bindgen::JsValue>")
                    .unwrap();
        }

        let (is_changed, parsed_ret_type) =
            replace_serde_json_with_jsvalue(&parsed_ret_type, false, 0);
        if is_changed > 0 {
            is_ret_value = true;
        }
        quote! { -> #parsed_ret_type }
    } else {
        // No return type (i.e., `()`)
        quote! {}
    };

    let binding = args.clone();
    let struct_fields = binding.into_iter().map(|arg| {
        let arg_name = arg.name;
        let arg_typ = arg.typ;
        quote! {
            pub #arg_name: #arg_typ
        }
    });

    let binding = args.clone();
    let struct_values = binding.into_iter().map(|arg| {
        let arg_name = arg.name;
        quote! {
            #arg_name
        }
    });

    let ret_val = if is_ret_value {
        quote! {
            Ok(res)
        }
    } else {
        quote! {
            Ok(serde_wasm_bindgen::from_value(res)?)
        }
    };

    let (generic_params, where_clause) = if generics_needed > 0 {
        let generics = (1..=generics_needed)
            .map(|i| Ident::new(&format!("T{}", i), Span::call_site()))
            .collect::<Vec<Ident>>();
        (
            quote! { <#(#generics),*> },
            quote! {
                where #(#generics: serde::Serialize + 'static),*
            },
        )
    } else {
        (quote! {}, quote! {})
    };

    quote! {
        pub async fn #func_name_ident #generic_params(#(#args),*) #ret_type #where_clause {
            #[derive(serde::Serialize)]
            struct Args #generic_params #where_clause {
                #(#struct_fields),*
            }

            let args = serde_wasm_bindgen::to_value(&Args {
                #(#struct_values),*
            }).unwrap();

            let res = crate::utils::common::invoke(#invoke_name_lit, args)
                .await?;

            #ret_val
        }
    }
}
