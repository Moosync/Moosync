// Moosync
// Copyright (C) 2024, 2025  Moosync <support@moosync.app>
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

/// Completely written using ChatGPT
/// Except the use statement parsing logic
use once_cell::sync::Lazy;
use proc_macro::TokenStream;
use quote::quote;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::path::Path;
use std::sync::Mutex;
use std::{env, fs};
use syn::{parse_macro_input, FnArg, ItemFn, ReturnType, Type};

use crate::common::{FnArgs, FnDetails};

lazy_static::lazy_static! {
    static ref FUNCTION_DETAILS: Mutex<Vec<FnDetails>> = Mutex::new(Vec::new());
    static ref TYPE_CACHE: Lazy<HashMap<String, HashSet<String>>> = Lazy::new(|| {
        let crate_path = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
        let src_path = Path::new(&crate_path).join("src");
        parse_use_statements_from_source(&src_path)
    });
}

pub fn generate_tauri_invoke_wrapper(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse the input TokenStream as a function
    let input = parse_macro_input!(item as ItemFn);

    // Extract the function name
    let fn_name = input.sig.ident.to_string();

    // Extract the arguments and resolve their types
    let args = input
        .sig
        .inputs
        .iter()
        .filter_map(|arg| match arg {
            FnArg::Typed(pat_type) => {
                if let syn::Pat::Ident(ident) = &*pat_type.pat {
                    let raw_type = resolve_type_path(&pat_type.ty);
                    Some(FnArgs {
                        name: ident.ident.to_string(),
                        arg_type: raw_type,
                    })
                } else {
                    None
                }
            }
            _ => None,
        })
        .collect::<Vec<_>>();

    let ret = match &input.sig.output {
        ReturnType::Type(_, ty) => Some(resolve_type_path(ty)),
        ReturnType::Default => None, // No return type (i.e., "-> ()")
    };

    // Add function details to the global variable
    let details = FnDetails {
        name: fn_name,
        args,
        ret,
    };
    FUNCTION_DETAILS
        .lock()
        .expect("Failed to acquire lock on FUNCTION_DETAILS")
        .push(details);

    // Return the original function unchanged
    TokenStream::from(quote! {
        #input
    })
}

/// Resolves a raw type using the cached `use` map. If resolution fails, returns the raw type as-is.
fn resolve_with_cache(raw_type: &str) -> String {
    let import = raw_type.split("::").last();
    if let Some(import) = import {
        let imports = TYPE_CACHE.get(import);
        if let Some(imports) = imports {
            return imports
                .iter()
                .find(|i| i.contains(raw_type))
                .cloned()
                .unwrap_or(raw_type.to_string());
        }
    }
    raw_type.to_string()
}

/// Reads all `.rs` files in the `src` directory recursively and extracts `use` statements
fn parse_use_statements_from_source(src_path: &Path) -> HashMap<String, HashSet<String>> {
    let mut use_map = HashMap::new();

    fn recursive_scan(dir: &Path, use_map: &mut HashMap<String, HashSet<String>>) {
        for entry in fs::read_dir(dir).expect("Failed to read directory") {
            let entry = entry.expect("Failed to read directory entry");
            let path = entry.path();

            if path.is_dir() {
                recursive_scan(&path, use_map);
            } else if path.is_file() && path.extension().is_some_and(|ext| ext == "rs") {
                let source_code = fs::read_to_string(&path).expect("Failed to read source file");
                parse_use_statements(&source_code, use_map);
            }
        }
    }

    recursive_scan(src_path, &mut use_map);
    use_map
}

/// Parses `use` statements from the given source code and populates the `use_map`
fn parse_use_statements(source: &str, use_map: &mut HashMap<String, HashSet<String>>) {
    // Regex to match `use` statements, including those spanning multiple lines
    let use_regex = Regex::new(r"use\s+([^;]+);").expect("Invalid regex");

    // Find all matches for `use` statements
    for caps in use_regex.captures_iter(source) {
        let full_use_statement = &caps[1];
        process_use_statement(full_use_statement.trim(), use_map);
    }
}

