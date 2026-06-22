extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::{format_ident, quote};
use syn::{
    FnArg, Ident, ImplItem, ImplItemFn, ItemImpl, Pat, Token,
    parse::{Parse, ParseStream},
    parse_macro_input,
    punctuated::Punctuated,
};

#[proc_macro_attribute]
pub fn generate(_args: TokenStream, input: TokenStream) -> TokenStream {
    let impl_block = parse_macro_input!(input as ItemImpl);
    let original_impl = impl_block.clone();

    let struct_name = get_struct_name(&impl_block);
    let trait_name = format_ident!("{}Interceptor", struct_name);
    let wrapper_name = format_ident!("Intercepted{}", struct_name);

    let mut trait_methods = Vec::new();
    let mut vec_impl_methods = Vec::new();
    let mut wrapper_methods_shared = Vec::new();
    let mut wrapper_methods_mut = Vec::new();

    for item in &impl_block.items {
        if let ImplItem::Fn(method) = item {
            if matches!(method.vis, syn::Visibility::Inherited) {
                continue;
            }

            let is_shared_self = method.sig.inputs.iter().any(|arg| match arg {
                FnArg::Receiver(r) => r.reference.is_some() && r.mutability.is_none(),
                _ => false,
            });
            let is_mut_self = method.sig.inputs.iter().any(|arg| match arg {
                FnArg::Receiver(r) => r.reference.is_some() && r.mutability.is_some(),
                _ => false,
            });

            if !is_shared_self && !is_mut_self {
                continue;
            }

            let method_name = &method.sig.ident;
            let before_name = format_ident!("before_{}", method_name);
            let after_name = format_ident!("after_{}", method_name);

            let (trait_sigs, wrapper_calls, vec_calls, inner_calls, mut_shadows) =
                extract_arguments(method);
            let return_type = get_return_type(method);

            let has_generics = !method.sig.generics.params.is_empty();

            if has_generics {
                let generated = generate_wrapper_method(
                    method,
                    &before_name,
                    &after_name,
                    &wrapper_calls,
                    &inner_calls,
                    &mut_shadows,
                    false, // is_intercepted = false
                );
                if is_mut_self {
                    wrapper_methods_mut.push(generated);
                } else {
                    wrapper_methods_shared.push(generated);
                }
            } else {
                trait_methods.push(generate_trait_methods(
                    &before_name,
                    &after_name,
                    &trait_sigs,
                    &return_type,
                ));
                vec_impl_methods.push(generate_vec_impl_methods(
                    &before_name,
                    &after_name,
                    &trait_sigs,
                    &vec_calls,
                    &return_type,
                ));
                let generated = generate_wrapper_method(
                    method,
                    &before_name,
                    &after_name,
                    &wrapper_calls,
                    &inner_calls,
                    &mut_shadows,
                    true, // is_intercepted = true
                );
                if is_mut_self {
                    wrapper_methods_mut.push(generated);
                } else {
                    wrapper_methods_shared.push(generated);
                }
            }
        }
    }

    TokenStream::from(quote! {
        #original_impl

        pub trait #trait_name: Send + Sync + 'static {
            #(#trait_methods)*
        }

        impl #trait_name for std::vec::Vec<std::sync::Arc<dyn #trait_name>> {
            #(#vec_impl_methods)*
        }

        pub struct #wrapper_name<R> {
            pub inner: R,
            pub interceptor: std::vec::Vec<std::sync::Arc<dyn #trait_name>>,
        }

        impl<R> std::ops::Deref for #wrapper_name<R>
        where
            R: std::ops::Deref<Target = #struct_name>,
        {
            type Target = #struct_name;
            fn deref(&self) -> &Self::Target {
                &*self.inner
            }
        }

        impl<R> std::ops::DerefMut for #wrapper_name<R>
        where
            R: std::ops::DerefMut<Target = #struct_name>,
        {
            fn deref_mut(&mut self) -> &mut Self::Target {
                &mut *self.inner
            }
        }

        impl<R> #wrapper_name<R>
        where
            R: std::ops::Deref<Target = #struct_name>,
        {
            #(#wrapper_methods_shared)*
        }

        impl<R> #wrapper_name<R>
        where
            R: std::ops::DerefMut<Target = #struct_name>,
        {
            #(#wrapper_methods_mut)*
        }
    })
}

fn get_struct_name(impl_block: &ItemImpl) -> &Ident {
    match &*impl_block.self_ty {
        syn::Type::Path(tp) => &tp.path.segments.last().unwrap().ident,
        _ => panic!("Expected impl block on a path type"),
    }
}

