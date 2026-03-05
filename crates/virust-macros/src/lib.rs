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

        // Register handler in global inventory
        // TODO: Implement inventory registration when multiple handlers are supported
        // inventory::submit!(__RouteEntry {
        //     name: stringify!(#fn_name),
        //     route_type: __RouteType::WebSocket,
        // });

        // For now, just store the route info in a static
        #[cfg(feature = "inventory")]
        inventory::submit!(__RouteEntry {
            name: stringify!(#fn_name),
            route_type: __RouteType::WebSocket,
        });
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn get(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn post(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn put(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}

#[proc_macro_attribute]
pub fn delete(_attr: TokenStream, item: TokenStream) -> TokenStream {
    item
}
