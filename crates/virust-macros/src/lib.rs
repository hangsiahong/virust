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

/// Parse route arguments from function parameters, extracting #[param] and #[body] attributes
fn parse_route_args(args: &Punctuated<FnArg, Comma>) -> Vec<RouteArg> {
    args.iter()
        .filter_map(|arg| {
            if let FnArg::Typed(pat) = arg {
                // Check for #[param] attribute (renamed from #[path] to avoid conflict with Rust builtin)
                let is_path = pat.attrs.iter().any(|attr| {
                    attr.path().is_ident("param")
                });

                // Check for #[body] attribute
                let is_body = pat.attrs.iter().any(|attr| {
                    attr.path().is_ident("body")
                });

                // Extract the identifier name from the pattern
                // Handle both simple patterns (id) and complex patterns (Json(id))
                let ident = match &*pat.pat {
                    Pat::Ident(ident) => Some(ident.ident.clone()),
                    Pat::Type(pat_type) => {
                        // Handle Type patterns like Json(id)
                        if let Pat::Ident(inner) = &*pat_type.pat {
                            Some(inner.ident.clone())
                        } else {
                            None
                        }
                    }
                    _ => None,
                };

                if let Some(name) = ident {
                    if is_path {
                        Some(RouteArg::Path(PathArg {
                            name: name.clone(),
                            typ: (*pat.ty).clone(),
                        }))
                    } else if is_body {
                        Some(RouteArg::Body(BodyArg {
                            name: name.clone(),
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

/// Strip #[param] and #[body] attributes from function parameters
fn strip_arg_attributes(mut input: ItemFn) -> ItemFn {
    for arg in input.sig.inputs.iter_mut() {
        if let FnArg::Typed(pat) = arg {
            pat.attrs.retain(|attr| {
                // Remove #[param] and #[body] attributes
                !attr.path().is_ident("param") && !attr.path().is_ident("body")
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

    // Parse route arguments to extract #[body] metadata
    let route_args = parse_route_args(&input.sig.inputs);

    // Filter for body parameters
    let body_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Body(b) => Some(b),
            _ => None,
        })
        .collect();

    // Validate: only one body parameter allowed
    if body_params.len() > 1 {
        panic!(
            "Only one #[body] parameter is allowed per function, found {}",
            body_params.len()
        );
    }

    // Strip #[body] attributes from function parameters
    let original_fn = strip_arg_attributes(input.clone());

    // Extract type information
    let (_input_type, _output_type) = extract_function_types(&input);

    // Generate the expanded code
    let expanded = if !body_params.is_empty() {
        // Generate body extractor wrapper for WebSocket
        let wrapper_name = format!("{}_wrapper", _fn_name_str);
        let wrapper_ident = Ident::new(&wrapper_name, fn_name.span());

        let body_param = &body_params[0];
        let body_name = &body_param.name;
        let body_typ = &body_param.typ;

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

        let vis = &input.vis;

        quote! {
            // Original function (stripped of attributes)
            #original_fn

            // Extractor wrapper for WebSocket
            #vis async fn #wrapper_ident(
                #body_name: #body_typ
            ) #return_type {
                #fn_name(#body_name).await
            }

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: virust_runtime::RouteType::WebSocket,
            });
        }
    } else {
        quote! {
            #input

            inventory::submit!(virust_runtime::RouteEntry {
                path: stringify!(#fn_name),
                route_type: virust_runtime::RouteType::WebSocket,
            });
        }
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

    // Validate: only one body parameter allowed
    if body_params.len() > 1 {
        panic!(
            "Only one #[body] parameter is allowed per function, found {}",
            body_params.len()
        );
    }

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
        // For Axum handlers:
        // - Multiple path params must be combined into single Path<(T1, T2, ...)>
        // - Single path param uses Path<T>
        // - Body params use Json<T>
        let (wrapper_params, extractor_code, all_params): (Vec<_>, _, Vec<_>) = if !path_params.is_empty() && !body_params.is_empty() {
            // Both path and body parameters
            if path_params.len() == 1 {
                // Single path param
                let path_param = &path_params[0];
                let path_name = &path_param.name;
                let path_typ = &path_param.typ;
                let body_param = &body_params[0];
                let body_name = &body_param.name;
                let body_typ = &body_param.typ;
                let params = vec![
                    quote! { #path_name: axum::extract::Path<#path_typ> },
                    quote! { #body_name: axum::Json<#body_typ> }
                ];
                let extract = quote! {
                    let #path_name = #path_name.0;
                    let #body_name = #body_name.0;
                };
                let all = vec![path_name.clone(), body_name.clone()];
                (params, extract, all)
            } else {
                // Multiple path params - combine into single tuple extractor
                let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
                let path_types: Vec<_> = path_params.iter().map(|p| &p.typ).collect();
                let path_tuple_type = quote! { (#(#path_types),*) };
                let body_param = &body_params[0];
                let body_name = &body_param.name;
                let body_typ = &body_param.typ;
                let params = vec![
                    quote! { _path: axum::extract::Path<#path_tuple_type> },
                    quote! { #body_name: axum::Json<#body_typ> }
                ];
                let extract = quote! {
                    let (#(#path_names),*) = _path.0;
                    let #body_name = #body_name.0;
                };
                let all: Vec<_> = path_params.iter().map(|p| p.name.clone())
                    .chain(std::iter::once(body_name.clone()))
                    .collect();
                (params, extract, all)
            }
        } else if !path_params.is_empty() {
            // Only path parameters
            if path_params.len() == 1 {
                // Single path param
                let path_param = &path_params[0];
                let path_name = &path_param.name;
                let path_typ = &path_param.typ;
                let params = vec![quote! { #path_name: axum::extract::Path<#path_typ> }];
                let extract = quote! { let #path_name = #path_name.0; };
                let all = vec![path_name.clone()];
                (params, extract, all)
            } else {
                // Multiple path params - combine into single tuple extractor
                let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
                let path_types: Vec<_> = path_params.iter().map(|p| &p.typ).collect();
                let path_tuple_type = quote! { (#(#path_types),*) };
                let params = vec![quote! { _path: axum::extract::Path<#path_tuple_type> }];
                let extract = quote! { let (#(#path_names),*) = _path.0; };
                let all = path_params.iter().map(|p| p.name.clone()).collect();
                (params, extract, all)
            }
        } else {
            // Only body parameters (single body param guaranteed by validation)
            let body_param = &body_params[0];
            let body_name = &body_param.name;
            let body_typ = &body_param.typ;
            let params = vec![quote! { #body_name: axum::Json<#body_typ> }];
            let extract = quote! { let #body_name = #body_name.0; };
            let all = vec![body_name.clone()];
            (params, extract, all)
        };

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

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

    // Validate: only one body parameter allowed
    if body_params.len() > 1 {
        panic!(
            "Only one #[body] parameter is allowed per function, found {}",
            body_params.len()
        );
    }

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
        // For Axum handlers:
        // - Multiple path params must be combined into single Path<(T1, T2, ...)>
        // - Single path param uses Path<T>
        // - Body params use Json<T>
        let (wrapper_params, extractor_code, all_params): (Vec<_>, _, Vec<_>) = if !path_params.is_empty() && !body_params.is_empty() {
            // Both path and body parameters
            if path_params.len() == 1 {
                // Single path param
                let path_param = &path_params[0];
                let path_name = &path_param.name;
                let path_typ = &path_param.typ;
                let body_param = &body_params[0];
                let body_name = &body_param.name;
                let body_typ = &body_param.typ;
                let params = vec![
                    quote! { #path_name: axum::extract::Path<#path_typ> },
                    quote! { #body_name: axum::Json<#body_typ> }
                ];
                let extract = quote! {
                    let #path_name = #path_name.0;
                    let #body_name = #body_name.0;
                };
                let all = vec![path_name.clone(), body_name.clone()];
                (params, extract, all)
            } else {
                // Multiple path params - combine into single tuple extractor
                let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
                let path_types: Vec<_> = path_params.iter().map(|p| &p.typ).collect();
                let path_tuple_type = quote! { (#(#path_types),*) };
                let body_param = &body_params[0];
                let body_name = &body_param.name;
                let body_typ = &body_param.typ;
                let params = vec![
                    quote! { _path: axum::extract::Path<#path_tuple_type> },
                    quote! { #body_name: axum::Json<#body_typ> }
                ];
                let extract = quote! {
                    let (#(#path_names),*) = _path.0;
                    let #body_name = #body_name.0;
                };
                let all: Vec<_> = path_params.iter().map(|p| p.name.clone())
                    .chain(std::iter::once(body_name.clone()))
                    .collect();
                (params, extract, all)
            }
        } else if !path_params.is_empty() {
            // Only path parameters
            if path_params.len() == 1 {
                // Single path param
                let path_param = &path_params[0];
                let path_name = &path_param.name;
                let path_typ = &path_param.typ;
                let params = vec![quote! { #path_name: axum::extract::Path<#path_typ> }];
                let extract = quote! { let #path_name = #path_name.0; };
                let all = vec![path_name.clone()];
                (params, extract, all)
            } else {
                // Multiple path params - combine into single tuple extractor
                let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
                let path_types: Vec<_> = path_params.iter().map(|p| &p.typ).collect();
                let path_tuple_type = quote! { (#(#path_types),*) };
                let params = vec![quote! { _path: axum::extract::Path<#path_tuple_type> }];
                let extract = quote! { let (#(#path_names),*) = _path.0; };
                let all = path_params.iter().map(|p| p.name.clone()).collect();
                (params, extract, all)
            }
        } else {
            // Only body parameters (single body param guaranteed by validation)
            let body_param = &body_params[0];
            let body_name = &body_param.name;
            let body_typ = &body_param.typ;
            let params = vec![quote! { #body_name: axum::Json<#body_typ> }];
            let extract = quote! { let #body_name = #body_name.0; };
            let all = vec![body_name.clone()];
            (params, extract, all)
        };

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

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

    // Validate: only one body parameter allowed
    if body_params.len() > 1 {
        panic!(
            "Only one #[body] parameter is allowed per function, found {}",
            body_params.len()
        );
    }

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
        // For Axum handlers:
        // - Multiple path params must be combined into single Path<(T1, T2, ...)>
        // - Single path param uses Path<T>
        // - Body params use Json<T>
        let (wrapper_params, extractor_code, all_params): (Vec<_>, _, Vec<_>) = if !path_params.is_empty() && !body_params.is_empty() {
            // Both path and body parameters
            if path_params.len() == 1 {
                // Single path param
                let path_param = &path_params[0];
                let path_name = &path_param.name;
                let path_typ = &path_param.typ;
                let body_param = &body_params[0];
                let body_name = &body_param.name;
                let body_typ = &body_param.typ;
                let params = vec![
                    quote! { #path_name: axum::extract::Path<#path_typ> },
                    quote! { #body_name: axum::Json<#body_typ> }
                ];
                let extract = quote! {
                    let #path_name = #path_name.0;
                    let #body_name = #body_name.0;
                };
                let all = vec![path_name.clone(), body_name.clone()];
                (params, extract, all)
            } else {
                // Multiple path params - combine into single tuple extractor
                let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
                let path_types: Vec<_> = path_params.iter().map(|p| &p.typ).collect();
                let path_tuple_type = quote! { (#(#path_types),*) };
                let body_param = &body_params[0];
                let body_name = &body_param.name;
                let body_typ = &body_param.typ;
                let params = vec![
                    quote! { _path: axum::extract::Path<#path_tuple_type> },
                    quote! { #body_name: axum::Json<#body_typ> }
                ];
                let extract = quote! {
                    let (#(#path_names),*) = _path.0;
                    let #body_name = #body_name.0;
                };
                let all: Vec<_> = path_params.iter().map(|p| p.name.clone())
                    .chain(std::iter::once(body_name.clone()))
                    .collect();
                (params, extract, all)
            }
        } else if !path_params.is_empty() {
            // Only path parameters
            if path_params.len() == 1 {
                // Single path param
                let path_param = &path_params[0];
                let path_name = &path_param.name;
                let path_typ = &path_param.typ;
                let params = vec![quote! { #path_name: axum::extract::Path<#path_typ> }];
                let extract = quote! { let #path_name = #path_name.0; };
                let all = vec![path_name.clone()];
                (params, extract, all)
            } else {
                // Multiple path params - combine into single tuple extractor
                let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
                let path_types: Vec<_> = path_params.iter().map(|p| &p.typ).collect();
                let path_tuple_type = quote! { (#(#path_types),*) };
                let params = vec![quote! { _path: axum::extract::Path<#path_tuple_type> }];
                let extract = quote! { let (#(#path_names),*) = _path.0; };
                let all = path_params.iter().map(|p| p.name.clone()).collect();
                (params, extract, all)
            }
        } else {
            // Only body parameters (single body param guaranteed by validation)
            let body_param = &body_params[0];
            let body_name = &body_param.name;
            let body_typ = &body_param.typ;
            let params = vec![quote! { #body_name: axum::Json<#body_typ> }];
            let extract = quote! { let #body_name = #body_name.0; };
            let all = vec![body_name.clone()];
            (params, extract, all)
        };

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

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

    // Validate: only one body parameter allowed
    if body_params.len() > 1 {
        panic!(
            "Only one #[body] parameter is allowed per function, found {}",
            body_params.len()
        );
    }

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
        // For Axum handlers:
        // - Multiple path params must be combined into single Path<(T1, T2, ...)>
        // - Single path param uses Path<T>
        // - Body params use Json<T>
        let (wrapper_params, extractor_code, all_params): (Vec<_>, _, Vec<_>) = if !path_params.is_empty() && !body_params.is_empty() {
            // Both path and body parameters
            if path_params.len() == 1 {
                // Single path param
                let path_param = &path_params[0];
                let path_name = &path_param.name;
                let path_typ = &path_param.typ;
                let body_param = &body_params[0];
                let body_name = &body_param.name;
                let body_typ = &body_param.typ;
                let params = vec![
                    quote! { #path_name: axum::extract::Path<#path_typ> },
                    quote! { #body_name: axum::Json<#body_typ> }
                ];
                let extract = quote! {
                    let #path_name = #path_name.0;
                    let #body_name = #body_name.0;
                };
                let all = vec![path_name.clone(), body_name.clone()];
                (params, extract, all)
            } else {
                // Multiple path params - combine into single tuple extractor
                let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
                let path_types: Vec<_> = path_params.iter().map(|p| &p.typ).collect();
                let path_tuple_type = quote! { (#(#path_types),*) };
                let body_param = &body_params[0];
                let body_name = &body_param.name;
                let body_typ = &body_param.typ;
                let params = vec![
                    quote! { _path: axum::extract::Path<#path_tuple_type> },
                    quote! { #body_name: axum::Json<#body_typ> }
                ];
                let extract = quote! {
                    let (#(#path_names),*) = _path.0;
                    let #body_name = #body_name.0;
                };
                let all: Vec<_> = path_params.iter().map(|p| p.name.clone())
                    .chain(std::iter::once(body_name.clone()))
                    .collect();
                (params, extract, all)
            }
        } else if !path_params.is_empty() {
            // Only path parameters
            if path_params.len() == 1 {
                // Single path param
                let path_param = &path_params[0];
                let path_name = &path_param.name;
                let path_typ = &path_param.typ;
                let params = vec![quote! { #path_name: axum::extract::Path<#path_typ> }];
                let extract = quote! { let #path_name = #path_name.0; };
                let all = vec![path_name.clone()];
                (params, extract, all)
            } else {
                // Multiple path params - combine into single tuple extractor
                let path_names: Vec<_> = path_params.iter().map(|p| &p.name).collect();
                let path_types: Vec<_> = path_params.iter().map(|p| &p.typ).collect();
                let path_tuple_type = quote! { (#(#path_types),*) };
                let params = vec![quote! { _path: axum::extract::Path<#path_tuple_type> }];
                let extract = quote! { let (#(#path_names),*) = _path.0; };
                let all = path_params.iter().map(|p| p.name.clone()).collect();
                (params, extract, all)
            }
        } else {
            // Only body parameters (single body param guaranteed by validation)
            let body_param = &body_params[0];
            let body_name = &body_param.name;
            let body_typ = &body_param.typ;
            let params = vec![quote! { #body_name: axum::Json<#body_typ> }];
            let extract = quote! { let #body_name = #body_name.0; };
            let all = vec![body_name.clone()];
            (params, extract, all)
        };

        // Get the return type from the original function
        let return_type = &original_fn.sig.output;

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

/// Marks a function parameter as a path parameter (LEGACY - use #[param] instead)
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

/// Marks a function parameter as a path parameter
///
/// Use this attribute to mark function parameters that should be extracted
/// from the URL path. This is the preferred attribute over #[path] which
/// conflicts with Rust's builtin module path attribute.
///
/// # Example
///
/// ```rust,no_run
/// # use virust_macros::{get, param};
/// #[get]
/// async fn get_user(#[param] id: String) -> String {
///     format!("User ID: {}", id)
/// }
/// ```
#[proc_macro_attribute]
pub fn param(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // For now, just return the item unchanged
    // The actual path parameter extraction is handled by the HTTP method macros
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

#[proc_macro_attribute]
pub fn render_component(attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as ItemFn);

    // Parse component name from attribute
    let attr_string = attr.to_string();
    let component_name = attr_string
        .trim_matches('"')
        .to_string();

    let fn_name = &input.sig.ident;
    let wrapper_name = format!("{}_wrapper", fn_name);
    let wrapper_ident = Ident::new(&wrapper_name, fn_name.span());

    // Strip #[path] and #[body] attributes from function parameters
    let original_fn = strip_arg_attributes(input.clone());

    // Extract parameters for props building
    let route_args = parse_route_args(&input.sig.inputs);

    // Filter for path parameters to build props
    let path_params: Vec<_> = route_args.iter()
        .filter_map(|arg| match arg {
            RouteArg::Path(p) => Some(p),
            _ => None,
        })
        .collect();

    // Generate parameter declarations for wrapper signature
    let wrapper_params: Vec<_> = path_params.iter()
        .map(|param| {
            let name = &param.name;
            let typ = &param.typ;
            quote! {
                #name: #typ
            }
        })
        .collect();

    // Generate parameter names for props building
    let param_names: Vec<_> = path_params.iter()
        .map(|param| &param.name)
        .collect();

    // Generate props building code from path parameters
    let props_building: Vec<_> = param_names.iter()
        .map(|name| {
            quote! {
                props[#name] = serde_json::to_value(#name.clone()).unwrap();
            }
        })
        .collect();

    // Generate wrapper function
    let expanded = quote! {
        #original_fn

        pub fn #wrapper_ident(#(#wrapper_params),*) -> impl axum::response::IntoResponse {
            use ::virust_runtime::RenderedHtml;

            let mut props = ::serde_json::json!({});
            #(#props_building)*

            RenderedHtml::with_props(#component_name, props)
        }
    };

    TokenStream::from(expanded)
}
