use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

#[proc_macro_attribute]
pub fn ws(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    // Generate handler wrapper
    let expanded = quote! {
        #input

        // Type definitions for route registry
        #[derive(Debug, Clone)]
        enum __RouteType {
            WebSocket,
            HttpGet,
            HttpPost,
            HttpPut,
            HttpDelete,
        }

        #[derive(Debug, Clone)]
        struct __RouteEntry {
            name: &'static str,
            route_type: __RouteType,
        }

        inventory::collect!(__RouteEntry);

        // Register handler in global inventory
        inventory::submit!(__RouteEntry {
            name: stringify!(#fn_name),
            route_type: __RouteType::WebSocket,
        });
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn get(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    let expanded = quote! {
        #input

        inventory::submit!(virust_macros::RouteEntry {
            name: stringify!(#fn_name).to_string(),
            route_type: virust_macros::RouteType::HttpGet,
        });
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn post(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    let expanded = quote! {
        #input

        inventory::submit!(virust_macros::RouteEntry {
            name: stringify!(#fn_name).to_string(),
            route_type: virust_macros::RouteType::HttpPost,
        });
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn put(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    let expanded = quote! {
        #input

        inventory::submit!(virust_macros::RouteEntry {
            name: stringify!(#fn_name).to_string(),
            route_type: virust_macros::RouteType::HttpPut,
        });
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn delete(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    let expanded = quote! {
        #input

        inventory::submit!(virust_macros::RouteEntry {
            name: stringify!(#fn_name).to_string(),
            route_type: virust_macros::RouteType::HttpDelete,
        });
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn typescript(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    // Generate TypeScript code
    let ts_code = virust_typescript::generate_typescript(
        &fn_name.to_string(),
        &format!("{}Input", fn_name),
        "{ /* input fields */ }",
        &format!("{}Output", fn_name),
        "{ /* output fields */ }"
    );

    let expanded = quote! {
        #input

        #ts_code
    };

    TokenStream::from(expanded)
}
