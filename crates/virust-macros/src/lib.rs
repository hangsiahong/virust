use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn};

/// HTTP method enum for route detection (internal use only)
#[derive(Debug, Clone, Copy)]
enum Method {
    Get,
    Post,
    Put,
    Delete,
}

#[proc_macro_attribute]
pub fn ws(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    let expanded = quote! {
        #input

        inventory::submit!(virust_runtime::RouteEntry {
            path: stringify!(#fn_name),
            route_type: virust_runtime::RouteType::WebSocket,
        });
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn get(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;

    // Check if function name is an HTTP method
    let http_method = match fn_name.to_string().to_uppercase().as_str() {
        "GET" => Method::Get,
        "POST" => Method::Post,
        "PUT" => Method::Put,
        "DELETE" => Method::Delete,
        _ => Method::Get, // Default to GET
    };

    // Map Method to RouteType
    let route_type = match http_method {
        Method::Get => quote!(virust_runtime::RouteType::HttpGet),
        Method::Post => quote!(virust_runtime::RouteType::HttpPost),
        Method::Put => quote!(virust_runtime::RouteType::HttpPut),
        Method::Delete => quote!(virust_runtime::RouteType::HttpDelete),
    };

    let expanded = quote! {
        #input

        inventory::submit!(virust_runtime::RouteEntry {
            path: stringify!(#fn_name),
            route_type: #route_type,
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

        inventory::submit!(virust_runtime::RouteEntry {
            path: stringify!(#fn_name),
            route_type: virust_runtime::RouteType::HttpPost,
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

        inventory::submit!(virust_runtime::RouteEntry {
            path: stringify!(#fn_name),
            route_type: virust_runtime::RouteType::HttpPut,
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

        inventory::submit!(virust_runtime::RouteEntry {
            path: stringify!(#fn_name),
            route_type: virust_runtime::RouteType::HttpDelete,
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
