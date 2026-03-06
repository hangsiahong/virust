use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemFn, ReturnType, Type, FnArg, Pat, Ident};
use syn::punctuated::Punctuated;
use syn::token::Comma;

/// HTTP method enum for route detection (internal use only)
#[derive(Debug, Clone, Copy)]
enum Method {
    Get,
    Post,
    Put,
    Delete,
}

/// Extract type name from a syn::Type, defaulting to "any" if unknown
fn extract_type_name(ty: &Type) -> String {
    match ty {
        Type::Path(type_path) => {
            if let Some(segment) = type_path.path.segments.last() {
                segment.ident.to_string()
            } else {
                "any".to_string()
            }
        }
        _ => "any".to_string(),
    }
}

/// Extract input and output type names from function signature
fn extract_function_types(input: &ItemFn) -> (String, String) {
    // Extract input type from first parameter
    let input_type = input.sig.inputs.iter()
        .next()
        .and_then(|arg| match arg {
            syn::FnArg::Typed(pat_type) => Some(extract_type_name(&pat_type.ty)),
            _ => None,
        })
        .unwrap_or_else(|| "any".to_string());

    // Extract output type from return type
    let output_type = match &input.sig.output {
        ReturnType::Type(_, ty) => extract_type_name(ty),
        ReturnType::Default => "void".to_string(),
    };

    (input_type, output_type)
}

/// Route argument with metadata
#[derive(Debug)]
enum RouteArg {
    Path(PathArg),
    Body(BodyArg),
}

/// Path parameter argument
#[derive(Debug)]
struct PathArg {
    name: Ident,
    typ: Type,
}

/// Body parameter argument
#[derive(Debug)]
struct BodyArg {
    name: Ident,
    typ: Type,
}

/// Parse route arguments from function parameters, extracting #[path] and #[body] attributes
fn parse_route_args(args: &Punctuated<FnArg, Comma>) -> Vec<RouteArg> {
    args.iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat) = arg {
                // Check for #[path] attribute
                let is_path = pat.attrs.iter().any(|attr| {
                    attr.path().is_ident("path")
                });

                // Check for #[body] attribute
                let is_body = pat.attrs.iter().any(|attr| {
                    attr.path().is_ident("body")
                });

                if let Pat::Ident(ident) = &*pat.pat {
                    if is_path {
                        Some(RouteArg::Path(PathArg {
                            name: ident.ident.clone(),
                            typ: (*pat.ty).clone(),
                        }))
                    } else if is_body {
                        Some(RouteArg::Body(BodyArg {
                            name: ident.ident.clone(),
                            typ: (*pat.ty).clone(),
                        }))
                    } else {
                        None
                    }
                } else {
                    None
                }
            } else {
                None
            }
        })
        .collect()
}

/// Strip #[path] and #[body] attributes from function parameters
fn strip_arg_attributes(mut input: ItemFn) -> ItemFn {
    for arg in input.sig.inputs.iter_mut() {
        if let FnArg::Typed(pat) = arg {
            pat.attrs.retain(|attr| {
                // Remove #[path] and #[body] attributes
                !attr.path().is_ident("path") && !attr.path().is_ident("body")
            });
        }
    }
    input
}

