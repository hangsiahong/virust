use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, Ident};

/// Implementation of the cache attribute macro
pub fn cache(attrs: TokenStream, input: TokenStream) -> TokenStream {
    // Parse the attribute arguments
    let attr_str = attrs.to_string();
    let input_fn = parse_macro_input!(input as ItemFn);

    // Parse max_age argument from attribute string
    // Handle format: "max_age = 60"
    let max_age_expr = if attr_str.contains("max_age") {
        // Extract the number after "max_age"
        let parts: Vec<&str> = attr_str.split('=').collect();
        if parts.len() >= 2 {
            let value_str = parts[1].trim().trim_end_matches(',').trim();
            if let Ok(value) = value_str.parse::<u64>() {
                quote! { #value }
            } else {
                // Return compile error if parsing fails
                return syn::Error::new(
                    syn::spanned::Spanned::span(&input_fn),
                    format!("Invalid max_age value: '{}'. Expected a positive integer.", value_str)
                ).to_compile_error().into();
            }
        } else {
            return syn::Error::new(
                syn::spanned::Spanned::span(&input_fn),
                "Missing value for max_age parameter. Usage: #[cache(max_age = 60)]"
            ).to_compile_error().into();
        }
    } else {
        return syn::Error::new(
            syn::spanned::Spanned::span(&input_fn),
            "Missing max_age parameter. Usage: #[cache(max_age = 60)]"
        ).to_compile_error().into();
    };

    // Keep original function unchanged
    let fn_name = &input_fn.sig.ident;
    let fn_attrs = &input_fn.attrs;
    let fn_vis = &input_fn.vis;
    let fn_sig = &input_fn.sig;
    let fn_body = &input_fn.block;

    // Create a wrapper struct name with proper camelCase
    let fn_name_str = fn_name.to_string();
    let struct_name = Ident::new(
        &format!("{}CacheMeta", fn_name_str),
        fn_name.span()
    );

    // Output: keep function + add compile-time registration via wrapper struct
    let expanded = quote! {
        #(#fn_attrs)*
        #fn_vis #fn_sig #fn_body

        // Register cache metadata at compile time
        #[automatically_derived]
        pub struct #struct_name;

        #[automatically_derived]
        impl virust_build::CacheRouteMetadata for #struct_name {
            const MAX_AGE: u64 = #max_age_expr;

            fn route_path() -> &'static str {
                stringify!(#fn_name)
            }
        }
    };

    TokenStream::from(expanded)
}
