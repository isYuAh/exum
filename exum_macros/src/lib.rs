use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::quote;
use syn::{parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated, Block, Expr, ExprLit, FnArg, Ident, ItemFn, Lit, LitStr, Meta, MetaNameValue, Pat, Signature, Token};

fn method_to_ident(method: &str) -> syn::Ident {
    syn::Ident::new(&method.to_uppercase(), Span::call_site())
}

fn collect_methods(expr: Expr) -> Vec<String> {
    match expr {
        Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) => s.value().split(',').map(|s| s.to_uppercase()).collect(),
        Expr::Array(arr) => arr
            .elems
            .iter()
            .map(|e| {
                if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = e {
                    s.value().to_uppercase()
                } else {
                    panic!("Array element must be a string literal")
                }
            })
            .collect(),
        _ => panic!("Expression must be a string literal or an array of string literals"),
    }
}

fn extract_params(path: &str) -> Vec<String> {
    path.split('/')
        .filter_map(|s| {
            if s.starts_with('{') && s.ends_with('}') {
                Some(s.trim_start_matches('{').trim_end_matches('}').to_string())
            } else {
                None
            }
        })
        .collect()
}

fn normalize_path(path: &str) -> String {
    if path.is_empty() {
        return "/".to_string();
    }

    if path == "/" {
        return "/".to_string();
    }

    let mut out = String::new();
    for seg in path.split('/') {
        if seg.is_empty() {
            continue;
        }
        out.push('/');

        if seg.starts_with(':') {
            let name = seg[1..].trim();
            if name.is_empty() {
                out.push_str(seg);
            } else {
                out.push('{');
                out.push_str(name);
                out.push('}');
            }
        } else if seg.starts_with('{') && seg.ends_with('}') {
            out.push_str(seg);
        } else {
            out.push_str(seg);
        }
    }

    if out.is_empty() {
        out.push('/');
    }
    out
}

fn extract_path(args: &Punctuated<Meta, Token![,]>) -> String {
    let mut path = "/".to_string();
    for meta in args {
        if let Meta::NameValue(MetaNameValue {path: path_meta, value, ..}) = meta {
            if path_meta.is_ident("path") {
                if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                    path = s.value();
                }
            }
        }
    }
    normalize_path(&path)
}