#[proc_macro_attribute]
pub fn ws(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = &input.sig.ident;
    let _fn_name_str = fn_name.to_string();

    // Extract type information
    let (_input_type, _output_type) = extract_function_types(&input);

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
    let fn_name = input.sig.ident.clone();
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;

    // Parse route arguments to extract #[path] and #[body] metadata
    let route_args = parse_route_args(&input.sig.inputs);

    // Filter for path parameters
    let path_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Path(p) => Some(p),
            _ => None,
        })
        .collect();

    // Filter for body parameters
    let body_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Body(b) => Some(b),
            _ => None,
        })
        .collect();

    // Strip #[path] and #[body] attributes from function parameters
    let original_fn = strip_arg_attributes(input.clone());

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

    // Extract type information
    let (_input_type, _output_type) = extract_function_types(&input);

    // Generate the expanded code
    let expanded = if !path_params.is_empty() || !body_params.is_empty() {
        // Generate path extractor wrapper
        let wrapper_name = format!("{}_wrapper", fn_name_str);
        let wrapper_ident = Ident::new(&wrapper_name, fn_name.span());

        // Generate parameter declarations for wrapper signature
        let wrapper_params: Vec<_> = path_params.iter()
            .map(|p| {
                let name = &p.name;
                let typ = &p.typ;
                quote! { #name: #typ }
            })
            .chain(body_params.iter().map(|b| {
                let name = &b.name;
                let typ = &b.typ;
                quote! { #name: #typ }
            }))
            .collect();

        // Generate parameter names for extraction and call
        let param_names: Vec<_> = path_params.iter()
            .map(|p| &p.name)
            .chain(body_params.iter().map(|b| &b.name))
            .collect();

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

        // Generate extractors based on what parameters are present
        let (extractor_code, all_params) = if !path_params.is_empty() && !body_params.is_empty() {
            // Both path and body parameters
            let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
            let body_names: Vec<_> = body_params.iter().map(|b| &b.name).collect();
            let code = quote! {
                let path = axum::extract::Path((#(#path_names),*));
                let (#(#path_names),*) = path.0;
                let json = axum::Json((#(#body_names),*));
                let (#(#body_names),*) = json.0;
            };
            (code, param_names)
        } else if !path_params.is_empty() {
            // Only path parameters
            let code = quote! {
                let path = axum::extract::Path((#(#param_names),*));
                let (#(#param_names),*) = path.0;
            };
            (code, param_names)
        } else {
            // Only body parameters
            let code = quote! {
                let json = axum::Json((#(#param_names),*));
                let (#(#param_names),*) = json.0;
            };
            (code, param_names)
        };

        quote! {
            // Original function (stripped of attributes)
            #original_fn

            // Extractor wrapper
            #vis async fn #wrapper_ident(
                #(#wrapper_params),*
            ) #return_type {
                #extractor_code
                #fn_name(#(#all_params),*).await
            }

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: #route_type,
            });
        }
    } else {
        quote! {
            #original_fn

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: #route_type,
            });
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn post(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = input.sig.ident.clone();
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;

    // Parse route arguments to extract #[path] and #[body] metadata
    let route_args = parse_route_args(&input.sig.inputs);

    // Filter for path parameters
    let path_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Path(p) => Some(p),
            _ => None,
        })
        .collect();

    // Filter for body parameters
    let body_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Body(b) => Some(b),
            _ => None,
        })
        .collect();

    // Strip #[path] and #[body] attributes from function parameters
    let original_fn = strip_arg_attributes(input.clone());

    // Extract type information
    let (_input_type, _output_type) = extract_function_types(&input);

    // Generate the expanded code
    let expanded = if !path_params.is_empty() || !body_params.is_empty() {
        // Generate path extractor wrapper
        let wrapper_name = format!("{}_wrapper", fn_name_str);
        let wrapper_ident = Ident::new(&wrapper_name, fn_name.span());

        // Generate parameter declarations for wrapper signature
        let wrapper_params: Vec<_> = path_params.iter()
            .map(|p| {
                let name = &p.name;
                let typ = &p.typ;
                quote! { #name: #typ }
            })
            .chain(body_params.iter().map(|b| {
                let name = &b.name;
                let typ = &b.typ;
                quote! { #name: #typ }
            }))
            .collect();

        // Generate parameter names for extraction and call
        let param_names: Vec<_> = path_params.iter()
            .map(|p| &p.name)
            .chain(body_params.iter().map(|b| &b.name))
            .collect();

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

        // Generate extractors based on what parameters are present
        let (extractor_code, all_params) = if !path_params.is_empty() && !body_params.is_empty() {
            // Both path and body parameters
            let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
            let body_names: Vec<_> = body_params.iter().map(|b| &b.name).collect();
            let code = quote! {
                let path = axum::extract::Path((#(#path_names),*));
                let (#(#path_names),*) = path.0;
                let json = axum::Json((#(#body_names),*));
                let (#(#body_names),*) = json.0;
            };
            (code, param_names)
        } else if !path_params.is_empty() {
            // Only path parameters
            let code = quote! {
                let path = axum::extract::Path((#(#param_names),*));
                let (#(#param_names),*) = path.0;
            };
            (code, param_names)
        } else {
            // Only body parameters
            let code = quote! {
                let json = axum::Json((#(#param_names),*));
                let (#(#param_names),*) = json.0;
            };
            (code, param_names)
        };

        quote! {
            // Original function (stripped of attributes)
            #original_fn

            // Extractor wrapper
            #vis async fn #wrapper_ident(
                #(#wrapper_params),*
            ) #return_type {
                #extractor_code
                #fn_name(#(#all_params),*).await
            }

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: virust_runtime::RouteType::HttpPost,
            });
        }
    } else {
        quote! {
            #original_fn

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: virust_runtime::RouteType::HttpPost,
            });
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn put(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = input.sig.ident.clone();
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;

    // Parse route arguments to extract #[path] and #[body] metadata
    let route_args = parse_route_args(&input.sig.inputs);

    // Filter for path parameters
    let path_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Path(p) => Some(p),
            _ => None,
        })
        .collect();

    // Filter for body parameters
    let body_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Body(b) => Some(b),
            _ => None,
        })
        .collect();

    // Strip #[path] and #[body] attributes from function parameters
    let original_fn = strip_arg_attributes(input.clone());

    // Extract type information
    let (_input_type, _output_type) = extract_function_types(&input);

    // Generate the expanded code
    let expanded = if !path_params.is_empty() || !body_params.is_empty() {
        // Generate path extractor wrapper
        let wrapper_name = format!("{}_wrapper", fn_name_str);
        let wrapper_ident = Ident::new(&wrapper_name, fn_name.span());

        // Generate parameter declarations for wrapper signature
        let wrapper_params: Vec<_> = path_params.iter()
            .map(|p| {
                let name = &p.name;
                let typ = &p.typ;
                quote! { #name: #typ }
            })
            .chain(body_params.iter().map(|b| {
                let name = &b.name;
                let typ = &b.typ;
                quote! { #name: #typ }
            }))
            .collect();

        // Generate parameter names for extraction and call
        let param_names: Vec<_> = path_params.iter()
            .map(|p| &p.name)
            .chain(body_params.iter().map(|b| &b.name))
            .collect();

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

        // Generate extractors based on what parameters are present
        let (extractor_code, all_params) = if !path_params.is_empty() && !body_params.is_empty() {
            // Both path and body parameters
            let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
            let body_names: Vec<_> = body_params.iter().map(|b| &b.name).collect();
            let code = quote! {
                let path = axum::extract::Path((#(#path_names),*));
                let (#(#path_names),*) = path.0;
                let json = axum::Json((#(#body_names),*));
                let (#(#body_names),*) = json.0;
            };
            (code, param_names)
        } else if !path_params.is_empty() {
            // Only path parameters
            let code = quote! {
                let path = axum::extract::Path((#(#param_names),*));
                let (#(#param_names),*) = path.0;
            };
            (code, param_names)
        } else {
            // Only body parameters
            let code = quote! {
                let json = axum::Json((#(#param_names),*));
                let (#(#param_names),*) = json.0;
            };
            (code, param_names)
        };

        quote! {
            // Original function (stripped of attributes)
            #original_fn

            // Extractor wrapper
            #vis async fn #wrapper_ident(
                #(#wrapper_params),*
            ) #return_type {
                #extractor_code
                #fn_name(#(#all_params),*).await
            }

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: virust_runtime::RouteType::HttpPut,
            });
        }
    } else {
        quote! {
            #original_fn

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: virust_runtime::RouteType::HttpPut,
            });
        }
    };

    TokenStream::from(expanded)
}

#[proc_macro_attribute]
pub fn delete(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);
    let fn_name = input.sig.ident.clone();
    let fn_name_str = fn_name.to_string();
    let vis = &input.vis;

    // Parse route arguments to extract #[path] and #[body] metadata
    let route_args = parse_route_args(&input.sig.inputs);

    // Filter for path parameters
    let path_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Path(p) => Some(p),
            _ => None,
        })
        .collect();

    // Filter for body parameters
    let body_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Body(b) => Some(b),
            _ => None,
        })
        .collect();

    // Strip #[path] and #[body] attributes from function parameters
    let original_fn = strip_arg_attributes(input.clone());

    // Extract type information
    let (_input_type, _output_type) = extract_function_types(&input);

    // Generate the expanded code
    let expanded = if !path_params.is_empty() || !body_params.is_empty() {
        // Generate path extractor wrapper
        let wrapper_name = format!("{}_wrapper", fn_name_str);
        let wrapper_ident = Ident::new(&wrapper_name, fn_name.span());

        // Generate parameter declarations for wrapper signature
        let wrapper_params: Vec<_> = path_params.iter()
            .map(|p| {
                let name = &p.name;
                let typ = &p.typ;
                quote! { #name: #typ }
            })
            .chain(body_params.iter().map(|b| {
                let name = &b.name;
                let typ = &b.typ;
                quote! { #name: #typ }
            }))
            .collect();

        // Generate parameter names for extraction and call
        let param_names: Vec<_> = path_params.iter()
            .map(|p| &p.name)
            .chain(body_params.iter().map(|b| &b.name))
            .collect();

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

        // Generate extractors based on what parameters are present
        let (extractor_code, all_params) = if !path_params.is_empty() && !body_params.is_empty() {
            // Both path and body parameters
            let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
            let body_names: Vec<_> = body_params.iter().map(|b| &b.name).collect();
            let code = quote! {
                let path = axum::extract::Path((#(#path_names),*));
                let (#(#path_names),*) = path.0;
                let json = axum::Json((#(#body_names),*));
                let (#(#body_names),*) = json.0;
            };
            (code, param_names)
        } else if !path_params.is_empty() {
            // Only path parameters
            let code = quote! {
                let path = axum::extract::Path((#(#param_names),*));
                let (#(#param_names),*) = path.0;
            };
            (code, param_names)
        } else {
            // Only body parameters
            let code = quote! {
                let json = axum::Json((#(#param_names),*));
                let (#(#param_names),*) = json.0;
            };
            (code, param_names)
        };

        quote! {
            // Original function (stripped of attributes)
            #original_fn

            // Extractor wrapper
            #vis async fn #wrapper_ident(
                #(#wrapper_params),*
            ) #return_type {
                #extractor_code
                #fn_name(#(#all_params),*).await
            }

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: virust_runtime::RouteType::HttpDelete,
            });
        }
    } else {
        quote! {
            #original_fn

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: virust_runtime::RouteType::HttpDelete,
            });
        }
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

/// Marks a function parameter as a path parameter
///
/// This is a helper attribute that can be used on function parameters.
/// Currently it serves as a marker for documentation purposes.
///
/// # Example
///
/// ```rust,no_run
/// # use virust_macros::get;
/// #[get]
/// async fn get_user(id: String) -> String {
///     format!("User ID: {}", id)
/// }
/// ```
///
/// Note: This attribute is a placeholder for future implementation.
/// The actual path parameter extraction will be handled by the runtime.
#[proc_macro_attribute]
pub fn path(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just return the item unchanged
    // In a full implementation, this would parse the function
    // and extract parameter metadata
    item
}

/// Marks a function parameter as a JSON body parameter
///
/// This is a helper attribute that can be used on function parameters.
/// Currently it serves as a marker for documentation purposes.
///
/// # Example
///
/// ```rust,no_run
/// # use virust_macros::post;
/// #[post]
/// async fn create_user(user: String) -> String {
///     format!("Created user: {}", user)
/// }
/// ```
///
/// Note: This attribute is a placeholder for future implementation.
/// The actual body parameter extraction will be handled by the runtime.
#[proc_macro_attribute]
pub fn body(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just return the item unchanged
    // In a full implementation, this would parse the function
    // and extract parameter metadata
    item
}