/// Processes a single `use` statement recursively, handling nested structures
fn process_use_statement(statement: &str, use_map: &mut HashMap<String, HashSet<String>>) {
    let mut queue = vec![];
    let mut ret = vec![];
    let mut last_index = 0;
    for (i, c) in statement.char_indices() {
        match c {
            '{' => {
                let end = statement[last_index..i].trim();
                if !end.is_empty() {
                    queue.push(end.to_string());
                }
                last_index = i + 1;
            }
            '}' => {
                let end = statement[last_index..i].trim();
                if !end.is_empty() {
                    ret.push(format!("{}{}", queue.join(""), end));
                }
                queue.pop();
                last_index = i + 1;
            }
            ',' => {
                let end = statement[last_index..i].trim();
                if !end.is_empty() {
                    ret.push(format!("{}{}", queue.join(""), end));
                }
                last_index = i + 1;
            }
            _ => {}
        }
    }

    let rem = statement[last_index..statement.len()].trim();
    if !rem.is_empty() {
        ret.push(rem.to_string());
    }

    for item in ret {
        let import = item.split("::").last();
        if let Some(import) = import {
            if use_map.contains_key(import) {
                let existing = use_map.get_mut(import);
                if let Some(existing) = existing {
                    existing.insert(item);
                }
            } else {
                let mut hash_set = HashSet::new();
                hash_set.insert(item.clone());
                use_map.insert(import.to_string(), hash_set);
            }
        }
    }
}

/// Extracts the raw type as a string from a `syn::Type`
#[allow(clippy::borrowed_box)]
fn resolve_type_path(ty: &Box<Type>) -> String {
    match &**ty {
        // Handle simple type paths (e.g., `A`, `Vec<A>`)
        Type::Path(type_path) => {
            // Base type (e.g., `Vec`, `HashMap`)
            let resolved_segments = type_path
                .path
                .segments
                .iter()
                .map(|s| s.ident.to_string())
                .collect::<Vec<_>>()
                .join("::");

            // Combine the resolved segments into a fully qualified path
            let base_type = resolve_with_cache(&resolved_segments);

            // Check for generics (e.g., `<A, B>`)
            if let Some(generic_args) = type_path
                .path
                .segments
                .last() // Generics are associated with the last segment
                .and_then(|segment| match &segment.arguments {
                    syn::PathArguments::AngleBracketed(generics) => Some(&generics.args),
                    _ => None,
                })
            {
                // Process each generic argument recursively
                let generic_types: Vec<String> = generic_args
                    .iter()
                    .filter_map(|arg| match arg {
                        syn::GenericArgument::Type(inner_ty) => {
                            Some(resolve_type_path(&Box::new(inner_ty.clone())))
                        }
                        _ => None, // Skip non-type generics (e.g., lifetimes)
                    })
                    .collect();

                // Return the base type with resolved generics
                format!("{}<{}>", base_type, generic_types.join(", "))
            } else {
                // No generics, return the resolved base type
                base_type
            }
        }
        Type::Tuple(type_tuple) => {
            // Recursively resolve each element in the tuple
            let resolved_elements: Vec<String> = type_tuple
                .elems
                .iter()
                .map(|elem| resolve_type_path(&Box::new(elem.clone())))
                .collect();

            // Return the resolved tuple as `(A, B, C, ...)`
            format!("({})", resolved_elements.join(", "))
        }
        Type::Reference(type_ref) => {
            // Skip `&` and `mut`, resolve the inner type
            resolve_type_path(&type_ref.elem)
        }
        _ => quote::quote!(#ty).to_string(),
    }
}

// Ensure the global variable is written to the file at program exit
#[ctor::dtor]
fn write_function_details_to_file() {
    // Retrieve the crate name from the environment variable
    let crate_name =
        std::env::var("CARGO_PKG_NAME").unwrap_or_else(|_| "unknown_crate".to_string());

    // Lock the global FUNCTION_DETAILS and serialize its content
    let data = FUNCTION_DETAILS
        .lock()
        .expect("Failed to acquire lock on FUNCTION_DETAILS");

    // Convert the current FUNCTION_DETAILS to a JSON object under the crate name
    let current_data = serde_json::json!({ crate_name.clone(): &*data });
    if current_data[crate_name.clone()]
        .as_array()
        .cloned()
        .unwrap_or_default()
        .is_empty()
    {
        return;
    }

    // Path to the output JSON file in the source directory
    let crate_path = env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR is not set");
    let file_path = Path::new(&crate_path)
        .join("function_details.json");

    // Read existing JSON file content, if any
    let mut existing_data = match std::fs::read_to_string(file_path.clone()) {
        Ok(content) => serde_json::from_str(&content).unwrap_or_else(|_| serde_json::json!({})),
        Err(_) => serde_json::json!({}), // If file doesn't exist, start with an empty object
    };

    // Merge the current data into the existing data
    if let serde_json::Value::Object(ref mut map) = existing_data {
        map.insert(crate_name.clone(), current_data[crate_name].clone());
    } else {
        existing_data = current_data;
    }

    // Serialize the updated data and write it back to the file
    let json_output =
        serde_json::to_string_pretty(&existing_data).expect("Failed to serialize JSON data");

    eprintln!("Writing output to {:?}", file_path);
    std::fs::write(file_path, json_output).expect("Failed to write JSON data to file");
}
