use percent_encoding::{utf8_percent_encode, NON_ALPHANUMERIC};
use proc_macro::{TokenStream};
use proc_macro2::Span;
use quote::{format_ident, quote};
use convert_case::{Case, Casing};
use syn::{ parse::Parser, parse_macro_input, parse_quote, punctuated::Punctuated, Block, Expr, ExprLit, FnArg, Ident, ItemFn, ItemImpl, ItemStruct, Lit, LitStr, Meta, MetaNameValue, Pat, Signature, Token};


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
                let inner = &s[1..s.len() - 1];
                let name = inner.trim_start_matches('*').trim_start_matches('*');
                Some(name.to_string())
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
        } else if seg.starts_with('*') {
            let name = &seg[1..];
            if name.starts_with('*') {
                out.push_str("{**");
                out.push_str(&name[1..]);
                out.push('}');
            } else {
                out.push_str("{*");
                out.push_str(name);
                out.push('}');
            }
        } else {
            out.push_str(&utf8_percent_encode(seg, NON_ALPHANUMERIC).to_string());
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

mod handle_input;
use handle_input::{handle_b_attr, handle_q_attr};

fn process_inputs(inputs: &Punctuated<FnArg, Token![,]>, path: &str, fn_name: &Ident) 
    -> (Option<FnArg>, Vec<FnArg>, Option<ItemStruct>, Vec<syn::Stmt>)
{
    let params = extract_params(path);
    let mut path_idents = Vec::new();
    let mut path_types = Vec::new();
    let mut other_inputs = Vec::new();
    let mut q_fields: Vec<syn::Field> = Vec::new();
    let mut inject_segs = Vec::new();

    for input in inputs {
        if let FnArg::Typed(pat_type) = input {
            let has_q_attr = pat_type.attrs.iter().any(|a| a.path().is_ident("q"));
            let has_b_attr = pat_type.attrs.iter().any(|a| a.path().is_ident("b"));
            let has_dep_attr = pat_type.attrs.iter().any(|a| a.path().is_ident("dep"));
            if has_q_attr {
                handle_q_attr(pat_type, &mut q_fields);
            } else if has_b_attr {
                handle_b_attr(pat_type, &mut other_inputs);
            } else if has_dep_attr {
                handle_dep_attr(pat_type, &mut inject_segs);
            } else if let Pat::Ident(ident) = &*pat_type.pat {
                let name = ident.ident.to_string();
                if params.contains(&name) {
                    path_idents.push(ident.ident.clone());
                    path_types.push(&pat_type.ty);
                    continue;
                } else {
                    other_inputs.push(input.clone());
                }
            } else {
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

    let q_struct = if !q_fields.is_empty() {
        let struct_ident = Ident::new(&format!("{}Query", fn_name.to_string().to_case(Case::Pascal)), Span::call_site());
        let q_struct: ItemStruct = parse_quote! {
            #[derive(serde::Deserialize)]
            struct #struct_ident {
                #(#q_fields),*
            }
        };
        let fields: Vec<Ident> = q_fields.iter().map(|f| f.ident.clone().unwrap()).collect();

        let query_arg: FnArg = parse_quote! {
            axum::extract::Query(#struct_ident { #(#fields),* }): axum::extract::Query<#struct_ident>
        };
        other_inputs.push(query_arg);
        Some(q_struct)
    } else {
        None
    };

    (path_arg, other_inputs, q_struct, inject_segs)
}
fn build_signature(
    path_arg: Option<FnArg>,
    mut other_inputs: Vec<FnArg>,
    original_sig: &Signature,
) -> Signature {
    let mut new_inputs = Vec::new();
    if let Some(p) = path_arg {
        new_inputs.push(p);
    }
    new_inputs.append(&mut other_inputs);

    let mut new_sig = original_sig.clone();
    new_sig.inputs.clear();
    for arg in new_inputs {
        new_sig.inputs.push(arg);
    }
    if matches!(new_sig.output, syn::ReturnType::Default) {
        new_sig.output = parse_quote!(-> impl axum::response::IntoResponse);
    }

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
    router_expr: proc_macro2::TokenStream,
    q_struct: Option<ItemStruct>,
    inject_segs: Vec<syn::Stmt>,

) -> TokenStream {
    let expanded = quote! {
        #q_struct
        #new_sig {
            #(#inject_segs),*
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
    let args = parse_args(args);
    let input_fn = parse_macro_input!(item as ItemFn);

    let path = extract_path(&args);
    let methods = extract_methods(&args);

    let (path_arg, other_inputs, q_struct, inject_segs) = process_inputs(&input_fn.sig.inputs, &path, &input_fn.sig.ident);
    let new_sig = build_signature(path_arg, other_inputs, &input_fn.sig);

    let router_expr = build_router_expr(&methods, &path, &input_fn.sig.ident);
    expand(new_sig, input_fn.block, router_expr, q_struct, inject_segs)
}

mod derive_route_macro;
use derive_route_macro::make_wrapper;

use crate::{handle_input::handle_dep_attr, process::RouteAttrType};

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

mod arg_parser;
#[proc_macro_attribute]
pub fn main(attr: TokenStream, item: TokenStream) -> TokenStream {
    let args = parse_macro_input!(attr as arg_parser::MainArgs);
    let input_fn = parse_macro_input!(item as ItemFn);

    let config_expr = if let Some(path) = args.config {
        quote! { ::exum::config::ApplicationConfig::from_file(#path) }
    } else {
        quote! { ::exum::config::ApplicationConfig::load() }
    };

    let vis = &input_fn.vis;
    let block = &input_fn.block;

    quote! {
        #[tokio::main]
        #vis async fn main() {
            let _CONFIG = #config_expr;
            init_global_state().await;
            let mut app = ::exum::Application::build(_CONFIG);
            {
                #block
            }
            global_container().prewarm_all().await;
            app.run().await;
        }
    }
    .into()
}

#[proc_macro_attribute]
pub fn state(args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);
    let args = parse_macro_input!(args as arg_parser::StateArgs);
    let prewarm = args.prewarm;

    let fn_name = &input_fn.sig.ident;
    let vis = &input_fn.vis;
    let sig = &input_fn.sig;
    let block = &input_fn.block;

    let output_ty = match &input_fn.sig.output {
        syn::ReturnType::Type(_, ty) => ty.clone(),
        syn::ReturnType::Default => {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "state function must return a type",
            )
            .to_compile_error()
            .into();
        }
    };

    let init_fn_name = format_ident!("__init_{}", fn_name);
    let def_fn_name = format_ident!("__state_def_{}", fn_name);

    let expanded = quote! {
        #vis #sig #block

        #[allow(non_upper_case_globals)]
        fn #init_fn_name() -> ::std::pin::Pin<
            ::std::boxed::Box<
                dyn ::std::future::Future<
                    Output = ::std::sync::Arc<
                        dyn ::std::any::Any + Send + Sync
                    >
                > + Send
            >
        > {
            Box::pin(async {
                let val: #output_ty = #fn_name().await;
                ::std::sync::Arc::new(val) as ::std::sync::Arc<dyn ::std::any::Any + Send + Sync>
            })
        }

        fn #def_fn_name() -> ::exum::StateDef {
            ::exum::StateDef {
                type_id: ::std::any::TypeId::of::<#output_ty>(),
                prewarm: #prewarm,
                init_fn: #init_fn_name,
            }
        }

        ::inventory::submit! {
            ::exum::StateDefFn(#def_fn_name)
        }
    };

    expanded.into()
}

mod process;

#[proc_macro_attribute]
pub fn controller(attr: TokenStream, item: TokenStream) -> TokenStream {
    let prefix = parse_macro_input!(attr as syn::LitStr).value();
    let mut impl_block = parse_macro_input!(item as ItemImpl);
    for item in &mut impl_block.items {
        if let syn::ImplItem::Fn(method) = item {
            for attr in &mut method.attrs {
                if let Some(ident) = attr.path().get_ident() {
                    let name = ident.to_string();
                    match process::valid_route_macro(&name) {
                        RouteAttrType::Route => {
                            let mut new_tokens = proc_macro2::TokenStream::new();
                            let mut has_path = false;
                            let _ = attr.parse_nested_meta(|meta| {
                                if meta.path.is_ident("path") {
                                    let lit: syn::LitStr = meta.value()?.parse()?;
                                    let joined = process::join_path(&prefix, &lit.value()); 
                                    new_tokens.extend(quote!(path = #joined,));
                                    has_path = true;
                                } else {
                                    let method = meta.input.to_string();
                                    new_tokens.extend(quote! {#method,});
                                }
                                Ok(())
                            });
                            *attr = syn::parse_quote!(#[#ident(#new_tokens)])
                        }
                        RouteAttrType::Derive => {
                            let lit = attr.parse_args::<syn::LitStr>().unwrap();
                            let joined = process::join_path(&prefix, &lit.value());
                            *attr = syn::parse_quote! {#[#ident(#joined)]}
                        }
                        RouteAttrType::Not => {}
                    }
                }
            }
        }
    }
    let items = impl_block.items;
    TokenStream::from(quote!{
        #(#items)*
    })
}