fn extract_arguments(
    method: &ImplItemFn,
) -> (
    Vec<TokenStream2>,
    Vec<TokenStream2>,
    Vec<TokenStream2>,
    Vec<TokenStream2>,
    Vec<TokenStream2>,
) {
    let mut trait_sigs = Vec::new();
    let mut wrapper_calls = Vec::new();
    let mut vec_calls = Vec::new();
    let mut inner_calls = Vec::new();
    let mut mut_shadows = Vec::new();

    for arg in &method.sig.inputs {
        if let FnArg::Typed(pat_type) = arg {
            if let Pat::Ident(pat_ident) = &*pat_type.pat {
                let ident = &pat_ident.ident;
                let p_ident = format_ident!("_{}", ident); // The prefixed variable
                let ty = &pat_type.ty;

                match &**ty {
                    syn::Type::Reference(_) => {
                        trait_sigs.push(quote! { #p_ident: #ty });
                        wrapper_calls.push(quote! { #ident });
                        vec_calls.push(quote! { #p_ident });
                        inner_calls.push(quote! { #ident });
                    }
                    _ => {
                        trait_sigs.push(quote! { #p_ident: &mut #ty });
                        wrapper_calls.push(quote! { &mut #p_ident });
                        vec_calls.push(quote! { #p_ident });
                        inner_calls.push(quote! { #p_ident });
                        mut_shadows.push(quote! { let mut #p_ident = #ident; });
                    }
                }
            }
        }
    }
    (
        trait_sigs,
        wrapper_calls,
        vec_calls,
        inner_calls,
        mut_shadows,
    )
}

fn get_return_type(method: &ImplItemFn) -> TokenStream2 {
    match &method.sig.output {
        syn::ReturnType::Default => quote! { () },
        syn::ReturnType::Type(_, ty) => quote! { #ty },
    }
}

fn generate_trait_methods(
    before_name: &Ident,
    after_name: &Ident,
    trait_sigs: &[TokenStream2],
    return_type: &TokenStream2,
) -> TokenStream2 {
    quote! {
        fn #before_name(&self, _ctx: &mut types::plugin::CallContext, #(#trait_sigs),*) -> Option<#return_type> { None }
        fn #after_name(&self, _ctx: &mut types::plugin::CallContext, _result: &mut #return_type) {}
    }
}

fn generate_vec_impl_methods(
    before_name: &Ident,
    after_name: &Ident,
    trait_sigs: &[TokenStream2],
    vec_calls: &[TokenStream2],
    return_type: &TokenStream2,
) -> TokenStream2 {
    quote! {
        fn #before_name(&self, _ctx: &mut types::plugin::CallContext, #(#trait_sigs),*) -> Option<#return_type> {
            for _interceptor in self {
                if let Some(_early_ret) = _interceptor.#before_name(_ctx, #(#vec_calls),*) {
                    return Some(_early_ret);
                }
            }
            None
        }
        fn #after_name(&self, _ctx: &mut types::plugin::CallContext, _result: &mut #return_type) {
            for _interceptor in self {
                _interceptor.#after_name(_ctx, _result);
            }
        }
    }
}

fn generate_wrapper_method(
    method: &ImplItemFn,
    before_name: &Ident,
    after_name: &Ident,
    wrapper_calls: &[TokenStream2],
    inner_calls: &[TokenStream2],
    mut_shadows: &[TokenStream2],
    is_intercepted: bool,
) -> TokenStream2 {
    let method_name = &method.sig.ident;
    let vis = &method.vis;
    let sig = &method.sig;

    if is_intercepted {
        let inner_call = if method.sig.asyncness.is_some() {
            quote! { self.inner.#method_name(#(#inner_calls),*).await }
        } else {
            quote! { self.inner.#method_name(#(#inner_calls),*) }
        };

        quote! {
            #vis #sig {
                #(#mut_shadows)*

                let mut _ctx = types::plugin::CallContext::default();

                if let Some(_early_ret) = self.interceptor.#before_name(&mut _ctx, #(#wrapper_calls),*) {
                    return _early_ret;
                }

                let mut _ret = #inner_call;

                self.interceptor.#after_name(&mut _ctx, &mut _ret);

                _ret
            }
        }
    } else {
        // FIXED: For non-intercepted methods (like generics), grab the original
        // argument names so we don't accidentally try to pass the missing `_` prefixed
        // variables
        let original_names: Vec<_> = method
            .sig
            .inputs
            .iter()
            .filter_map(|arg| {
                if let FnArg::Typed(pat_type) = arg {
                    if let Pat::Ident(pat_ident) = &*pat_type.pat {
                        let ident = &pat_ident.ident;
                        return Some(quote! { #ident });
                    }
                }
                None
            })
            .collect();

        let inner_call = if method.sig.asyncness.is_some() {
            quote! { self.inner.#method_name(#(#original_names),*).await }
        } else {
            quote! { self.inner.#method_name(#(#original_names),*) }
        };

        quote! {
            #vis #sig {
                #inner_call
            }
        }
    }
}

struct InterceptorTypes {
    paths: Punctuated<syn::Path, Token![,]>,
}

impl Parse for InterceptorTypes {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(InterceptorTypes {
            paths: input.parse_terminated(syn::Path::parse, Token![,])?,
        })
    }
}

#[proc_macro]
pub fn generate_plugin_system(input: TokenStream) -> TokenStream {
    let parsed_input = parse_macro_input!(input as InterceptorTypes);

    let mut fields = Vec::new();
    let mut defaults = Vec::new();
    let mut getters = Vec::new();
    let mut builder_impls = Vec::new();
    let mut init_statements = Vec::new();

    for path in parsed_input.paths {
        let segments: Vec<_> = path.segments.iter().collect();
        let struct_ident = &segments.last().unwrap().ident;
        let snake_case = to_snake_case(&struct_ident.to_string());

        let mod_segments = &segments[0..segments.len() - 1];

        let mod_prefix = if !mod_segments.is_empty() {
            quote! { #(#mod_segments)::* :: }
        } else {
            let default_mod = format_ident!("{}", snake_case);
            quote! { #default_mod :: }
        };

        let field_ident = format_ident!("{}", snake_case);
        let trait_ident = format_ident!("{}Interceptor", struct_ident);
        let wrapper_ident = format_ident!("Intercepted{}", struct_ident);
        let getter_ident = format_ident!("get_{}", snake_case);
        let getter_mut_ident = format_ident!("get_{}_mut", snake_case);
        let marker_ident = format_ident!("{}Marker", struct_ident);

        fields.push(quote! {
            pub #field_ident: Vec<std::sync::Arc<dyn #mod_prefix #trait_ident>>
        });

        defaults.push(quote! {
            #field_ident: Vec::new()
        });

        getters.push(quote! {
            pub async fn #getter_ident(&self) -> #mod_prefix #wrapper_ident<tokio::sync::OwnedRwLockReadGuard<#mod_prefix #struct_ident>> {
                let _db = self.plugins.get::<#mod_prefix #struct_ident>();
                let _guard = _db.read_owned().await;
                #mod_prefix #wrapper_ident {
                    inner: _guard,
                    interceptor: self.interceptors.#field_ident.clone(),
                }
            }

            pub async fn #getter_mut_ident(&self) -> #mod_prefix #wrapper_ident<tokio::sync::OwnedRwLockWriteGuard<#mod_prefix #struct_ident>> {
                let _db = self.plugins.get::<#mod_prefix #struct_ident>();
                let _guard = _db.write_owned().await;
                #mod_prefix #wrapper_ident {
                    inner: _guard,
                    interceptor: self.interceptors.#field_ident.clone(),
                }
            }
        });

        builder_impls.push(quote! {
            #[doc(hidden)]
            pub struct #marker_ident;

            impl<T> RegisterInterceptor<#marker_ident> for T
            where
                T: #mod_prefix #trait_ident + 'static,
            {
                fn register(self, _registry: &mut Interceptors) {
                    _registry.#field_ident.push(std::sync::Arc::new(self) as std::sync::Arc<dyn #mod_prefix #trait_ident>);
                }
            }
        });

        init_statements.push(quote! {
            _registry.register::<#path>(<#path as types::plugin::Plugin>::init(_context));
        });
    }

    TokenStream::from(quote! {
        pub struct Interceptors {
            #(#fields),*
        }

        impl Default for Interceptors {
            fn default() -> Self {
                Self {
                    #(#defaults),*
                }
            }
        }

        impl crate::StateManager {
            #(#getters)*
        }

        pub trait RegisterInterceptor<Marker> {
            fn register(self, registry: &mut Interceptors);
        }

        impl Interceptors {
            pub fn with<M, I>(mut self, interceptor: I) -> Self
            where
                I: RegisterInterceptor<M>,
            {
                interceptor.register(&mut self);
                self
            }
        }

        #(#builder_impls)*

        pub fn init_all_plugins(_registry: &mut types::plugin::PluginRegistry, _context: &types::plugin::PluginContext) {
            #(#init_statements)*
        }
    })
}

fn to_snake_case(s: &str) -> String {
    let mut res = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                res.push('_');
            }
            res.push(c.to_ascii_lowercase());
        } else {
            res.push(c);
        }
    }
    res
}