fn extract_methods(args: &Punctuated<Meta, Token![,]>) -> Vec<String> {
    let mut methods = Vec::new();
    for meta in args {
        if let Meta::NameValue(MetaNameValue {path: path_meta, value, ..}) = meta {
            if path_meta.is_ident("method") {
                methods.extend(collect_methods(value.clone()));
            }
        }
    }
    if methods.is_empty() {
        methods.push("POST".to_string());
    }
    methods
}
fn parse_args(args: TokenStream) -> Punctuated<Meta, Token![,]> {
    let attr_ts2: proc_macro2::TokenStream = args.into();
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    parser.parse2(attr_ts2).unwrap()
}
fn process_inputs(inputs: &Punctuated<FnArg, Token![,]>, path: &str) 
    -> (Option<FnArg>, Vec<FnArg>)
{
    let params = extract_params(path);
    let mut path_idents = Vec::new();
    let mut path_types = Vec::new();
    let mut other_inputs = Vec::new();

    for input in inputs {
        if let FnArg::Typed(pat_type) = input {
            let has_q_attr = pat_type.attrs.iter().any(|a| a.path().is_ident("q"));
            if has_q_attr {
                if let Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    let var_ident = pat_ident.ident.clone();
                    let ty = *pat_type.ty.clone();
                    let new_input: FnArg =
                        syn::parse_quote! { axum::extract::Query(#var_ident): axum::extract::Query<#ty> };
                    other_inputs.push(new_input);
                    continue;
                }
            }
            if let Pat::Ident(ident) = &*pat_type.pat {
                let name = ident.ident.to_string();
                if params.contains(&name) {
                    path_idents.push(ident.ident.clone());
                    path_types.push(&pat_type.ty);
                    continue;
                }
            }
            other_inputs.push(input.clone());
        }
    }

    let path_arg: Option<FnArg> = if !path_idents.is_empty() {
        Some(parse_quote! {
            axum::extract::Path((#(#path_idents),*)): axum::extract::Path<(#(#path_types),*)>
        })
    } else {
        None
    };

    (path_arg, other_inputs)
}
fn build_signature(
    fn_name: &Ident,
    path_arg: Option<FnArg>,
    mut other_inputs: Vec<FnArg>
) -> Signature {
    let mut new_inputs = Vec::new();
    if let Some(p) = path_arg {
        new_inputs.push(p);
    }
    new_inputs.append(&mut other_inputs);

    let mut new_sig: Signature = parse_quote! {
        async fn #fn_name(#(#new_inputs),*) -> impl axum::response::IntoResponse
    };
    new_sig.output = parse_quote!(-> impl axum::response::IntoResponse);
    new_sig
}
fn build_router_expr(methods: &[String], path: &str, fn_name: &Ident) -> proc_macro2::TokenStream {
    let path_lit = LitStr::new(path, Span::call_site());
    let mut router_expr = quote! { router };
    for m in methods {
        let method_ident = method_to_ident(m);
        router_expr = quote! {
            #router_expr.route(#path_lit, axum::routing::on(axum::routing::MethodFilter::#method_ident, #fn_name))
        };
    }
    router_expr
}
fn expand(
    new_sig: Signature,
    block: Box<Block>,
    router_expr: proc_macro2::TokenStream
) -> TokenStream {
    let expanded = quote! {
        #new_sig {
            #block
        }

        inventory::submit! {
            exum::RouteDef {
                router: |router| #router_expr,
            }
        }
    };
    TokenStream::from(expanded)
}



#[proc_macro_attribute]
pub fn route(args: TokenStream, item: TokenStream) -> TokenStream {
    let attr_ts2: proc_macro2::TokenStream = args.clone().into();
    let parser = Punctuated::<Meta, Token![,]>::parse_terminated;
    let args = parser.parse2(attr_ts2).unwrap();
    let input_fn = parse_macro_input!(item as ItemFn);
    let mut method: Vec<String> = Vec::new();
    let mut path = String::from("/");

    for meta in args {
        if let Meta::NameValue(MetaNameValue {path: path_meta, value, ..}) = meta {
            if path_meta.is_ident("path") {
                if let Expr::Lit(ExprLit { lit: Lit::Str(s), .. }) = value {
                    path = s.value();
                }
            } else if path_meta.is_ident("method") {
                method.extend(collect_methods(value));
            } else {
                panic!("Unknow attribute in #[route(...)]");
            }
        } else {
            panic!("unexpected attribute format in #[route(...)]");
        }
    }

    if method.is_empty() { method.push("POST".to_string()) };
    path = normalize_path(&path);
    let path_lit = LitStr::new(&path, Span::call_site());
    let fn_name = &input_fn.sig.ident;
    let params = extract_params(&path);
    let mut path_idents = Vec::new();
    let mut path_types = Vec::new();
    let mut other_inputs = Vec::new();

    for input in &input_fn.sig.inputs {
        if let FnArg::Typed(pat_type) = input {
            let has_q_attr = pat_type.attrs.iter().any(|a| a.path().is_ident("q"));
            if has_q_attr {
                if let Pat::Ident(pat_ident) = *pat_type.pat.clone() {
                    let var_ident = pat_ident.ident.clone();
                    let ty = *pat_type.ty.clone();
                    let new_input: FnArg =
                        syn::parse_quote! { axum::extract::Query(#var_ident): axum::extract::Query<#ty> };
                    other_inputs.push(new_input);
                    continue;
                } else {
                    continue;
                }
            }
            if let Pat::Ident(ident) = &*pat_type.pat {
                let name = ident.ident.to_string();
                if params.contains(&name) {
                    path_idents.push(ident.ident.clone());
                    path_types.push(&pat_type.ty);
                } else {
                    other_inputs.push(input.clone());
                }
            }else {
                other_inputs.push(input.clone());
            }
        }
    }

    let path_arg: Option<FnArg> = if !path_idents.is_empty() {
        Some(parse_quote! {
            axum::extract::Path((#(#path_idents),*)): axum::extract::Path<(#(#path_types),*)>
        })
    } else {
        None
    };
    let mut new_inputs = Vec::new();
    if let Some(p) = path_arg {
        new_inputs.push(p);
    }
    new_inputs.extend(other_inputs);
    let mut new_sig: Signature = parse_quote! {
        async fn #fn_name(#(#new_inputs),*) -> impl axum::response::IntoResponse
    };

    let mut router_expr = quote! { router };
    for m in &method {
        let method_ident = method_to_ident(m);
        router_expr = quote! {
            #router_expr.route(#path_lit, axum::routing::on(axum::routing::MethodFilter::#method_ident, #fn_name))
        };
    }
    new_sig.output = parse_quote!(-> impl axum::response::IntoResponse);
    let block = &input_fn.block;

    let expanded = quote! {
        #new_sig {
            #block
        }

        inventory::submit! {
            exum::RouteDef {
                router: |router| #router_expr,
            }
        }
    };
    TokenStream::from(expanded)
}

fn make_wrapper(attr: TokenStream, item: TokenStream, method: &str) -> TokenStream {
    let lit = parse_macro_input!(attr as LitStr);

    let new_attr = format!("path = \"{}\", method = \"{}\"", lit.value(), method);
    let new_attr_ts: TokenStream = new_attr.parse().unwrap();

    route(new_attr_ts, item)
}

#[proc_macro_attribute]
pub fn get(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "GET")
}
#[proc_macro_attribute]
pub fn post(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "POST")
}
#[proc_macro_attribute]
pub fn put(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "PUT")
}
#[proc_macro_attribute]
pub fn delete(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "DELETE")
}
#[proc_macro_attribute]
pub fn options(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "OPTIONS")
}
#[proc_macro_attribute]
pub fn head(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "HEAD")
}
#[proc_macro_attribute]
pub fn trace(attr: TokenStream, item: TokenStream) -> TokenStream {
    make_wrapper(attr, item, "TRACE")
}